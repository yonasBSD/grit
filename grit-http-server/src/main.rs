//! Minimal Git smart HTTP server powered by grit.
//!
//! Implements the four endpoints required for git clone/push over HTTP:
//!   GET  /:repo/info/refs?service=git-upload-pack
//!   GET  /:repo/info/refs?service=git-receive-pack
//!   POST /:repo/git-upload-pack
//!   POST /:repo/git-receive-pack

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use clap::Parser;

#[derive(Parser)]
#[command(name = "grit-http-server", about = "Git smart HTTP server powered by grit")]
struct Args {
    /// Root directory containing bare repositories
    #[arg(short, long, default_value = ".")]
    root: PathBuf,

    /// Address to bind to
    #[arg(short, long, default_value = "127.0.0.1:9418")]
    bind: String,
}

#[derive(Clone)]
struct AppState {
    root: PathBuf,
}

impl AppState {
    fn repo_path(&self, repo: &str) -> Result<PathBuf, Response> {
        // Sanitize: reject path traversal
        if repo.contains("..") {
            return Err((StatusCode::BAD_REQUEST, "invalid repository path").into_response());
        }
        let path = self.root.join(repo);
        grit_protocol::validate_repo_path(&path).map_err(|_| {
            (StatusCode::NOT_FOUND, "repository not found").into_response()
        })
    }
}

#[derive(serde::Deserialize)]
struct InfoRefsQuery {
    service: Option<String>,
}

fn pkt_line(data: &str) -> Vec<u8> {
    let len = data.len() + 4;
    format!("{len:04x}{data}").into_bytes()
}

fn no_cache_headers(builder: axum::http::response::Builder) -> axum::http::response::Builder {
    builder
        .header(header::EXPIRES, "Fri, 01 Jan 1980 00:00:00 GMT")
        .header(header::PRAGMA, "no-cache")
        .header(header::CACHE_CONTROL, "no-cache, max-age=0, must-revalidate")
}

// GET /:repo/info/refs?service=git-upload-pack or git-receive-pack
async fn info_refs(
    State(state): State<Arc<AppState>>,
    Path(repo): Path<String>,
    Query(query): Query<InfoRefsQuery>,
) -> Response {
    let service = match &query.service {
        Some(s) if s == "git-upload-pack" || s == "git-receive-pack" => s.clone(),
        _ => {
            return (StatusCode::BAD_REQUEST, "unsupported service").into_response();
        }
    };

    let repo_path = match state.repo_path(&repo) {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Parse protocol version from query (Git protocol v2 sends it via Git-Protocol header,
    // but for the PoC we just support v1 and detect v2 from the environment)
    let protocol_version = None; // v1 for now

    let content_type = format!("application/x-{service}-advertisement");

    let advertisement = match service.as_str() {
        "git-upload-pack" => {
            grit_protocol::upload_pack::advertise_refs(&repo_path, protocol_version)
        }
        "git-receive-pack" => grit_protocol::receive_pack::advertise_refs(&repo_path),
        _ => unreachable!(),
    };

    let raw_adv = match advertisement {
        Ok(data) => data,
        Err(e) => {
            eprintln!("info/refs error: {e:#}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response();
        }
    };

    // Build the response: pkt-line service header + flush + raw advertisement
    let mut body = Vec::new();
    body.extend_from_slice(&pkt_line(&format!("# service={service}\n")));
    body.extend_from_slice(b"0000"); // flush-pkt
    body.extend_from_slice(&raw_adv);

    no_cache_headers(axum::http::Response::builder())
        .header(header::CONTENT_TYPE, content_type)
        .body(axum::body::Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

// POST /:repo/git-upload-pack
async fn upload_pack_rpc(
    State(state): State<Arc<AppState>>,
    Path(repo): Path<String>,
    body: Bytes,
) -> Response {
    let repo_path = match state.repo_path(&repo) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let protocol_version = None;

    match grit_protocol::upload_pack::stateless_rpc(&repo_path, &body, protocol_version) {
        Ok(data) => no_cache_headers(axum::http::Response::builder())
            .header(header::CONTENT_TYPE, "application/x-git-upload-pack-result")
            .body(axum::body::Body::from(data))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        Err(e) => {
            eprintln!("upload-pack error: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, "upload-pack failed").into_response()
        }
    }
}

// POST /:repo/git-receive-pack
async fn receive_pack_rpc(
    State(state): State<Arc<AppState>>,
    Path(repo): Path<String>,
    body: Bytes,
) -> Response {
    let repo_path = match state.repo_path(&repo) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match grit_protocol::receive_pack::stateless_rpc(&repo_path, &body) {
        Ok(data) => no_cache_headers(axum::http::Response::builder())
            .header(header::CONTENT_TYPE, "application/x-git-receive-pack-result")
            .body(axum::body::Body::from(data))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        Err(e) => {
            eprintln!("receive-pack error: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, "receive-pack failed").into_response()
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let root = std::fs::canonicalize(&args.root)?;
    eprintln!("Serving repositories from: {}", root.display());

    let state = Arc::new(AppState { root });

    let app = Router::new()
        .route("/{*repo}", get(info_refs_dispatch).post(rpc_dispatch))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    eprintln!("Listening on {}", args.bind);
    axum::serve(listener, app).await?;

    Ok(())
}

// Dispatch GET requests — only info/refs is valid
async fn info_refs_dispatch(
    state: State<Arc<AppState>>,
    Path(full_path): Path<String>,
    query: Query<InfoRefsQuery>,
) -> Response {
    if let Some(repo) = full_path.strip_suffix("/info/refs") {
        if repo.is_empty() {
            return (StatusCode::BAD_REQUEST, "missing repository").into_response();
        }
        info_refs(state, Path(repo.to_string()), query).await
    } else {
        (StatusCode::NOT_FOUND, "not found").into_response()
    }
}

// Dispatch POST requests to upload-pack or receive-pack
async fn rpc_dispatch(
    state: State<Arc<AppState>>,
    Path(full_path): Path<String>,
    body: Bytes,
) -> Response {
    if let Some(repo) = full_path.strip_suffix("/git-upload-pack") {
        if repo.is_empty() {
            return (StatusCode::BAD_REQUEST, "missing repository").into_response();
        }
        upload_pack_rpc(state, Path(repo.to_string()), body).await
    } else if let Some(repo) = full_path.strip_suffix("/git-receive-pack") {
        if repo.is_empty() {
            return (StatusCode::BAD_REQUEST, "missing repository").into_response();
        }
        receive_pack_rpc(state, Path(repo.to_string()), body).await
    } else {
        (StatusCode::NOT_FOUND, "not found").into_response()
    }
}
