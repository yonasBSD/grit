//! `git status` as a structured operation.
//!
//! The library computes a [`StatusModel`] — every fact the user-facing output
//! needs, with **no presentation applied** — and the `grit` binary renders it
//! into porcelain v1/v2, short, or long format (applying colour, column layout,
//! path quoting, and the comment prefix). This is the reference example of the
//! library/CLI split described on [`crate::porcelain`].
//!
//! # Status of the extraction
//!
//! This module currently defines the data contract ([`StatusOptions`] in,
//! [`StatusModel`] out). The computation that produces the model is being moved
//! out of `grit/src/commands/status.rs::run` in stages; once it lands here as
//! [`status`], the three CLI formatters (`format_porcelain_v2`, `format_short`,
//! `format_long`) consume a `&StatusModel` instead of a dozen loose arguments.
//!
//! The model's shape is taken directly from the inputs those three formatters
//! share today: HEAD + its tree, the staged (index-vs-HEAD) and unstaged
//! (index-vs-worktree) diffs, the untracked and ignored path lists, the
//! in-progress operation [`state`](crate::state::WtStatusState), the loaded and
//! sparse-expanded index, and the stash count.

use crate::diff::DiffEntry;
use crate::index::Index;
use crate::objects::ObjectId;
use crate::state::{HeadState, WtStatusState};

/// How untracked files are reported (`git status --untracked-files=<mode>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UntrackedMode {
    /// `no` — do not list untracked files.
    No,
    /// `normal` — list untracked files and directories.
    Normal,
    /// `all` — list every individual untracked file, recursing into directories.
    All,
}

/// How ignored files are reported (`git status --ignored[=<mode>]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoredMode {
    /// Do not list ignored files (the default).
    No,
    /// `traditional` — list ignored files and directories.
    Traditional,
    /// `matching` — list only ignored paths that match an ignore pattern.
    Matching,
}

/// Rename/copy detection settings for the status diffs (`status.renames` /
/// `--find-renames`, `status.renameLimit`, copy detection).
#[derive(Debug, Clone, Copy)]
pub struct RenameDetection {
    /// Rename similarity threshold, as a percentage (e.g. `50` for 50%).
    pub threshold: u32,
    /// Whether to also detect copies (`-C` / `--find-copies`).
    pub copies: bool,
}

/// Inputs that drive **what** `status` computes.
///
/// The CLI translates clap arguments and config (`status.showUntrackedFiles`,
/// `status.renames`, `status.aheadBehind`, submodule ignore settings, …) into
/// this plain struct. Presentation choices — short vs. porcelain vs. long,
/// colour, column layout, path quoting, `-z` — are deliberately absent; they
/// belong to the renderer, not the computation.
#[derive(Debug, Clone)]
pub struct StatusOptions {
    /// Untracked-file reporting mode.
    pub untracked: UntrackedMode,
    /// Ignored-file reporting mode.
    pub ignored: IgnoredMode,
    /// Rename/copy detection, or `None` to skip rename detection.
    pub renames: Option<RenameDetection>,
    /// Limit the report to paths matching these pathspecs (empty = whole tree).
    pub pathspecs: Vec<String>,
    /// Compute ahead/behind counts relative to the upstream branch.
    pub ahead_behind: bool,
}

impl Default for StatusOptions {
    fn default() -> Self {
        Self {
            untracked: UntrackedMode::Normal,
            ignored: IgnoredMode::No,
            renames: None,
            pathspecs: Vec::new(),
            ahead_behind: true,
        }
    }
}

/// The computed result of `git status`: everything the renderers need, with no
/// presentation applied.
///
/// Fields mirror what the CLI's `format_porcelain_v2`, `format_short`, and
/// `format_long` read today, so a renderer can be a pure function of this model
/// plus the user's chosen output format.
#[derive(Debug, Clone)]
pub struct StatusModel {
    /// The resolved HEAD (branch, detached, or unborn).
    pub head: HeadState,
    /// Tree OID of the HEAD commit, or `None` on an unborn branch.
    pub head_tree: Option<ObjectId>,
    /// In-progress operation state (merge, rebase, cherry-pick, bisect, …).
    pub state: WtStatusState,
    /// The loaded index, with sparse-directory placeholders expanded — the same
    /// index the renderers query for per-stage entries.
    pub index: Index,
    /// Staged changes: the index-vs-HEAD-tree diff.
    pub staged: Vec<DiffEntry>,
    /// Unstaged changes: the index-vs-worktree diff.
    pub unstaged: Vec<DiffEntry>,
    /// Untracked paths (subject to [`StatusOptions::untracked`]).
    pub untracked: Vec<String>,
    /// Ignored paths (subject to [`StatusOptions::ignored`]).
    pub ignored: Vec<String>,
    /// Number of stash entries (for the optional stash footer / `--show-stash`).
    pub stash_count: usize,
    /// Whether the on-disk index used the sparse-directory format.
    pub index_sparse_on_disk: bool,
    /// Sparse-directory prefixes present in the on-disk index, if any.
    pub sparse_directory_prefixes: Vec<Vec<u8>>,
}
