//! Upload-pack protocol handler (server side of fetch/clone).
//!
//! Wraps `grit upload-pack` subprocess with piped I/O for use in
//! HTTP smart transport.

use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Run upload-pack ref advertisement (for `GET /info/refs?service=git-upload-pack`).
///
/// Returns the raw pkt-line advertisement bytes suitable for wrapping
/// in an HTTP response with service header.
pub fn advertise_refs(
    repo_path: &Path,
    protocol_version: Option<u8>,
) -> Result<Vec<u8>> {
    let grit = crate::grit_executable();
    let mut cmd = Command::new(&grit);
    cmd.arg("upload-pack")
        .arg("--stateless-rpc")
        .arg("--http-backend-info-refs")
        .arg(repo_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(v) = protocol_version {
        cmd.env("GIT_PROTOCOL", format!("version={v}"));
    }

    let output = cmd
        .output()
        .with_context(|| format!("failed to spawn '{} upload-pack'", grit.display()))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("upload-pack --advertise-refs failed: {}", err.trim());
    }

    Ok(output.stdout)
}

/// Run a stateless upload-pack RPC exchange (for `POST /git-upload-pack`).
///
/// Takes the request body as input and returns the response body.
/// Supports both protocol v1 (stateless-rpc) and v2.
pub fn stateless_rpc(
    repo_path: &Path,
    request_body: &[u8],
    protocol_version: Option<u8>,
) -> Result<Vec<u8>> {
    let grit = crate::grit_executable();
    let mut cmd = Command::new(&grit);

    if protocol_version == Some(2) {
        // Protocol v2 uses serve-v2 --stateless-rpc
        cmd.arg("serve-v2")
            .arg("--stateless-rpc");
    } else {
        cmd.arg("upload-pack")
            .arg("--stateless-rpc")
            .arg(repo_path);
    }

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(v) = protocol_version {
        cmd.env("GIT_PROTOCOL", format!("version={v}"));
    }

    // For serve-v2, set GIT_DIR since it discovers from cwd
    if protocol_version == Some(2) {
        cmd.current_dir(repo_path);
    }

    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to spawn '{} upload-pack'", grit.display()))?;

    // Write request body to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(request_body)?;
    }

    let output = child
        .wait_with_output()
        .context("failed to wait for upload-pack")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        // Don't fail on non-zero exit for upload-pack — some errors are normal
        // (e.g., client disconnect). Log but return what we have.
        if !output.stdout.is_empty() {
            eprintln!("upload-pack stderr (non-fatal): {}", err.trim());
            return Ok(output.stdout);
        }
        anyhow::bail!("upload-pack failed: {}", err.trim());
    }

    Ok(output.stdout)
}
