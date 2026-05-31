//! `grit multi-pack-index` — manage multi-pack index files.
//!
//! [`verify`](MpiCommand::Verify) checks active MIDX layer(s) (root file or chain in
//! `multi-pack-index.d`). [`write`](MpiCommand::Write) builds a new MIDX from pack indexes,
//! including incremental split layout when `--incremental` is set.

use anyhow::{bail, Context, Result};
use clap::{Args as ClapArgs, Subcommand};
use grit_lib::midx::{
    read_midx_objects, write_multi_pack_index_with_options, WriteMultiPackIndexOptions,
};
use grit_lib::pack::read_pack_index;
use grit_lib::repo::Repository;
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::grit_exe;

/// Arguments for `grit multi-pack-index`.
#[derive(Debug, ClapArgs)]
#[command(about = "Manage multi-pack index")]
pub struct Args {
    #[command(subcommand)]
    pub command: MpiCommand,
}

#[derive(Debug, Subcommand)]
pub enum MpiCommand {
    /// Check the MIDX file for consistency (header and version).
    Verify(VerifyArgs),
    /// Build a new multi-pack index from existing pack indexes.
    Write(WriteArgs),
    /// Run `grit repack -d`, then write the multi-pack index.
    Repack(RepackArgs),
    /// Delete packfiles no longer referenced by the multi-pack index.
    Expire(ExpireArgs),
    /// Rewrite the multi-pack index from all packs (no incremental chain merge).
    Compact(CompactArgs),
}

#[derive(Debug, ClapArgs)]
pub struct VerifyArgs {}

#[derive(Debug, ClapArgs)]
pub struct WriteArgs {
    /// Write an incremental MIDX layer (split layout under `multi-pack-index.d`).
    #[arg(long)]
    pub incremental: bool,
    /// Write placeholder bitmap sidecar (compat with Git `--bitmap`).
    #[arg(long)]
    pub bitmap: bool,
    /// Omit bitmap / `.rev` sidecars even when `--bitmap` would write them.
    #[arg(long = "no-bitmap")]
    pub no_bitmap: bool,
    /// Preferred pack basename (`pack-<hash>.idx` or `.pack`); RIDX position 0 in the MIDX.
    #[arg(long = "preferred-pack", value_name = "FILE")]
    pub preferred_pack: Option<String>,
    /// Read `pack-*.idx` basenames from stdin (one per line) to include in order.
    #[arg(long = "stdin-packs")]
    pub stdin_packs: bool,
    /// Suppress progress (accepted for compat).
    #[arg(long = "no-progress")]
    pub no_progress: bool,
    /// Show progress (accepted for compat).
    #[arg(long = "progress")]
    pub progress: bool,
}

#[derive(Debug, ClapArgs)]
pub struct RepackArgs {
    /// Suppress progress (accepted for compat).
    #[arg(long = "no-progress")]
    pub no_progress: bool,
    /// Show progress (accepted for compat).
    #[arg(long = "progress")]
    pub progress: bool,
    /// Maximum total size (in bytes) of packs to combine; `0` repacks everything.
    #[arg(long = "batch-size", default_value_t = 0)]
    pub batch_size: u64,
}

#[derive(Debug, ClapArgs)]
pub struct ExpireArgs {
    /// Suppress progress (accepted for compat).
    #[arg(long = "no-progress")]
    pub no_progress: bool,
    /// Show progress (accepted for compat).
    #[arg(long = "progress")]
    pub progress: bool,
}

#[derive(Debug, ClapArgs)]
pub struct CompactArgs {}

/// Run `grit multi-pack-index`.
pub fn run(args: Args) -> Result<()> {
    let repo = Repository::discover(None).context("not a git repository")?;
    match args.command {
        MpiCommand::Verify(_) => cmd_verify(&repo),
        MpiCommand::Write(w) => cmd_write(&repo, &w),
        MpiCommand::Repack(a) => cmd_repack(&repo, &a),
        MpiCommand::Expire(_) => cmd_expire(&repo),
        MpiCommand::Compact(_) => cmd_compact(&repo),
    }
}

/// Parse argv when clap would reject unknown global flags (e.g. `--object-dir` before `write`).
pub fn run_from_argv(argv: &[String]) -> Result<()> {
    let mut object_dir: Option<PathBuf> = None;
    let mut rest: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < argv.len() {
        let a = argv[i].as_str();
        if a == "--object-dir" {
            let Some(val) = argv.get(i + 1) else {
                bail!("--object-dir requires a path");
            };
            object_dir = Some(PathBuf::from(val));
            i += 2;
            continue;
        }
        if let Some(val) = a.strip_prefix("--object-dir=") {
            object_dir = Some(PathBuf::from(val));
            i += 1;
            continue;
        }
        rest.push(argv[i].clone());
        i += 1;
    }

    if let Some(dir) = object_dir {
        std::env::set_var("GIT_OBJECT_DIRECTORY", dir);
    }

    let sub = rest.first().map(|s| s.as_str()).unwrap_or("");
    let repo = Repository::discover(None).context("not a git repository")?;
    match sub {
        "write" => {
            let mut incremental = false;
            let mut bitmap = false;
            let mut no_bitmap = false;
            let mut preferred_pack = None;
            let mut stdin_packs = false;
            let mut no_progress = false;
            let mut progress = false;
            let mut i = 1usize;
            while i < rest.len() {
                let a = rest[i].as_str();
                match a {
                    "--incremental" => incremental = true,
                    "--bitmap" => bitmap = true,
                    "--no-bitmap" => no_bitmap = true,
                    "--stdin-packs" => stdin_packs = true,
                    "--no-progress" => no_progress = true,
                    "--progress" => progress = true,
                    _ if a.starts_with("--preferred-pack=") => {
                        preferred_pack = Some(a["--preferred-pack=".len()..].to_string());
                    }
                    "--preferred-pack" => {
                        let Some(v) = rest.get(i + 1) else {
                            bail!("--preferred-pack requires a value");
                        };
                        preferred_pack = Some(v.clone());
                        i += 1;
                    }
                    other => bail!("unsupported multi-pack-index write option: {other}"),
                }
                i += 1;
            }
            cmd_write(
                &repo,
                &WriteArgs {
                    incremental,
                    bitmap,
                    no_bitmap,
                    preferred_pack,
                    stdin_packs,
                    no_progress,
                    progress,
                },
            )
        }
        "verify" => cmd_verify(&repo),
        "repack" => {
            let mut no_progress = false;
            let mut progress = false;
            let mut batch_size: u64 = 0;
            let mut i = 1usize;
            while i < rest.len() {
                let a = rest[i].as_str();
                if a == "--no-progress" {
                    no_progress = true;
                } else if a == "--progress" {
                    progress = true;
                } else if let Some(v) = a.strip_prefix("--batch-size=") {
                    batch_size = v
                        .parse()
                        .with_context(|| format!("invalid --batch-size value: {v}"))?;
                } else if a == "--batch-size" {
                    let Some(v) = rest.get(i + 1) else {
                        bail!("--batch-size requires a value");
                    };
                    batch_size = v
                        .parse()
                        .with_context(|| format!("invalid --batch-size value: {v}"))?;
                    i += 1;
                } else {
                    bail!("unsupported multi-pack-index repack option: {a}");
                }
                i += 1;
            }
            cmd_repack(
                &repo,
                &RepackArgs {
                    no_progress,
                    progress,
                    batch_size,
                },
            )
        }
        "expire" => {
            for a in rest.iter().skip(1) {
                if a == "--no-progress" || a == "--progress" {
                    // accepted for compat
                } else {
                    bail!("unsupported multi-pack-index expire option: {a}");
                }
            }
            cmd_expire(&repo)
        }
        "compact" => {
            if rest.len() > 1 {
                bail!("unsupported multi-pack-index compact arguments");
            }
            cmd_compact(&repo)
        }
        other => bail!("unsupported multi-pack-index subcommand: {other}"),
    }
}

fn objects_dir_for_repo(repo: &Repository) -> PathBuf {
    if let Ok(rel) = std::env::var("GIT_OBJECT_DIRECTORY") {
        let base = repo.work_tree.as_deref().unwrap_or(&repo.git_dir);
        base.join(rel)
    } else {
        repo.git_dir.join("objects")
    }
}

fn pack_dir(repo: &Repository) -> PathBuf {
    objects_dir_for_repo(repo).join("pack")
}

fn cmd_write(repo: &Repository, args: &WriteArgs) -> Result<()> {
    let write_rev = std::env::var("GIT_TEST_MIDX_WRITE_REV").ok().as_deref() == Some("1");
    let pack_names_subset_ordered = if args.stdin_packs {
        let stdin = io::stdin();
        let mut lines = Vec::new();
        for line in stdin.lock().lines() {
            let line = line.context("read stdin for multi-pack-index --stdin-packs")?;
            let t = line.trim();
            if !t.is_empty() {
                lines.push(t.to_string());
            }
        }
        Some(lines)
    } else {
        None
    };
    let write_bitmaps = args.bitmap && !args.no_bitmap;
    write_multi_pack_index_with_options(
        &pack_dir(repo),
        &WriteMultiPackIndexOptions {
            preferred_pack_idx: None,
            preferred_pack_name: args.preferred_pack.clone(),
            pack_names_subset_ordered,
            write_bitmap_placeholders: write_bitmaps,
            incremental: args.incremental,
            write_rev_placeholder: write_rev && write_bitmaps,
        },
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}

fn cmd_repack(repo: &Repository, args: &RepackArgs) -> Result<()> {
    let objects_dir = objects_dir_for_repo(repo);

    // Without an existing MIDX there is nothing to drive the batch selection;
    // fall back to repacking everything and (re)writing the MIDX (matches the
    // `--batch-size=0` "repack all" behavior used by t5319).
    let pd = pack_dir(repo);
    let have_midx = pd.join("multi-pack-index").exists() || midx_chain_path(&pd).exists();
    if !have_midx {
        return repack_all_and_write_midx(repo);
    }

    let (names, objects) = read_midx_objects(&objects_dir).map_err(|e| anyhow::anyhow!("{e}"))?;
    let include = if args.batch_size > 0 {
        select_packs_for_batch(&objects_dir, &names, &objects, args.batch_size)?
    } else {
        // batch-size 0 => include every (local, non-cruft) pack referenced by the MIDX.
        let mut set = HashSet::new();
        for o in &objects {
            set.insert(o.pack_int_id);
        }
        // Drop cruft packs from the candidate set.
        set.retain(|&id| {
            names
                .get(id)
                .map(|n| !is_cruft_idx_name(&objects_dir, n))
                .unwrap_or(false)
        });
        set
    };

    if include.len() <= 1 {
        // Nothing meaningful to combine; leave packs untouched.
        return Ok(());
    }

    // Gather the OIDs attributed to the included packs and feed them to
    // pack-objects, producing a single new pack.
    let mut oids: Vec<String> = Vec::new();
    for o in &objects {
        if include.contains(&o.pack_int_id) {
            oids.push(o.oid.to_hex());
        }
    }

    let work_dir = repo.work_tree.as_deref().unwrap_or(&repo.git_dir);
    let base = pd.join("pack");
    let grit = grit_exe::grit_executable();
    let mut cmd = Command::new(&grit);
    cmd.current_dir(work_dir)
        .arg("pack-objects")
        .arg(base.to_string_lossy().to_string())
        .arg("--delta-base-offset");
    if args.no_progress || !args.progress {
        cmd.arg("-q");
    }
    cmd.stdin(Stdio::piped()).stdout(Stdio::null());
    let mut child = cmd.spawn().context("could not start pack-objects")?;
    {
        let mut stdin = child.stdin.take().context("pack-objects stdin")?;
        for oid in &oids {
            writeln!(stdin, "{oid}")?;
        }
    }
    let status = child.wait().context("could not finish pack-objects")?;
    if !status.success() {
        bail!("pack-objects failed with status {status}");
    }

    write_multi_pack_index_with_options(&pack_dir(repo), &WriteMultiPackIndexOptions::default())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

fn repack_all_and_write_midx(repo: &Repository) -> Result<()> {
    let work_dir = repo.work_tree.as_deref().unwrap_or(&repo.git_dir);
    let mut cmd = Command::new(grit_exe::grit_executable());
    cmd.current_dir(work_dir).args(["repack", "-d", "-q"]);
    let status = cmd
        .status()
        .context("failed to run grit repack for multi-pack-index")?;
    if !status.success() {
        bail!("repack failed with status {status}");
    }
    write_multi_pack_index_with_options(&pack_dir(repo), &WriteMultiPackIndexOptions::default())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// `git multi-pack-index expire`: delete packfiles no longer referenced by the
/// MIDX (every object they held is provided by another pack), then rewrite the
/// MIDX over the survivors. Mirrors `expire_midx_packs` in git/midx-write.c.
fn cmd_expire(repo: &Repository) -> Result<()> {
    let objects_dir = objects_dir_for_repo(repo);
    let pd = pack_dir(repo);
    if !pd.join("multi-pack-index").exists() && !midx_chain_path(&pd).exists() {
        return Ok(());
    }
    let (names, objects) = read_midx_objects(&objects_dir).map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut count = vec![0usize; names.len()];
    for o in &objects {
        if let Some(c) = count.get_mut(o.pack_int_id) {
            *c += 1;
        }
    }

    let mut survivors: Vec<String> = Vec::new();
    let mut to_drop: Vec<String> = Vec::new();
    for (i, name) in names.iter().enumerate() {
        // Never expire cruft packs.
        let keep = count.get(i).copied().unwrap_or(0) > 0 || is_cruft_idx_name(&objects_dir, name);
        if keep {
            survivors.push(name.clone());
        } else {
            to_drop.push(name.clone());
        }
    }

    if to_drop.is_empty() {
        return Ok(());
    }

    // Rewrite the MIDX over the surviving packs before removing files.
    if survivors.is_empty() {
        // No packs left to index: just clear the MIDX and drop the packs.
        grit_lib::midx::clear_pack_midx_state(&pd).map_err(|e| anyhow::anyhow!("{e}"))?;
    } else {
        write_multi_pack_index_with_options(
            &pd,
            &WriteMultiPackIndexOptions {
                pack_names_subset_ordered: Some(survivors.clone()),
                ..Default::default()
            },
        )
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    }

    for idx_name in &to_drop {
        remove_pack_files(&pd, idx_name);
    }
    Ok(())
}

/// Remove every sidecar of a pack given its `.idx` basename (`pack-<hash>.idx`).
fn remove_pack_files(pack_dir: &Path, idx_name: &str) {
    let stem = idx_name.strip_suffix(".idx").unwrap_or(idx_name);
    for ext in ["idx", "pack", "rev", "bitmap", "mtimes", "keep", "promisor"] {
        let _ = fs::remove_file(pack_dir.join(format!("{stem}.{ext}")));
    }
}

fn is_cruft_idx_name(objects_dir: &Path, idx_name: &str) -> bool {
    let stem = idx_name.strip_suffix(".idx").unwrap_or(idx_name);
    objects_dir
        .join("pack")
        .join(format!("{stem}.mtimes"))
        .exists()
}

/// Select the set of MIDX pack ids to combine so the estimated total stays
/// under `batch_size`, oldest packs first. Mirrors `fill_included_packs_batch`.
fn select_packs_for_batch(
    objects_dir: &Path,
    names: &[String],
    objects: &[grit_lib::midx::MidxObjectRef],
    batch_size: u64,
) -> Result<HashSet<usize>> {
    let pack_dir = objects_dir.join("pack");

    // Per-pack referenced-object counts (from the MIDX).
    let mut referenced = vec![0u64; names.len()];
    for o in objects {
        if let Some(c) = referenced.get_mut(o.pack_int_id) {
            *c += 1;
        }
    }

    struct Info {
        id: usize,
        mtime: i64,
        referenced: u64,
        num_objects: u64,
        pack_size: u64,
        usable: bool,
    }
    let mut infos: Vec<Info> = Vec::with_capacity(names.len());
    for (id, name) in names.iter().enumerate() {
        let stem = name.strip_suffix(".idx").unwrap_or(name);
        let idx_path = pack_dir.join(format!("{stem}.idx"));
        let pack_path = pack_dir.join(format!("{stem}.pack"));
        let mtime = fs::metadata(&pack_path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let pack_size = fs::metadata(&pack_path).map(|m| m.len()).unwrap_or(0);
        let (num_objects, usable) = match read_pack_index(&idx_path) {
            Ok(pi) => {
                let n = pi.entries.len() as u64;
                // Skip cruft packs and empty packs (Git want_included_pack).
                let cruft = is_cruft_idx_name(objects_dir, name);
                (n, n > 0 && !cruft)
            }
            Err(_) => (0, false),
        };
        infos.push(Info {
            id,
            mtime,
            referenced: referenced.get(id).copied().unwrap_or(0),
            num_objects,
            pack_size,
            usable,
        });
    }

    infos.sort_by(|a, b| a.mtime.cmp(&b.mtime));

    let mut include = HashSet::new();
    let mut total_size: u64 = 0;
    for info in &infos {
        if total_size >= batch_size {
            break;
        }
        if !info.usable || info.num_objects == 0 {
            continue;
        }
        // expected_size = referenced/num_objects * pack_size, via shifted ints.
        let mut expected = (info.referenced as u128) << 14;
        expected /= info.num_objects as u128;
        expected = expected.saturating_mul(info.pack_size as u128);
        expected = (expected + (1u128 << 13)) >> 14;
        let expected = expected.min(u64::MAX as u128) as u64;

        if expected >= batch_size {
            continue;
        }
        total_size = total_size.saturating_add(expected);
        include.insert(info.id);
    }
    Ok(include)
}

fn cmd_compact(repo: &Repository) -> Result<()> {
    write_multi_pack_index_with_options(&pack_dir(repo), &WriteMultiPackIndexOptions::default())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

fn midx_chain_path(pack_dir: &Path) -> PathBuf {
    pack_dir
        .join("multi-pack-index.d")
        .join("multi-pack-index-chain")
}

fn cmd_verify(repo: &Repository) -> Result<()> {
    let pd = pack_dir(repo);
    let root = pd.join("multi-pack-index");
    let chain = midx_chain_path(&pd);
    if root.exists() {
        let data = fs::read(&root).with_context(|| format!("could not read {}", root.display()))?;
        verify_midx_header_bytes(&data).with_context(|| format!("{}", root.display()))?;
        return Ok(());
    }
    if chain.exists() {
        let contents = fs::read_to_string(&chain)
            .with_context(|| format!("could not read {}", chain.display()))?;
        let midx_d = pd.join("multi-pack-index.d");
        for line in contents.lines() {
            let h = line.trim();
            if h.is_empty() {
                continue;
            }
            let path = midx_d.join(format!("multi-pack-index-{h}.midx"));
            let data =
                fs::read(&path).with_context(|| format!("could not read {}", path.display()))?;
            verify_midx_header_bytes(&data).with_context(|| format!("{}", path.display()))?;
        }
        return Ok(());
    }
    bail!(
        "no multi-pack-index at {} or chain at {}",
        root.display(),
        chain.display()
    );
}

/// Validates the leading bytes of a multi-pack-index file.
pub fn verify_midx_header_bytes(data: &[u8]) -> Result<()> {
    const MIDX_SIGNATURE: u32 = 0x4d49_4458; // b"MIDX"

    if data.len() < 12 {
        bail!("multi-pack-index file too small");
    }
    let sig = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    if sig != MIDX_SIGNATURE {
        bail!("bad multi-pack-index signature");
    }
    let version = data[4];
    if version != 1 && version != 2 {
        bail!("unsupported multi-pack-index version {version}");
    }
    let hash_version = data[5];
    if hash_version != 1 {
        bail!("unsupported hash version {hash_version} in multi-pack-index");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_rejects_too_short() {
        assert!(verify_midx_header_bytes(&[0u8; 8]).is_err());
    }

    #[test]
    fn verify_accepts_minimal_v1_header() {
        let mut v = vec![0u8; 12];
        v[0..4].copy_from_slice(b"MIDX");
        v[4] = 1;
        v[5] = 1;
        assert!(verify_midx_header_bytes(&v).is_ok());
    }
}
