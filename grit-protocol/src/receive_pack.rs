//! Receive-pack protocol handler (server side of push).
//!
//! Wraps `grit receive-pack` subprocess with piped I/O for use in
//! HTTP smart transport.

use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Find the byte offset of the first flush packet (`0000`) in pkt-line data.
/// Returns the offset of the `0000` itself, or `None` if not found.
fn find_flush(data: &[u8]) -> Option<usize> {
    let mut pos = 0;
    while pos + 4 <= data.len() {
        let len_hex = &data[pos..pos + 4];
        if len_hex == b"0000" {
            return Some(pos);
        }
        if let Ok(s) = std::str::from_utf8(len_hex) {
            if let Ok(len) = usize::from_str_radix(s, 16) {
                if len < 4 || pos + len > data.len() {
                    break;
                }
                pos += len;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    None
}

/// Return only the ref advertisement (up to and including the flush).
///
/// `grit receive-pack` with stdin closed outputs the advertisement
/// followed by a spurious report-status. We truncate at the first flush.
fn advertisement_only(data: &[u8]) -> Vec<u8> {
    match find_flush(data) {
        Some(pos) => data[..pos + 4].to_vec(),
        None => data.to_vec(),
    }
}

/// Strip the ref advertisement prefix from receive-pack output,
/// returning only the report-status that follows the first flush.
fn strip_advertisement(data: &[u8]) -> Vec<u8> {
    match find_flush(data) {
        Some(pos) => data[pos + 4..].to_vec(),
        None => data.to_vec(),
    }
}

/// Run receive-pack ref advertisement (for `GET /info/refs?service=git-receive-pack`).
///
/// Returns the raw pkt-line advertisement bytes.
pub fn advertise_refs(repo_path: &Path) -> Result<Vec<u8>> {
    let grit = crate::grit_executable();
    let mut cmd = Command::new(&grit);
    cmd.arg("receive-pack")
        .arg(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // For info/refs discovery, we need to get just the advertisement.
    // receive-pack writes the advertisement then reads from stdin.
    // We close stdin immediately to make it exit after advertising.
    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to spawn '{} receive-pack'", grit.display()))?;

    // Close stdin immediately — receive-pack will write its ref advertisement
    // and then try to read, hitting EOF and exiting.
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .context("failed to wait for receive-pack")?;

    // receive-pack may exit non-zero when stdin closes early — that's expected
    // for the advertisement phase. We just need the stdout.
    if output.stdout.is_empty() && !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("receive-pack advertisement failed: {}", err.trim());
    }

    Ok(advertisement_only(&output.stdout))
}

/// Run a stateless receive-pack RPC exchange (for `POST /git-receive-pack`).
///
/// Takes the request body (pkt-line commands + pack data) and returns
/// the response (report-status).
pub fn stateless_rpc(
    repo_path: &Path,
    request_body: &[u8],
) -> Result<Vec<u8>> {
    let grit = crate::grit_executable();
    let mut cmd = Command::new(&grit);
    cmd.arg("receive-pack")
        .arg(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to spawn '{} receive-pack'", grit.display()))?;

    // Write the request body to stdin.
    // receive-pack first writes its advertisement to stdout, then reads from stdin.
    // For HTTP stateless mode, the advertisement was already sent in the info/refs phase.
    // We need to feed the push data after the advertisement is written.
    if let Some(mut stdin) = child.stdin.take() {
        // Write request body — the push commands + pack data
        let _ = stdin.write_all(request_body);
        // Close stdin to signal end of input
    }

    let output = child
        .wait_with_output()
        .context("failed to wait for receive-pack")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        if !output.stdout.is_empty() {
            eprintln!("receive-pack stderr (non-fatal): {}", err.trim());
            return Ok(strip_advertisement(&output.stdout));
        }
        anyhow::bail!("receive-pack failed: {}", err.trim());
    }

    Ok(strip_advertisement(&output.stdout))
}
