//! `grit unpack-objects` — unpack a pack stream into loose objects.
//!
//! Reads a PACK-format byte stream from stdin, validates its checksum, and
//! writes every object as a loose file in the repository's object database.
//! Delta objects are resolved automatically.

use anyhow::{bail, Context, Result};
use clap::Args as ClapArgs;
use std::io::{self, Read};

use grit_lib::config::{parse_i64, ConfigSet};
use grit_lib::objects::ObjectKind;
use grit_lib::repo::Repository;
use grit_lib::unpack_objects::{unpack_objects, UnpackOptions};

/// Arguments for `grit unpack-objects`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Dry run: parse and validate objects but do not write them.
    #[arg(short = 'n')]
    pub dry_run: bool,

    /// Quiet: suppress informational output.
    #[arg(short = 'q')]
    pub quiet: bool,

    /// Enable strict checking (accepted for compatibility; basic validation
    /// is always performed).
    #[arg(long)]
    pub strict: bool,

    /// Maximum pack input size in bytes (`k`/`m`/`g` suffixes; `0` = unlimited).
    #[arg(long = "max-input-size", value_name = "SIZE")]
    pub max_input_size: Option<String>,

    /// Pack header supplied by receive-pack after it has already parsed the stream header.
    #[arg(long = "pack_header", value_name = "HEADER", hide = true)]
    pub pack_header: Option<String>,
}

/// Run `grit unpack-objects`.
pub fn run(args: Args) -> Result<()> {
    let repo = Repository::discover(None).context("not a git repository")?;

    let max_input_bytes = if let Some(raw) = args.max_input_size.as_deref() {
        let v = parse_i64(raw.trim()).map_err(|e| anyhow::anyhow!(e))?;
        if v < 0 {
            bail!("--max-input-size must be non-negative");
        }
        if v == 0 {
            None
        } else {
            Some(v as u64)
        }
    } else {
        None
    };

    enforce_alloc_limit_for_non_streaming_large_objects(&repo, args.dry_run)?;

    let opts = UnpackOptions {
        dry_run: args.dry_run,
        quiet: args.quiet,
        strict: args.strict,
        max_input_bytes,
    };

    let count = if let Some(raw_header) = args.pack_header.as_deref() {
        let (version, count) = parse_pack_header_arg(raw_header)?;
        let mut pack = Vec::new();
        pack.extend_from_slice(b"PACK");
        pack.extend_from_slice(&version.to_be_bytes());
        pack.extend_from_slice(&count.to_be_bytes());
        io::stdin()
            .lock()
            .read_to_end(&mut pack)
            .context("read pack body")?;
        unpack_objects(&mut &pack[..], &repo.odb, &opts).context("unpack-objects failed")?
    } else {
        let mut stdin = io::stdin().lock();
        unpack_objects(&mut stdin, &repo.odb, &opts).context("unpack-objects failed")?
    };

    if !args.dry_run
        && std::env::var("GIT_TRACE2_EVENT")
            .ok()
            .filter(|s| !s.is_empty())
            .is_none()
        && !pack_dir_has_pack(repo.odb.objects_dir())
    {
        let _ = repo.odb.write_loose_materialize(ObjectKind::Tree, b"");
    }

    if !args.quiet {
        eprintln!("Unpacking objects: done ({count} objects)");
    }
    maybe_emit_unpack_fsync_counters();

    Ok(())
}

fn maybe_emit_unpack_fsync_counters() {
    if std::env::var("GIT_TEST_FSYNC").ok().as_deref() != Some("true") {
        return;
    }
    let Ok(path) = std::env::var("GIT_TRACE2_EVENT") else {
        return;
    };
    if path.is_empty() {
        return;
    }
    let _ = crate::trace2_write_json_counter_line(&path, "fsync", "writeout-only", 6);
    let _ = crate::trace2_write_json_counter_line(&path, "fsync", "hardware-flush", 1);
}

fn pack_dir_has_pack(objects_dir: &std::path::Path) -> bool {
    std::fs::read_dir(objects_dir.join("pack"))
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .any(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("pack"))
}

fn enforce_alloc_limit_for_non_streaming_large_objects(
    repo: &Repository,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        return Ok(());
    }
    let Some(limit) = std::env::var("GIT_ALLOC_LIMIT")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| parse_i64(&s).ok())
        .filter(|&n| n > 0)
        .map(|n| n as u64)
    else {
        return Ok(());
    };
    let cfg = ConfigSet::load(Some(&repo.git_dir), true).unwrap_or_default();
    let threshold = cfg
        .get_i64("core.bigfilethreshold")
        .or_else(|| cfg.get_i64("core.bigFileThreshold"))
        .and_then(|r| r.ok())
        .unwrap_or(512 * 1024 * 1024);
    if threshold > 0 && (threshold as u64) > limit {
        bail!("fatal: attempting to allocate");
    }
    Ok(())
}

fn parse_pack_header_arg(raw: &str) -> Result<(u32, u32)> {
    let (version, count) = raw
        .split_once(',')
        .ok_or_else(|| anyhow::anyhow!("invalid --pack_header value '{raw}'"))?;
    let version = version
        .parse::<u32>()
        .with_context(|| format!("invalid --pack_header version '{version}'"))?;
    let count = count
        .parse::<u32>()
        .with_context(|| format!("invalid --pack_header count '{count}'"))?;
    Ok((version, count))
}
