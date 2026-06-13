//! `gs log` — the recent history reachable from HEAD.
//!
//! Deliberately minimal: it shows one page of commits (newest first) and, when
//! there's more, prints the command to fetch the next page.

use anyhow::{Context, Result};
use grit_lib::rev_list::{rev_list, RevListOptions};

use crate::context::{self, short_oid, subject_line};

/// How many commits to show per page.
const PAGE: usize = 10;

pub fn run(before: Option<String>) -> Result<()> {
    let repo = context::discover()?;
    let start = before.unwrap_or_else(|| "HEAD".to_owned());

    let opts = RevListOptions {
        // One extra so we know whether there's a next page.
        max_count: Some(PAGE + 1),
        ..Default::default()
    };
    let result = rev_list(&repo, &[start.clone()], &[], &opts)
        .with_context(|| format!("could not list commits from {start}"))?;

    if result.commits.is_empty() {
        println!("No commits yet.");
        return Ok(());
    }

    for oid in result.commits.iter().take(PAGE) {
        let commit = context::read_commit(&repo, oid)?;
        println!("{}  {}", short_oid(oid), subject_line(&commit.message));
    }

    if let Some(next) = result.commits.get(PAGE) {
        println!();
        println!("→ more: gs log --before={}", short_oid(next));
    }
    Ok(())
}
