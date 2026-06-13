//! `gs status` (and bare `gs`) — the dashboard: where you are, the commits
//! you're ahead by, what's changed, and what to do next.

use anyhow::{Context, Result};
use grit_lib::porcelain::status::{status, StatusModel, StatusOptions};
use grit_lib::progress::NullProgress;
use grit_lib::repo::Repository;
use grit_lib::state::HeadState;

use crate::context::{self, short_oid};
use crate::ui;

/// Maximum number of commits to list in the status shortlog before summarizing.
const SHORTLOG_LIMIT: usize = 10;

pub fn run() -> Result<()> {
    let repo = context::discover()?;
    let model =
        status(&repo, &StatusOptions::default(), &mut NullProgress).context("could not compute status")?;

    print_header(&repo, &model)?;
    print_changes(&model);
    print_hints(&model);
    Ok(())
}

/// The branch line, plus an ahead-of-target shortlog when there is one.
fn print_header(repo: &Repository, model: &StatusModel) -> Result<()> {
    match &model.head {
        HeadState::Branch {
            short_name,
            oid: Some(head_oid),
            ..
        } => {
            let Some(target) = context::find_target_branch(repo)? else {
                println!("On {short_name}");
                println!();
                return Ok(());
            };

            let commits = context::commits_ahead_of(repo, *head_oid, target.oid)?;
            if commits.is_empty() {
                println!("On {short_name}  ·  even with {}", target.display_name);
                println!();
                return Ok(());
            }

            println!(
                "On {short_name}  ·  {} ahead of {}",
                commits.len(),
                target.display_name
            );
            println!();
            for commit in commits.iter().take(SHORTLOG_LIMIT) {
                println!("  {}  {}", short_oid(&commit.oid), commit.subject);
            }
            if commits.len() > SHORTLOG_LIMIT {
                println!("  … and {} more", commits.len() - SHORTLOG_LIMIT);
            }
            println!();
        }
        HeadState::Branch { short_name, .. } => {
            println!("On {short_name} — no commits yet");
            println!();
        }
        HeadState::Detached { oid } => {
            println!("Detached at {}", short_oid(oid));
            println!();
        }
        HeadState::Invalid => {
            println!("HEAD is in an unknown state");
            println!();
        }
    }
    Ok(())
}

fn print_changes(model: &StatusModel) {
    let clean =
        model.staged.is_empty() && model.unstaged.is_empty() && model.untracked.is_empty();
    if clean {
        println!("Nothing to commit — working tree clean.");
        return;
    }

    ui::print_change_group("Staged", &model.staged);
    ui::print_change_group("Changed (not staged)", &model.unstaged);
    ui::print_untracked(&model.untracked);
}

fn print_hints(model: &StatusModel) {
    let mut hints = Vec::new();
    if !model.unstaged.is_empty() || !model.untracked.is_empty() {
        hints.push("gs add <file> to stage");
    }
    if !model.staged.is_empty() {
        hints.push("gs commit \"message\" to commit");
    }
    if !hints.is_empty() {
        println!("→ {}", hints.join("  ·  "));
    }
}
