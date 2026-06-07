//! Interactive `git add -p` — stage selected hunks from the index↔worktree diff.
//!
//! Uses the same Myers line-diff and hunk-splitting approach as [`crate::commands::stash`] patch
//! mode, then writes blended blob content and updated modes into the index.

use anyhow::{bail, Context, Result};
use grit_lib::crlf::{self, ConvertToGitOpts};
use grit_lib::diff::{diff_index_to_worktree, mode_from_metadata, DiffStatus};
use grit_lib::index::{Index, IndexEntry, MODE_TREE};
use grit_lib::merge_file::is_binary;
use grit_lib::objects::{ObjectId, ObjectKind};
use grit_lib::odb::Odb;
use grit_lib::repo::Repository;
use similar::{Algorithm, TextDiff};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::commands::add::{resolved_env_index_path, AddConfig};
use crate::commands::checkout::{patch_path_filter_matches, resolve_pathspec};
use crate::commands::stash::{partial_unified_for_op_range, split_hunk_at_first_gap};
use grit_lib::index::entry_from_metadata;

/// Blend index and worktree bytes for **staging** (`git add -p`).
///
/// [`checkout::blend_line_diff_by_hunk_ranges`] uses `accepted` with **revert/checkout** semantics
/// (accepted ⇒ keep the index/source side). For `add -p`, user `y` means take the **worktree**
/// side, so we invert the boolean vector.
fn blend_for_stage_hunks(
    index_bytes: &[u8],
    work_bytes: &[u8],
    ranges: &[(usize, usize)],
    stage_yes: &[bool],
) -> String {
    let revert_accepted: Vec<bool> = stage_yes.iter().map(|a| !*a).collect();
    crate::commands::checkout::blend_line_diff_by_hunk_ranges(
        index_bytes,
        work_bytes,
        ranges,
        &revert_accepted,
    )
}

/// Which prompt verb to use for the current file, mirroring Git's `prompt_mode_type`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum HunkKind {
    ModeChange,
    Deletion,
    Addition,
    Hunk,
}

/// Build the bracketed permitted-letter suffix used in the interactive prompt, matching
/// `add-patch.c`: navigation letters appear with multiple hunks, `,s` when the hunk can split,
/// `,e` unless the file is a deletion, and `,p,P` always.
fn prompt_suffix(n_hunks: usize, splittable: bool, is_deletion: bool) -> String {
    let mut s = String::new();
    if n_hunks > 1 {
        // ,k / ,K (previous), ,j / ,J (next), ,g,/ for goto/search.
        s.push_str(",k,K,j,J,g,/");
    }
    if splittable {
        s.push_str(",s");
    }
    if !is_deletion {
        s.push_str(",e");
    }
    s.push_str(",p,P");
    s
}

/// Per-hunk decision state mirroring Git's `UNDECIDED_HUNK`/`USE_HUNK`/`SKIP_HUNK`
/// (`add-patch.c`). The interactive hunk loop tracks one of these per hunk so navigation
/// (`j`/`k`/`J`/`K`/`g`/`/`) can find undecided hunks and `(was: y)`/`(was: n)` can annotate
/// already-decided ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Decision {
    Undecided,
    Use,
    Skip,
}

/// `dec_mod` from `add-patch.c`: `(value + max - 1) % max`.
fn dec_mod(value: usize, max: usize) -> usize {
    (value + max - 1) % max
}

/// `summarize_hunk` (`add-patch.c`): one-line summary used by the `g` goto-list, e.g.
/// `_-1,2_+1,3__________+15` (padded to width 20, then the first non-context line of the hunk,
/// truncated to width 80). `hunk_text` is the rendered `@@ ... @@`-headed hunk body.
fn summarize_hunk(hunk_text: &str) -> String {
    const SUMMARY_HEADER_WIDTH: usize = 20;
    const SUMMARY_LINE_WIDTH: usize = 80;
    let mut lines = hunk_text.lines();
    let header = lines.next().unwrap_or("");
    // Parse `@@ -o,c +o,c @@` into the ` -o,c +o,c ` summary prefix git emits.
    let mut out = String::new();
    if let Some(rest) = header.strip_prefix("@@ ") {
        if let Some(idx) = rest.find(" @@") {
            let ranges = &rest[..idx]; // e.g. "-1,2 +1,3"
            let mut parts = ranges.split_whitespace();
            let old = parts.next().unwrap_or("-0,0").trim_start_matches('-');
            let new = parts.next().unwrap_or("+0,0").trim_start_matches('+');
            let (oo, oc) = split_range(old);
            let (no, nc) = split_range(new);
            out = format!(" -{oo},{oc} +{no},{nc} ");
        }
    }
    if out.len() < SUMMARY_HEADER_WIDTH {
        out.push_str(&" ".repeat(SUMMARY_HEADER_WIDTH - out.len()));
    }
    // First line that is not a context line (does not begin with a space).
    for line in lines {
        if !line.starts_with(' ') {
            out.push_str(line);
            break;
        }
    }
    if out.len() > SUMMARY_LINE_WIDTH {
        out.truncate(SUMMARY_LINE_WIDTH);
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Parse a unified-diff range `o,c` (or bare `o`, meaning count 1) into `(offset, count)` strings.
fn split_range(s: &str) -> (String, String) {
    match s.split_once(',') {
        Some((o, c)) => (o.to_string(), c.to_string()),
        None => (s.to_string(), "1".to_string()),
    }
}

const DISPLAY_HUNKS_LINES: usize = 20;

/// Render a single hunk (`@@ -o,c +o,c @@` header + body) for the op range `[start, end)`, using
/// **absolute** line offsets from `old_lines`/`new_lines`. Leading/trailing equal runs in the range
/// are capped to `context` lines of surrounding context, matching `git diff`'s hunk headers — which
/// the simpler [`partial_unified_for_op_range`] cannot because it rebuilds offsets from 1.
fn render_hunk_with_offsets(
    old_lines: &[&str],
    new_lines: &[&str],
    ops: &[similar::DiffOp],
    start: usize,
    end: usize,
    context: usize,
) -> String {
    use similar::DiffOp;
    // Collect body lines as (marker, text) and track absolute old/new starts and counts.
    let mut body: Vec<(char, String)> = Vec::new();
    let mut old_start: Option<usize> = None; // 0-based
    let mut new_start: Option<usize> = None;
    let mut old_count = 0usize;
    let mut new_count = 0usize;

    let range = &ops[start..end];
    let last = range.len();
    for (k, op) in range.iter().enumerate() {
        match *op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => {
                // Determine how many context lines to keep: up to `context` at the leading edge
                // (k == 0) and trailing edge (k == last-1); interior equal runs are kept whole.
                let lead = k == 0;
                let trail = k == last - 1;
                let (skip_front, take) = if lead && trail {
                    // Whole-range equal (shouldn't happen for a change hunk): keep all.
                    (0, len)
                } else if lead {
                    let keep = context.min(len);
                    (len - keep, keep)
                } else if trail {
                    (0, context.min(len))
                } else {
                    (0, len)
                };
                for j in skip_front..(skip_front + take) {
                    if old_start.is_none() {
                        old_start = Some(old_index + j);
                        new_start = Some(new_index + j);
                    }
                    body.push((' ', old_lines[old_index + j].to_string()));
                    old_count += 1;
                    new_count += 1;
                }
            }
            DiffOp::Delete {
                old_index,
                old_len,
                new_index,
            } => {
                for j in 0..old_len {
                    if old_start.is_none() {
                        old_start = Some(old_index + j);
                        new_start = Some(new_index);
                    }
                    body.push(('-', old_lines[old_index + j].to_string()));
                    old_count += 1;
                }
            }
            DiffOp::Insert {
                old_index,
                new_index,
                new_len,
            } => {
                for j in 0..new_len {
                    if old_start.is_none() {
                        old_start = Some(old_index);
                        new_start = Some(new_index + j);
                    }
                    body.push(('+', new_lines[new_index + j].to_string()));
                    new_count += 1;
                }
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                for j in 0..old_len {
                    if old_start.is_none() {
                        old_start = Some(old_index + j);
                        new_start = Some(new_index);
                    }
                    body.push(('-', old_lines[old_index + j].to_string()));
                    old_count += 1;
                }
                for j in 0..new_len {
                    if old_start.is_none() {
                        old_start = Some(old_index);
                        new_start = Some(new_index + j);
                    }
                    body.push(('+', new_lines[new_index + j].to_string()));
                    new_count += 1;
                }
            }
        }
    }

    // Git: a zero-count side uses the bare 0-based position (`-0,0` / `+N,0`); a non-empty side
    // uses the 1-based first line.
    let o_off = match old_start {
        Some(s) if old_count > 0 => s + 1,
        Some(s) => s,
        None => 0,
    };
    let n_off = match new_start {
        Some(s) if new_count > 0 => s + 1,
        Some(s) => s,
        None => 0,
    };
    let o_hdr = if old_count == 1 {
        format!("{o_off}")
    } else {
        format!("{o_off},{old_count}")
    };
    let n_hdr = if new_count == 1 {
        format!("{n_off}")
    } else {
        format!("{n_off},{new_count}")
    };
    let mut s = format!("@@ -{o_hdr} +{n_hdr} @@\n");
    for (m, line) in body {
        s.push(m);
        s.push_str(&line);
        s.push('\n');
    }
    s
}

/// Compute the natural hunk ranges (`[start, end)` op-index spans) that `git diff -U<context>`
/// would emit: changes separated by an equal run longer than `2 * context` lines start a new
/// hunk; shorter gaps are absorbed. Each returned range still includes the full separating equal
/// runs (the renderer caps the shown context to `context`). With overlap on the boundary equal run
/// so each hunk renders its surrounding context, mirroring [`split_hunk_into_all`].
fn natural_hunk_ranges(ops: &[similar::DiffOp], context: usize) -> Vec<(usize, usize)> {
    let is_eq = |i: usize| matches!(ops.get(i), Some(similar::DiffOp::Equal { .. }));
    let eq_len = |i: usize| match ops.get(i) {
        Some(similar::DiffOp::Equal { len, .. }) => *len,
        _ => 0,
    };
    let n = ops.len();
    let change_idxs: Vec<usize> = (0..n).filter(|&i| !is_eq(i)).collect();
    if change_idxs.is_empty() {
        return vec![(0, n)];
    }

    let mut ranges: Vec<(usize, usize)> = Vec::new();
    // `group_first_change`/`prev_change` track the first/last change op of the current hunk.
    let mut group_first_change = change_idxs[0];
    let mut prev_change = change_idxs[0];
    for &c in &change_idxs[1..] {
        // Between two consecutive change ops there is at most one equal op; a big gap (> 2*context)
        // closes the current hunk and starts a new one. The separating equal op is shared so both
        // hunks render context around it.
        let big_gap = (prev_change + 1..c).any(|mid| eq_len(mid) > 2 * context);
        if big_gap {
            // Range start: equal op before the first change (leading context), else the change.
            let start = if group_first_change > 0 && is_eq(group_first_change - 1) {
                group_first_change - 1
            } else {
                group_first_change
            };
            // Range end: include the trailing equal op after the last change.
            let end = if is_eq(prev_change + 1) {
                prev_change + 2
            } else {
                prev_change + 1
            };
            ranges.push((start, end));
            group_first_change = c;
        }
        prev_change = c;
    }
    // Close the final hunk.
    let start = if group_first_change > 0 && is_eq(group_first_change - 1) {
        group_first_change - 1
    } else {
        group_first_change
    };
    let end = if is_eq(prev_change + 1) {
        prev_change + 2
    } else {
        prev_change + 1
    };
    ranges.push((start, end));
    ranges
}

/// Read a full answer line from stdin, returning `None` on EOF and `Some(trimmed)` otherwise
/// (only the trailing newline is stripped; interior/leading spaces are preserved so that
/// `g 1` and `/ pattern` round-trip). Matches `read_single_character`/`strbuf_getline`.
fn read_answer(reader: &mut impl BufRead) -> Result<Option<String>> {
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    let trimmed = line.trim_end_matches(['\n', '\r']).to_string();
    Ok(Some(trimmed))
}

/// Convert per-hunk decisions to the `accepted: Vec<bool>` the blender expects (USE ⇒ true).
fn decisions_to_accepted(decisions: &[Decision]) -> Vec<bool> {
    decisions.iter().map(|d| *d == Decision::Use).collect()
}

/// Index of the first still-undecided hunk, if any (git's `get_first_undecided`).
fn first_undecided(decisions: &[Decision]) -> Option<usize> {
    decisions.iter().position(|d| *d == Decision::Undecided)
}

/// Fully split a hunk into all of its `splittable_into` sub-hunks (git's `s` splits all the way,
/// not just the first gap). Returns true if at least one split happened.
///
/// Unlike [`split_hunk_at_first_gap`] (a hard op-range partition), the boundary equal run is
/// **shared** by the two adjacent sub-hunks so each one renders with the surrounding context git
/// shows (the `@@ ... @@` line and the trailing/leading context lines). Equal ops never consult
/// `accepted` during blending, so the overlap is harmless to staging.
fn split_hunk_into_all(
    ranges: &mut Vec<(usize, usize)>,
    hunk_index: usize,
    ops: &[similar::DiffOp],
) -> bool {
    if hunk_index >= ranges.len() {
        return false;
    }
    let (start, end) = ranges[hunk_index];
    let is_eq = |i: usize| matches!(ops.get(i), Some(similar::DiffOp::Equal { .. }));

    // Find the boundaries of each maximal change-run, then split at the middle of the equal runs
    // that separate them. `boundaries[k]` is `(eq_run_start, eq_run_end)` for the k-th internal gap.
    let mut sub: Vec<(usize, usize)> = Vec::new();
    let mut i = start;
    // Leading context.
    while i < end && is_eq(i) {
        i += 1;
    }
    let mut seg_start = start;
    while i < end {
        // Consume a change run.
        while i < end && !is_eq(i) {
            i += 1;
        }
        // Consume the following equal run.
        let eq_start = i;
        while i < end && is_eq(i) {
            i += 1;
        }
        if eq_start < i && i < end {
            // Internal gap: end this sub-hunk after the equal run, start the next at its start so
            // the equal run is shared as trailing/leading context.
            sub.push((seg_start, i));
            seg_start = eq_start;
        }
    }
    sub.push((seg_start, end));

    if sub.len() < 2 {
        return false;
    }
    ranges.splice(hunk_index..=hunk_index, sub);
    true
}

/// Render the `g`-command hunk list (git's `display_hunks`): up to `DISPLAY_HUNKS_LINES` lines
/// starting at `start`, each `%c%2d: <summary>`. Returns the index one past the last shown.
fn display_hunk_list(
    out: &mut impl Write,
    ranges: &[(usize, usize)],
    decisions: &[Decision],
    work: &[u8],
    start: usize,
    render_hunk: &dyn Fn(usize, &[(usize, usize)], &[u8]) -> String,
) -> usize {
    let end = (start + DISPLAY_HUNKS_LINES).min(ranges.len());
    for (i, dec) in decisions.iter().enumerate().take(end).skip(start) {
        let mark = match dec {
            Decision::Use => '+',
            Decision::Skip => '-',
            Decision::Undecided => ' ',
        };
        let text = render_hunk(i, ranges, work);
        let summary = summarize_hunk(&text);
        write!(out, "{mark}{:>2}: {summary}", i + 1).ok();
    }
    end
}

/// `?` help during the hunk loop. Git prints the always-available lines, then only the remainder
/// lines whose command character is present in the current `nav` suffix.
fn write_patch_help(out: &mut impl Write, nav: &str) {
    let base = "y - stage this hunk\n\
                n - do not stage this hunk\n\
                q - quit; do not stage this hunk or any of the remaining ones\n\
                a - stage this hunk and all later hunks in the file\n\
                d - do not stage this hunk or any of the later hunks in the file\n";
    write!(out, "{base}").ok();
    // Remainder lines, each gated on its command character appearing in `nav`.
    let remainder: &[(char, &str)] = &[
        (
            'k',
            "k - leave this hunk undecided, see previous undecided hunk",
        ),
        ('K', "K - leave this hunk undecided, see previous hunk"),
        (
            'j',
            "j - leave this hunk undecided, see next undecided hunk",
        ),
        ('J', "J - leave this hunk undecided, see next hunk"),
        ('g', "g - select a hunk to go to"),
        ('/', "/ - search for a hunk matching the given regex"),
        ('s', "s - split the current hunk into smaller hunks"),
        ('e', "e - manually edit the current hunk"),
        ('p', "p - print the current hunk"),
    ];
    for (ch, line) in remainder {
        if nav.contains(*ch) {
            writeln!(out, "{line}").ok();
        }
    }
    writeln!(out, "? - print help").ok();
}

/// 7-character abbreviated blob OID for `data` (Git's default short hash in patch headers).
fn short_oid_of(odb: &Odb, data: &[u8]) -> String {
    let _ = odb;
    let oid = Odb::hash_object_data(ObjectKind::Blob, data);
    oid.to_hex().chars().take(7).collect()
}

/// Number of sub-hunks the op range `start..end` would split into (gap-based, matching
/// [`split_hunk_at_first_gap`]): one more than the count of internal equal-runs flanked by changes.
fn splittable_into(ops: &[similar::DiffOp], start: usize, end: usize) -> usize {
    let is_eq = |i: usize| matches!(ops.get(i), Some(similar::DiffOp::Equal { .. }));
    let mut count = 1usize;
    let mut i = start;
    // Skip leading context.
    while i < end && is_eq(i) {
        i += 1;
    }
    while i < end {
        // Consume a run of changes.
        while i < end && !is_eq(i) {
            i += 1;
        }
        // Consume the following equal run; if more changes follow, this is a split point.
        let eq_start = i;
        while i < end && is_eq(i) {
            i += 1;
        }
        if eq_start < i && i < end {
            count += 1;
        }
    }
    count
}

/// Tunables for `git add -p` that come from `-U`/`--inter-hunk-context`/`--no-auto-advance`
/// (or the corresponding `diff.*` config). Resolved in [`crate::commands::add`].
pub(crate) struct PatchOptions {
    /// Number of context lines around each hunk (default 3).
    pub context: usize,
    /// Context lines kept between otherwise-adjacent hunks (default 0).
    pub inter_hunk_context: usize,
    /// Whether to auto-advance to the next hunk after a decision (default true).
    pub auto_advance: bool,
}

impl Default for PatchOptions {
    fn default() -> Self {
        Self {
            context: 3,
            inter_hunk_context: 0,
            auto_advance: true,
        }
    }
}

/// Run `git add -p` / `git add --patch`.
pub(crate) fn run_add_patch(
    repo: &Repository,
    pathspecs: &[String],
    add_cfg: &AddConfig,
    opts: &PatchOptions,
) -> Result<()> {
    run_add_patch_with_reader(repo, pathspecs, add_cfg, opts, None)
}

/// Like [`run_add_patch`] but lets a caller (e.g. `add -i`'s patch sub-command) thread its own
/// already-buffered stdin reader through, so input is not lost between the two BufReaders.
///
/// # Errors
/// Propagates I/O, ODB, and index errors.
pub(crate) fn run_add_patch_with_reader(
    repo: &Repository,
    pathspecs: &[String],
    add_cfg: &AddConfig,
    opts: &PatchOptions,
    external_reader: Option<&mut dyn BufRead>,
) -> Result<()> {
    let _ = opts.inter_hunk_context;
    let auto_advance = opts.auto_advance;
    let context = opts.context;
    let work_tree = repo
        .work_tree
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("this operation must be run in a work tree"))?;

    let cwd = std::env::current_dir().context("resolving cwd")?;
    let filter_paths: Vec<String> = pathspecs
        .iter()
        .map(|p| resolve_pathspec(p, work_tree, &cwd))
        .collect();

    let index_path = resolved_env_index_path(repo);
    let raw_index = Index::load(&index_path).unwrap_or_else(|_| Index::new());
    let mut index = repo.load_index_at(&index_path).context("loading index")?;

    let mut entries = diff_index_to_worktree(&repo.odb, &index, work_tree, false, false)?;
    entries.retain(|e| {
        if e.status == DiffStatus::Unmerged {
            return false;
        }
        patch_path_filter_matches(e.path(), &filter_paths)
    });
    entries.sort_by(|a, b| a.path().cmp(b.path()));

    if entries
        .iter()
        .any(|entry| path_under_sparse_index_dir(&raw_index, entry.path()))
    {
        emit_index_trace_region("ensure_full_index");
    }

    if entries.is_empty() {
        println!("No changes.");
        return Ok(());
    }

    let stdin = io::stdin();
    let mut owned_reader;
    let mut reader: &mut dyn BufRead = match external_reader {
        Some(r) => r,
        None => {
            owned_reader = stdin.lock();
            &mut owned_reader
        }
    };
    let mut out = io::stdout();

    let odb = &repo.odb;
    let conv = &add_cfg.conv;
    let attrs = &add_cfg.attrs;

    // Track how many candidate files turned out to be binary; if every one did, Git prints
    // "Only binary files changed." (add-patch.c) instead of silently doing nothing.
    let total_entries = entries.len();
    let mut binary_count = 0usize;

    for entry in entries {
        let path_str = entry.path().to_owned();
        let path_bytes = path_str.as_bytes();

        let Some(ie) = index.get(path_bytes, 0).cloned() else {
            continue;
        };

        if ie.mode == 0o160000 {
            continue;
        }

        let abs_path = work_tree.join(&path_str);
        let meta = match fs::symlink_metadata(&abs_path) {
            Ok(m) => m,
            Err(e)
                if e.kind() == std::io::ErrorKind::NotFound
                    || e.raw_os_error() == Some(20) /* ENOTDIR */ =>
            {
                if entry.status != DiffStatus::Deleted {
                    continue;
                }
                handle_deleted_file(
                    repo,
                    &mut index,
                    index_path.as_path(),
                    &path_str,
                    &ie,
                    &mut reader,
                    &mut out,
                    odb,
                )?;
                continue;
            }
            Err(_) => continue,
        };

        let file_attrs = crlf::get_file_attrs(attrs, &path_str, false, &add_cfg.config);

        let index_blob = if ie.oid == ObjectId::zero() {
            Vec::new()
        } else {
            let obj = match odb.read(&ie.oid) {
                Ok(o) if o.kind == ObjectKind::Blob => o.data,
                _ => continue,
            };
            obj
        };

        let work_blob = if meta.file_type().is_symlink() {
            let target = fs::read_link(&abs_path)?;
            target.to_string_lossy().into_owned().into_bytes()
        } else {
            let raw = fs::read(&abs_path).unwrap_or_default();
            let prior_blob = if ie.oid != ObjectId::zero() {
                Some(index_blob.clone())
            } else {
                None
            };
            let opts = ConvertToGitOpts {
                index_blob: prior_blob.as_deref(),
                renormalize: false,
                check_safecrlf: true,
            };
            match crlf::convert_to_git_with_opts(&raw, &path_str, conv, &file_attrs, opts) {
                Ok(c) => c,
                Err(msg) => {
                    eprintln!("{msg}");
                    continue;
                }
            }
        };

        if is_binary(&index_blob) || is_binary(&work_blob) {
            binary_count += 1;
            continue;
        }

        // An intent-to-add path (or a `DiffStatus::Added` entry) is rendered as a *new file*: the
        // index side is empty and the prompt verb is "Stage addition", with no mode-change prompt.
        let is_addition = entry.status == DiffStatus::Added || ie.intent_to_add();
        // A whole-file deletion (the worktree path is gone): Git renders this as `Stage deletion`
        // with no split (`s`) or edit (`e`) option, and the file header shows `deleted file mode`.
        let is_deletion = entry.status == DiffStatus::Deleted;
        let mode_differs =
            !is_addition && parse_mode_u32(&entry.old_mode) != parse_mode_u32(&entry.new_mode);
        let content_differs = index_blob != work_blob;

        let mut effective_mode = ie.mode;
        let index_side_bytes = index_blob.clone();

        if mode_differs {
            write!(out, "(1/1) Stage mode change [y,n,q,a,d,e,p,P,?]? ").ok();
            out.flush().ok();
            match read_one_command(&mut reader, &mut out)? {
                ReadCmd::Eof => {
                    repo.write_index_at(&index_path, &mut index)?;
                    return Ok(());
                }
                ReadCmd::Invalid => {}
                ReadCmd::Char { lower, .. } => match lower {
                    'y' => effective_mode = mode_from_metadata(&meta),
                    'q' => {
                        repo.write_index_at(&index_path, &mut index)?;
                        return Ok(());
                    }
                    _ => {}
                },
            }
        }

        if !content_differs {
            if mode_differs && effective_mode != ie.mode {
                write_index_blob_and_mode(
                    odb,
                    &mut index,
                    &path_str,
                    &abs_path,
                    &index_side_bytes,
                    effective_mode,
                )?;
            }
            continue;
        }

        let mut cur_work = work_blob;

        'rediff: loop {
            let index_str = String::from_utf8_lossy(&index_side_bytes);
            let work_str = String::from_utf8_lossy(&cur_work);
            let text_diff = TextDiff::configure()
                .algorithm(Algorithm::Myers)
                .diff_lines(index_str.as_ref(), work_str.as_ref());
            let ops: Vec<_> = text_diff.ops().to_vec();
            let has_change = ops
                .iter()
                .any(|o| !matches!(o, similar::DiffOp::Equal { .. }));
            if !has_change {
                if mode_differs && effective_mode != ie.mode {
                    write_index_blob_and_mode(
                        odb,
                        &mut index,
                        &path_str,
                        &abs_path,
                        &index_side_bytes,
                        effective_mode,
                    )?;
                }
                break 'rediff;
            }

            let n_ops = ops.len();
            // Additions/deletions are a single whole-file hunk; otherwise split into the natural
            // hunks `git diff -U<context>` would produce so navigation (j/k/J/K/g//) works without
            // requiring `s` first.
            let mut hunk_ranges: Vec<(usize, usize)> = if is_addition || is_deletion {
                vec![(0, n_ops)]
            } else {
                natural_hunk_ranges(&ops, context)
            };
            let mut decisions = vec![Decision::Undecided; hunk_ranges.len()];
            let mut hunk_index = 0usize;
            // -1 means "nothing rendered yet"; the hunk body is only re-printed when the cursor
            // lands on a different hunk (matching `rendered_hunk_index` in `add-patch.c`). Split
            // resets this to force a re-render of the now-current hunk.
            let mut rendered_hunk_index: isize = -1;

            // Render the file diff header once per file (git: `render_diff_header`).
            writeln!(out, "diff --git a/{path_str} b/{path_str}").ok();
            if is_addition {
                let short = short_oid_of(odb, &cur_work);
                let new_mode = mode_from_metadata(&meta);
                writeln!(out, "new file mode {new_mode:06o}").ok();
                writeln!(out, "index 0000000..{short}").ok();
                write!(out, "--- /dev/null\n+++ b/{path_str}\n").ok();
            } else if is_deletion {
                let short = short_oid_of(odb, &index_side_bytes);
                writeln!(out, "deleted file mode {:06o}", ie.mode).ok();
                writeln!(out, "index {short}..0000000").ok();
                write!(out, "--- a/{path_str}\n+++ /dev/null\n").ok();
            } else {
                write!(out, "--- a/{path_str}\n+++ b/{path_str}\n").ok();
            }

            // Render the hunk body text for hunk `i` (header `@@ ... @@` + body lines) with the
            // absolute line offsets `git diff` shows. `work` may change after an `e` edit, which
            // re-diffs, so the ops/lines for that path are recomputed by the caller on `rediff`.
            let index_side_str = String::from_utf8_lossy(&index_side_bytes).into_owned();
            let old_lines: Vec<&str> = index_side_str.lines().collect();
            let render_hunk = |i: usize, ranges: &[(usize, usize)], work: &[u8]| -> String {
                let (s, e) = ranges[i];
                let work_str = String::from_utf8_lossy(work).into_owned();
                let new_lines: Vec<&str> = work_str.lines().collect();
                render_hunk_with_offsets(&old_lines, &new_lines, &ops, s, e, context)
            };

            'hunk_loop: loop {
                let n_hunks = hunk_ranges.len();
                if hunk_index >= n_hunks {
                    hunk_index = 0;
                }

                // Find the nearest undecided hunk before/after the cursor (cyclic), git's
                // `undecided_previous`/`undecided_next`.
                let mut undecided_previous: Option<usize> = None;
                let mut undecided_next: Option<usize> = None;
                if n_hunks > 0 {
                    let mut i = dec_mod(hunk_index, n_hunks);
                    while i != hunk_index {
                        if decisions[i] == Decision::Undecided {
                            undecided_previous = Some(i);
                            break;
                        }
                        i = dec_mod(i, n_hunks);
                    }
                    let mut i = (hunk_index + 1) % n_hunks;
                    while i != hunk_index {
                        if decisions[i] == Decision::Undecided {
                            undecided_next = Some(i);
                            break;
                        }
                        i = (i + 1) % n_hunks;
                    }
                }

                // Everything decided? Without auto-advance we stop advancing; with it we move on.
                if undecided_previous.is_none()
                    && undecided_next.is_none()
                    && decisions[hunk_index] != Decision::Undecided
                    && auto_advance
                {
                    break 'hunk_loop;
                }

                if rendered_hunk_index != hunk_index as isize {
                    write!(out, "{}", render_hunk(hunk_index, &hunk_ranges, &cur_work)).ok();
                    rendered_hunk_index = hunk_index as isize;
                }

                let (s, e) = hunk_ranges[hunk_index];
                let display_idx = hunk_index + 1;

                // Build the navigation suffix exactly as git does (order-sensitive).
                let mut nav = String::new();
                let allow_prev_undecided = undecided_previous.is_some();
                let allow_prev = n_hunks > 1;
                let allow_next_undecided = undecided_next.is_some();
                let allow_next = n_hunks > 1;
                let allow_goto = n_hunks > 1;
                if allow_prev_undecided {
                    nav.push_str(",k");
                }
                if allow_prev {
                    nav.push_str(",K");
                }
                if allow_next_undecided {
                    nav.push_str(",j");
                }
                if allow_next {
                    nav.push_str(",J");
                }
                if allow_goto {
                    nav.push_str(",g,/");
                }
                let splittable = !is_deletion && splittable_into(&ops, s, e) > 1;
                if splittable {
                    nav.push_str(",s");
                }
                let allow_edit = !is_deletion && !(mode_differs && hunk_index == 0);
                if allow_edit {
                    nav.push_str(",e");
                }
                nav.push_str(",p,P");

                let kind = if is_deletion {
                    HunkKind::Deletion
                } else if is_addition {
                    HunkKind::Addition
                } else if mode_differs && hunk_index == 0 {
                    HunkKind::ModeChange
                } else {
                    HunkKind::Hunk
                };
                let verb = match kind {
                    HunkKind::ModeChange => "Stage mode change",
                    HunkKind::Deletion => "Stage deletion",
                    HunkKind::Addition => "Stage addition",
                    HunkKind::Hunk => "Stage this hunk",
                };
                let was = match decisions[hunk_index] {
                    Decision::Use => " (was: y)",
                    Decision::Skip => " (was: n)",
                    Decision::Undecided => "",
                };
                write!(
                    out,
                    "({display_idx}/{n_hunks}) {verb}{was} [y,n,q,a,d{nav},?]? "
                )
                .ok();
                out.flush().ok();

                // `soft_increment`: after y/n/e move to the next undecided hunk, or off the end.
                let soft_increment = |dec_next: Option<usize>| dec_next.unwrap_or(n_hunks);

                let answer = match read_answer(&mut reader)? {
                    None => {
                        // EOF: git prints a trailing newline and applies decided hunks so far.
                        writeln!(out).ok();
                        let accepted = decisions_to_accepted(&decisions);
                        let blended = blend_for_stage_hunks(
                            &index_side_bytes,
                            &cur_work,
                            &hunk_ranges,
                            &accepted,
                        );
                        write_index_blob_and_mode(
                            odb,
                            &mut index,
                            &path_str,
                            &abs_path,
                            blended.as_bytes(),
                            effective_mode,
                        )?;
                        repo.write_index_at(&index_path, &mut index)?;
                        return Ok(());
                    }
                    Some(a) => a,
                };

                if answer.is_empty() {
                    continue 'hunk_loop;
                }
                let first = answer.chars().next().unwrap();
                let lower = first.to_ascii_lowercase();

                // 'g' takes a hunk number and '/' takes a regexp, so they may be multi-char.
                if answer.chars().count() != 1 && lower != 'g' && first != '/' {
                    writeln!(out, "Only one letter is expected, got '{answer}'").ok();
                    continue 'hunk_loop;
                }

                match lower {
                    'y' => {
                        decisions[hunk_index] = Decision::Use;
                        hunk_index = soft_increment(undecided_next);
                    }
                    'n' => {
                        decisions[hunk_index] = Decision::Skip;
                        hunk_index = soft_increment(undecided_next);
                    }
                    'a' => {
                        for d in decisions.iter_mut().skip(hunk_index) {
                            if *d == Decision::Undecided {
                                *d = Decision::Use;
                            }
                        }
                        hunk_index = first_undecided(&decisions).unwrap_or(0);
                    }
                    'd' => {
                        for d in decisions.iter_mut().skip(hunk_index) {
                            if *d == Decision::Undecided {
                                *d = Decision::Skip;
                            }
                        }
                        hunk_index = first_undecided(&decisions).unwrap_or(0);
                    }
                    'q' => {
                        // Git: `q` sets `patch_update_resp = file_diff_nr` and breaks, then
                        // `putchar('\n')` and applies the decided hunks for this file before
                        // stopping all further files.
                        writeln!(out).ok();
                        let accepted = decisions_to_accepted(&decisions);
                        let blended = blend_for_stage_hunks(
                            &index_side_bytes,
                            &cur_work,
                            &hunk_ranges,
                            &accepted,
                        );
                        write_index_blob_and_mode(
                            odb,
                            &mut index,
                            &path_str,
                            &abs_path,
                            blended.as_bytes(),
                            effective_mode,
                        )?;
                        repo.write_index_at(&index_path, &mut index)?;
                        return Ok(());
                    }
                    _ if first == 'K' => {
                        if allow_prev {
                            hunk_index = dec_mod(hunk_index, n_hunks);
                        } else {
                            writeln!(out, "No other hunk").ok();
                        }
                    }
                    _ if first == 'J' => {
                        if allow_next {
                            hunk_index += 1;
                        } else {
                            writeln!(out, "No other hunk").ok();
                        }
                    }
                    _ if first == 'k' => {
                        if let Some(p) = undecided_previous {
                            hunk_index = p;
                        } else {
                            writeln!(out, "No other undecided hunk").ok();
                        }
                    }
                    _ if first == 'j' => {
                        if let Some(n) = undecided_next {
                            hunk_index = n;
                        } else {
                            writeln!(out, "No other undecided hunk").ok();
                        }
                    }
                    'g' => {
                        if !allow_goto {
                            writeln!(out, "No other hunks to goto").ok();
                            continue 'hunk_loop;
                        }
                        // Strip the leading 'g' and trim.
                        let mut arg: String = answer
                            .chars()
                            .skip(1)
                            .collect::<String>()
                            .trim()
                            .to_string();
                        // Show the hunk list until the user provides a target.
                        let mut start = hunk_index as isize - (DISPLAY_HUNKS_LINES as isize) / 2;
                        if start < 0 {
                            start = 0;
                        }
                        let mut start = start as usize;
                        while arg.is_empty() {
                            let end = display_hunk_list(
                                &mut out,
                                &hunk_ranges,
                                &decisions,
                                &cur_work,
                                start,
                                &render_hunk,
                            );
                            if end < n_hunks {
                                write!(out, "go to which hunk (<ret> to see more)? ").ok();
                            } else {
                                write!(out, "go to which hunk? ").ok();
                            }
                            out.flush().ok();
                            start = end;
                            match read_answer(&mut reader)? {
                                None => {
                                    writeln!(out).ok();
                                    let accepted = decisions_to_accepted(&decisions);
                                    let blended = blend_for_stage_hunks(
                                        &index_side_bytes,
                                        &cur_work,
                                        &hunk_ranges,
                                        &accepted,
                                    );
                                    write_index_blob_and_mode(
                                        odb,
                                        &mut index,
                                        &path_str,
                                        &abs_path,
                                        blended.as_bytes(),
                                        effective_mode,
                                    )?;
                                    repo.write_index_at(&index_path, &mut index)?;
                                    return Ok(());
                                }
                                Some(a) => arg = a.trim().to_string(),
                            }
                        }
                        match arg.parse::<usize>() {
                            Ok(n) if n >= 1 && n <= n_hunks => hunk_index = n - 1,
                            Ok(_) => {
                                if n_hunks == 1 {
                                    writeln!(out, "Sorry, only 1 hunk available.").ok();
                                } else {
                                    writeln!(out, "Sorry, only {n_hunks} hunks available.").ok();
                                }
                            }
                            Err(_) => {
                                writeln!(out, "Invalid number: '{arg}'").ok();
                            }
                        }
                    }
                    _ if first == '/' => {
                        if !allow_goto {
                            writeln!(out, "No other hunks to search").ok();
                            continue 'hunk_loop;
                        }
                        let mut pat: String = answer.chars().skip(1).collect::<String>();
                        pat = pat.trim_end_matches(['\n', '\r']).to_string();
                        if pat.is_empty() {
                            write!(out, "search for regex? ").ok();
                            out.flush().ok();
                            match read_answer(&mut reader)? {
                                None => {
                                    writeln!(out).ok();
                                    let accepted = decisions_to_accepted(&decisions);
                                    let blended = blend_for_stage_hunks(
                                        &index_side_bytes,
                                        &cur_work,
                                        &hunk_ranges,
                                        &accepted,
                                    );
                                    write_index_blob_and_mode(
                                        odb,
                                        &mut index,
                                        &path_str,
                                        &abs_path,
                                        blended.as_bytes(),
                                        effective_mode,
                                    )?;
                                    repo.write_index_at(&index_path, &mut index)?;
                                    return Ok(());
                                }
                                Some(a) => pat = a.trim_end_matches(['\n', '\r']).to_string(),
                            }
                            if pat.is_empty() {
                                continue 'hunk_loop;
                            }
                        }
                        match regex::Regex::new(&pat) {
                            Ok(re) => {
                                let mut i = hunk_index;
                                let mut found = false;
                                loop {
                                    let text = render_hunk(i, &hunk_ranges, &cur_work);
                                    if re.is_match(&text) {
                                        found = true;
                                        break;
                                    }
                                    i = (i + 1) % n_hunks;
                                    if i == hunk_index {
                                        break;
                                    }
                                }
                                if found {
                                    hunk_index = i;
                                } else {
                                    writeln!(out, "No hunk matches the given pattern").ok();
                                }
                            }
                            Err(_) => {
                                writeln!(out, "Malformed search regexp {pat}").ok();
                            }
                        }
                    }
                    's' => {
                        if !splittable {
                            writeln!(out, "Sorry, cannot split this hunk").ok();
                        } else {
                            let before = hunk_ranges.len();
                            if split_hunk_into_all(&mut hunk_ranges, hunk_index, &ops) {
                                let added = hunk_ranges.len() - before;
                                // The new sub-hunks (and the original slot) are all undecided.
                                for _ in 0..added {
                                    decisions.insert(hunk_index + 1, Decision::Undecided);
                                }
                                decisions[hunk_index] = Decision::Undecided;
                                writeln!(out, "Split into {} hunks.", added + 1).ok();
                                rendered_hunk_index = -1;
                            }
                        }
                    }
                    'e' => {
                        if !allow_edit {
                            writeln!(out, "Sorry, cannot edit this hunk").ok();
                        } else {
                            match edit_hunk_and_apply(
                                &mut out,
                                path_str.as_str(),
                                &index_side_bytes,
                                &cur_work,
                                &ops[s..e],
                                context,
                            ) {
                                Ok(EditResult::Unchanged) => {
                                    // No-op edit: stage the hunk as-is (git: `hunk->use = USE_HUNK`).
                                    decisions[hunk_index] = Decision::Use;
                                    hunk_index = soft_increment(undecided_next);
                                }
                                Ok(EditResult::Edited(new_work)) => {
                                    cur_work = new_work;
                                    decisions[hunk_index] = Decision::Use;
                                    hunk_index = soft_increment(undecided_next);
                                }
                                Ok(EditResult::Aborted) | Err(_) => {}
                            }
                        }
                    }
                    'p' => {
                        // p/P just re-render the current hunk.
                        rendered_hunk_index = -1;
                    }
                    '?' => {
                        write_patch_help(&mut out, &nav);
                    }
                    ' ' => {}
                    _ => {
                        writeln!(out, "Unknown command '{answer}' (use '?' for help)").ok();
                    }
                }
            }

            // Git prints a trailing newline when leaving `patch_update_file`.
            writeln!(out).ok();

            let accepted = decisions_to_accepted(&decisions);
            let blended =
                blend_for_stage_hunks(&index_side_bytes, &cur_work, &hunk_ranges, &accepted);

            if accepted.iter().any(|&a| a) || (mode_differs && effective_mode != ie.mode) {
                write_index_blob_and_mode(
                    odb,
                    &mut index,
                    &path_str,
                    &abs_path,
                    blended.as_bytes(),
                    effective_mode,
                )?;
            }
            break 'rediff;
        }
    }

    // Mirror Git: if every candidate file was binary (and thus skipped), say so.
    if total_entries > 0 && binary_count == total_entries {
        println!("Only binary files changed.");
    }

    repo.write_index_at(&index_path, &mut index)
        .context("writing index")?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadCmd {
    Eof,
    Invalid,
    /// A single-character command. `lower` is folded for matching; `raw` keeps the original case
    /// for the "Unknown command '<x>'" diagnostic.
    Char {
        lower: char,
        raw: char,
    },
}

impl ReadCmd {
    fn ch(lower: char, raw: char) -> Self {
        ReadCmd::Char { lower, raw }
    }
}

fn read_one_command(reader: &mut impl BufRead, out: &mut impl Write) -> Result<ReadCmd> {
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(ReadCmd::Eof);
    }
    let trimmed = line.trim_end_matches(['\n', '\r']);
    let t = trimmed.trim();
    if t.is_empty() {
        return Ok(ReadCmd::ch(' ', ' '));
    }
    if t.chars().count() > 1 {
        // Git: `err(s, _("Only one letter is expected, got '%s'"), ...)`.
        writeln!(out, "Only one letter is expected, got '{t}'")?;
        return Ok(ReadCmd::Invalid);
    }
    let c = t.chars().next().unwrap_or(' ');
    Ok(ReadCmd::ch(c.to_ascii_lowercase(), c))
}

fn parse_mode_u32(m: &str) -> u32 {
    u32::from_str_radix(m, 8).unwrap_or(0)
}

fn handle_deleted_file(
    repo: &Repository,
    index: &mut Index,
    index_path: &Path,
    path_str: &str,
    ie: &IndexEntry,
    reader: &mut impl BufRead,
    out: &mut impl Write,
    odb: &Odb,
) -> Result<()> {
    let index_blob = if ie.oid == ObjectId::zero() {
        Vec::new()
    } else {
        let obj = odb.read(&ie.oid)?;
        if obj.kind != ObjectKind::Blob {
            return Ok(());
        }
        obj.data
    };
    if is_binary(&index_blob) {
        return Ok(());
    }

    let work_blob = Vec::<u8>::new();
    let index_str = String::from_utf8_lossy(&index_blob);
    let work_str = String::from_utf8_lossy(&work_blob);
    let text_diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_lines(index_str.as_ref(), work_str.as_ref());
    let ops: Vec<_> = text_diff.ops().to_vec();
    let n_ops = ops.len();
    let mut hunk_ranges = vec![(0, n_ops)];
    let mut accepted = vec![false; 1];
    let mut hunk_cursor = 0usize;

    loop {
        if hunk_cursor >= hunk_ranges.len() {
            break;
        }
        let display_idx = hunk_cursor + 1;
        let n_hunks = hunk_ranges.len();
        let (s, e) = hunk_ranges[hunk_cursor];
        let hunk_only =
            partial_unified_for_op_range(path_str, &index_blob, &work_blob, &ops[s..e], 3, true);
        // Deletion file header: `deleted file mode`, `index <old>..0000000`, `+++ /dev/null`.
        let short = short_oid_of(odb, &index_blob);
        writeln!(out, "diff --git a/{path_str} b/{path_str}").ok();
        writeln!(out, "deleted file mode {:06o}", ie.mode).ok();
        writeln!(out, "index {short}..0000000").ok();
        write!(out, "--- a/{path_str}\n+++ /dev/null\n").ok();
        write!(out, "{hunk_only}").ok();
        // Deletions never offer edit (`e`); split (`s`) only when the hunk is actually splittable.
        let splittable = splittable_into(&ops, s, e) > 1;
        let suffix = prompt_suffix(n_hunks, splittable, true);
        write!(
            out,
            "({display_idx}/{n_hunks}) Stage deletion [y,n,q,a,d{suffix},?]? "
        )
        .ok();
        out.flush().ok();

        match read_one_command(reader, out)? {
            ReadCmd::Eof => {
                repo.write_index_at(index_path, index)?;
                return Ok(());
            }
            ReadCmd::Invalid => continue,
            ReadCmd::Char { lower, .. } => match lower {
                'y' => {
                    accepted[hunk_cursor] = true;
                    hunk_cursor += 1;
                }
                'n' => {
                    hunk_cursor += 1;
                }
                'a' => {
                    for j in hunk_cursor..n_hunks {
                        accepted[j] = true;
                    }
                    break;
                }
                'd' => break,
                'q' => {
                    repo.write_index_at(index_path, index)?;
                    return Ok(());
                }
                's' => {
                    if !split_hunk_at_first_gap(&mut hunk_ranges, hunk_cursor, &ops) {
                        writeln!(out, "Sorry, cannot split this hunk").ok();
                        continue;
                    }
                    let n = hunk_ranges.len();
                    accepted.resize(n, false);
                }
                '?' => {
                    writeln!(
                        out,
                        "y - stage this hunk for deletion\n\
                         n - do not stage this hunk\n\
                         q - quit\n\
                         a - stage this and all later hunks\n\
                         d - skip remaining hunks in this file\n\
                         s - split hunk\n"
                    )
                    .ok();
                }
                _ => {}
            },
        }
    }

    if accepted.iter().any(|&a| a) {
        let blended = blend_for_stage_hunks(&index_blob, &work_blob, &hunk_ranges, &accepted);
        if blended.is_empty() {
            index.remove(path_str.as_bytes());
        } else {
            let oid = odb.write(ObjectKind::Blob, blended.as_bytes())?;
            if let Some(ent) = index.get_mut(path_str.as_bytes(), 0) {
                ent.oid = oid;
                ent.size = blended.len() as u32;
            }
        }
    }
    Ok(())
}

fn write_index_blob_and_mode(
    odb: &Odb,
    index: &mut Index,
    path_str: &str,
    abs_path: &Path,
    blob_data: &[u8],
    mode: u32,
) -> Result<()> {
    let oid = odb.write(ObjectKind::Blob, blob_data)?;
    let meta = fs::symlink_metadata(abs_path).ok();
    // Whether the staged blob equals the current worktree bytes. When a partial hunk (or an edited
    // hunk) stages content that differs from the worktree, the index entry's stat must NOT claim to
    // match the worktree — otherwise `git diff` (diff-files) takes the stat fast-path and reports
    // the path clean even though the staged blob differs (t3701 "real edit works").
    let worktree_bytes = fs::read(abs_path).ok();
    let blob_matches_worktree = worktree_bytes.as_deref() == Some(blob_data);
    let mut new_ent = if let Some(m) = meta.as_ref() {
        let mut e = entry_from_metadata(m, path_str.as_bytes(), oid, mode);
        e.mode = mode;
        if !blob_matches_worktree {
            // Record the blob's true size and drop the worktree mtime so diff-files re-hashes and
            // sees the difference (Git leaves such entries stat-dirty).
            e.size = blob_data.len() as u32;
            e.mtime_sec = 0;
            e.mtime_nsec = 0;
            e.ctime_sec = 0;
            e.ctime_nsec = 0;
        }
        e
    } else {
        IndexEntry {
            ctime_sec: 0,
            ctime_nsec: 0,
            mtime_sec: 0,
            mtime_nsec: 0,
            dev: 0,
            ino: 0,
            mode,
            uid: 0,
            gid: 0,
            size: blob_data.len() as u32,
            oid,
            flags: path_str.len().min(0xFFF) as u16,
            flags_extended: None,
            path: path_str.as_bytes().to_vec(),
            base_index_pos: 0,
        }
    };
    new_ent.set_intent_to_add(false);
    new_ent.set_assume_unchanged(false);
    new_ent.set_skip_worktree(false);
    index.stage_file(new_ent);
    Ok(())
}

fn path_under_sparse_index_dir(index: &Index, path: &str) -> bool {
    let path = path.trim_end_matches('/');
    index
        .entries
        .iter()
        .filter(|entry| entry.stage() == 0 && entry.mode == MODE_TREE)
        .filter_map(|entry| std::str::from_utf8(&entry.path).ok())
        .map(|prefix| prefix.trim_end_matches('/'))
        .any(|prefix| {
            let prefix_slash = format!("{prefix}/");
            path == prefix || path.starts_with(&prefix_slash)
        })
}

fn emit_index_trace_region(label: &str) {
    if let Ok(trace2_event) = std::env::var("GIT_TRACE2_EVENT") {
        if !trace2_event.trim().is_empty() {
            let _ = crate::trace2_region_json(&trace2_event, "index", label);
        }
    }
}

/// Open `content` in the user's editor (`GIT_EDITOR`/`VISUAL`/`EDITOR`), returning the edited bytes.
fn run_editor_on_text(content: &[u8]) -> Result<Vec<u8>> {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::new().context("temp file for add -p edit")?;
    f.as_file_mut().write_all(content)?;
    f.flush()?;
    let path = f.path().to_owned();
    let editor = std::env::var("GIT_EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("{editor} \"$1\""))
        .arg("sh")
        .arg(&path)
        .status()
        .context("running editor")?;
    if !status.success() {
        bail!("editor failed");
    }
    fs::read(&path).context("reading edited file")
}

/// Compute the inclusive index(old)-side line span `[old_start, old_end)` covered by `op_slice`.
fn index_span(op_slice: &[similar::DiffOp]) -> (usize, usize) {
    let mut start = usize::MAX;
    let mut end = 0usize;
    for op in op_slice {
        let (s, e) = match *op {
            similar::DiffOp::Equal { old_index, len, .. } => (old_index, old_index + len),
            similar::DiffOp::Delete {
                old_index, old_len, ..
            } => (old_index, old_index + old_len),
            similar::DiffOp::Insert { old_index, .. } => (old_index, old_index),
            similar::DiffOp::Replace {
                old_index, old_len, ..
            } => (old_index, old_index + old_len),
        };
        start = start.min(s);
        end = end.max(e);
    }
    if start == usize::MAX {
        (0, 0)
    } else {
        (start, end)
    }
}

/// Manually edit the current hunk (the `e` command), mirroring `edit_hunk_manually` +
/// `recount_edited_hunk` + apply-check in `add-patch.c`.
///
/// Renders the hunk body with a commented quick-guide, runs the editor, strips comment lines,
/// then applies the edited hunk to the index-side content at this hunk's location to produce the
/// new full worktree content. If the edited hunk's context/removed lines do not match the index
/// content, prints `error: patch failed` / `hunk does not apply` (matching `git apply`) and
/// returns `Ok(None)`.
///
/// # Returns
/// - `Ok(Some(new_work))` — the new worktree-side content after applying the edited hunk.
/// - `Ok(None)` — the edit was abandoned/empty or did not apply (hunk left unchanged).
///
/// # Errors
/// Propagates editor/IO failures.
/// Outcome of a manual hunk edit (`e`).
enum EditResult {
    /// The editor left the hunk unchanged: stage it as-is (`hunk->use = USE_HUNK`, no rediff).
    Unchanged,
    /// The hunk was edited: `cur_work` is replaced with the new full-file content.
    Edited(Vec<u8>),
    /// The edit was abandoned (all lines removed, or the patch did not apply): leave undecided.
    Aborted,
}

fn edit_hunk_and_apply(
    out: &mut impl Write,
    path: &str,
    index_bytes: &[u8],
    work_bytes: &[u8],
    op_slice: &[similar::DiffOp],
    context: usize,
) -> Result<EditResult> {
    // The body to present is the hunk text (header + ` `/`+`/`-` lines), as displayed.
    let hunk_text =
        partial_unified_for_op_range(path, index_bytes, work_bytes, op_slice, context, true);

    // Comment guide, matching add-patch.c. Comment char defaults to '#'.
    let mut buf = String::new();
    buf.push_str("# Manual hunk edit mode -- see bottom for a quick guide.\n");
    buf.push_str(&hunk_text);
    buf.push_str("# ---\n");
    buf.push_str("# To remove '-' lines, make them ' ' lines (context).\n");
    buf.push_str("# To remove '+' lines, delete them.\n");
    buf.push_str("# Lines starting with # will be removed.\n");
    buf.push_str(
        "# If it does not apply cleanly, you will be given an opportunity to\n\
         # edit again.  If all lines of the hunk are removed, then the edit is\n\
         # aborted and the hunk is left unchanged.\n",
    );

    let edited = run_editor_on_text(buf.as_bytes())?;
    let edited = String::from_utf8_lossy(&edited).into_owned();

    // If the editor left the presented buffer unchanged (e.g. EDITOR=: / touch), git stages the
    // hunk as-is without re-diffing the file.
    if edited == buf {
        return Ok(EditResult::Unchanged);
    }

    // Strip comment lines.
    let body: Vec<&str> = edited.lines().filter(|l| !l.starts_with('#')).collect();

    // Drop the @@ header line(s); keep ` `/`+`/`-`/`\` body lines.
    let mut old_lines: Vec<String> = Vec::new();
    let mut new_lines: Vec<String> = Vec::new();
    let mut saw_body = false;
    for line in &body {
        if line.starts_with("@@") {
            saw_body = true;
            continue;
        }
        if !saw_body {
            // Lines before the header (shouldn't happen) are ignored.
            continue;
        }
        if line.starts_with('\\') {
            continue; // "\ No newline at end of file"
        }
        let (marker, rest) = match line.chars().next() {
            Some(c @ (' ' | '+' | '-')) => (c, &line[1..]),
            // A line with no leading marker is treated as context (Git strips a single space).
            _ => (' ', *line),
        };
        match marker {
            ' ' => {
                old_lines.push(rest.to_string());
                new_lines.push(rest.to_string());
            }
            '-' => old_lines.push(rest.to_string()),
            '+' => new_lines.push(rest.to_string()),
            _ => {}
        }
    }

    if old_lines.is_empty() && new_lines.is_empty() {
        // All lines removed: abandon the edit.
        return Ok(EditResult::Aborted);
    }

    // Apply positionally, like `git apply`: locate where the edited hunk's old side
    // (context + removed lines) matches a contiguous run of the index content, preferring the
    // original hunk position, then splice the new side (context + added) in its place.
    let (orig_old_start, _orig_old_end) = index_span(op_slice);
    let index_str = String::from_utf8_lossy(index_bytes);
    let index_lines: Vec<&str> = index_str.lines().collect();

    let match_at = locate_hunk(&index_lines, &old_lines, orig_old_start);
    let Some(pos) = match_at else {
        writeln!(out, "error: patch failed: {path}:{}", orig_old_start + 1).ok();
        writeln!(out, "error: {path}: patch does not apply").ok();
        writeln!(
            out,
            "Your edited hunk does not apply. Edit again (saying \"no\" discards!) [y/n]? "
        )
        .ok();
        return Ok(EditResult::Aborted);
    };

    let trailing_newline = work_bytes.ends_with(b"\n") || index_bytes.ends_with(b"\n");
    let mut result_lines: Vec<String> = Vec::new();
    result_lines.extend(index_lines[..pos].iter().map(|s| s.to_string()));
    result_lines.extend(new_lines.iter().cloned());
    result_lines.extend(
        index_lines[(pos + old_lines.len()).min(index_lines.len())..]
            .iter()
            .map(|s| s.to_string()),
    );

    let mut new_content = result_lines.join("\n");
    if trailing_newline && !new_content.is_empty() {
        new_content.push('\n');
    }
    Ok(EditResult::Edited(new_content.into_bytes()))
}

/// Find the line index in `haystack` where `needle` matches contiguously, preferring `hint` then
/// scanning outward (the position-then-fuzz search `git apply` performs). Returns `None` if no
/// match exists. An empty `needle` (pure insertion) matches at `hint` (clamped).
fn locate_hunk(haystack: &[&str], needle: &[String], hint: usize) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(hint.min(haystack.len()));
    }
    if n > haystack.len() {
        return None;
    }
    let matches_at = |p: usize| {
        haystack[p..p + n]
            .iter()
            .zip(needle)
            .all(|(a, b)| *a == b.as_str())
    };
    let last = haystack.len() - n;
    let start = hint.min(last);
    // Search forward then backward from the hint.
    for p in start..=last {
        if matches_at(p) {
            return Some(p);
        }
    }
    for p in (0..start).rev() {
        if matches_at(p) {
            return Some(p);
        }
    }
    None
}
