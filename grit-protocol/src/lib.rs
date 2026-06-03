//! Git smart protocol handlers for HTTP transport.
//!
//! Provides a clean Rust API for running git upload-pack and receive-pack
//! operations. Currently implemented by spawning `grit` as a subprocess
//! with piped I/O — the same model as `git-http-backend`.
//!
//! Future work: replace subprocess calls with in-process protocol handling
//! once the core protocol logic is libified with generic Read/Write streams.

pub mod upload_pack;
pub mod receive_pack;

use std::path::{Path, PathBuf};

/// Find the grit executable path.
///
/// Checks `GUST_BIN` env var first (for test harness compatibility),
/// then falls back to `grit` on PATH.
pub fn grit_executable() -> PathBuf {
    if let Ok(bin) = std::env::var("GUST_BIN") {
        if !bin.is_empty() {
            return PathBuf::from(bin);
        }
    }
    // Try the same binary that's running (if we're inside grit)
    if let Ok(exe) = std::env::current_exe() {
        let name = exe.file_name().unwrap_or_default().to_string_lossy();
        if name.starts_with("grit") {
            return exe;
        }
    }
    PathBuf::from("grit")
}

/// Validate that a path looks like a git repository (bare or non-bare).
pub fn validate_repo_path(path: &Path) -> anyhow::Result<PathBuf> {
    // Bare repo: has HEAD file directly
    if path.join("HEAD").is_file() {
        return Ok(path.to_path_buf());
    }
    // Non-bare: has .git/HEAD
    let dot_git = path.join(".git");
    if dot_git.join("HEAD").is_file() {
        return Ok(path.to_path_buf());
    }
    anyhow::bail!("not a git repository: {}", path.display())
}
