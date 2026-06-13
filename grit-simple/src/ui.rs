//! Small output-formatting helpers shared by `gs` commands.

use grit_lib::diff::{DiffEntry, DiffStatus};

/// The path a diff entry refers to (prefers the new side, falls back to old).
pub fn entry_path(entry: &DiffEntry) -> &str {
    entry
        .new_path
        .as_deref()
        .or(entry.old_path.as_deref())
        .unwrap_or("?")
}

/// A single-character glyph summarizing a change.
fn glyph(status: &DiffStatus) -> char {
    match status {
        DiffStatus::Added => '+',
        DiffStatus::Deleted => '-',
        DiffStatus::Modified | DiffStatus::TypeChanged => '~',
        DiffStatus::Renamed | DiffStatus::Copied => '»',
        DiffStatus::Unmerged => '!',
    }
}

/// A short word describing a change, shown in a trailing note.
fn label(entry: &DiffEntry) -> &'static str {
    match entry.status {
        DiffStatus::Added => "new",
        DiffStatus::Deleted => "deleted",
        DiffStatus::Modified => "modified",
        DiffStatus::TypeChanged => "type changed",
        DiffStatus::Renamed => "renamed",
        DiffStatus::Copied => "copied",
        DiffStatus::Unmerged => "conflict",
    }
}

/// Print a titled group of diff entries (does nothing when empty).
pub fn print_change_group(title: &str, entries: &[DiffEntry]) {
    if entries.is_empty() {
        return;
    }
    println!("{title}");
    for entry in entries {
        println!(
            "  {}  {:<32}  {}",
            glyph(&entry.status),
            entry_path(entry),
            label(entry)
        );
    }
    println!();
}

/// Print the untracked-files group (does nothing when empty).
pub fn print_untracked(paths: &[String]) {
    if paths.is_empty() {
        return;
    }
    println!("Untracked");
    for path in paths {
        println!("  ?  {path}");
    }
    println!();
}
