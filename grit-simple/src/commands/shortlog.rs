//! `gs shortlog` — the commits on this branch that aren't on the target yet.

use anyhow::{bail, Context, Result};
use grit_lib::objects::ObjectId;
use grit_lib::state::{resolve_head, HeadState};

use crate::context::{self, short_oid};

pub fn run() -> Result<()> {
    let repo = context::discover()?;
    let head = resolve_head(&repo.git_dir).context("could not resolve HEAD")?;
    let (branch_name, head_oid) = current_branch_and_oid(&head)?;

    println!("On {branch_name}");

    let Some(target) = context::find_target_branch(&repo)? else {
        println!("No target branch found (tried target.branch, origin/master, origin/main, master, main).");
        return Ok(());
    };

    let commits = context::commits_ahead_of(&repo, head_oid, target.oid)?;
    println!(
        "Ahead of {} by {} commit{}",
        target.display_name,
        commits.len(),
        if commits.len() == 1 { "" } else { "s" }
    );

    for commit in commits {
        println!("{} {}", short_oid(&commit.oid), commit.subject);
    }

    Ok(())
}

fn current_branch_and_oid(head: &HeadState) -> Result<(&str, ObjectId)> {
    match head {
        HeadState::Branch {
            short_name,
            oid: Some(oid),
            ..
        } => Ok((short_name.as_str(), *oid)),
        HeadState::Branch { short_name, .. } => {
            bail!("branch '{short_name}' does not have any commits yet")
        }
        HeadState::Detached { .. } => bail!("HEAD is detached; gs shortlog needs a current branch"),
        HeadState::Invalid => bail!("HEAD is invalid"),
    }
}
