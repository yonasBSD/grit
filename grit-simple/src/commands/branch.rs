//! `gs branch` — list branches, or create / delete one.

use anyhow::{bail, Context, Result};
use grit_lib::refs;
use grit_lib::repo::Repository;
use grit_lib::state::{resolve_head, HeadState};

use crate::context;

pub fn run(name: Option<String>, delete: bool) -> Result<()> {
    let repo = context::discover()?;
    match name {
        None => list(&repo),
        Some(name) if delete => delete_branch(&repo, &name),
        Some(name) => create(&repo, &name),
    }
}

fn list(repo: &Repository) -> Result<()> {
    let current = match resolve_head(&repo.git_dir)? {
        HeadState::Branch { short_name, .. } => Some(short_name),
        _ => None,
    };

    let mut branches: Vec<String> = refs::list_refs(&repo.git_dir, "refs/heads/")
        .context("could not list branches")?
        .into_iter()
        .map(|(refname, _)| {
            refname
                .strip_prefix("refs/heads/")
                .unwrap_or(&refname)
                .to_owned()
        })
        .collect();
    branches.sort();

    if branches.is_empty() {
        println!("No branches yet.");
        return Ok(());
    }

    for branch in branches {
        if current.as_deref() == Some(branch.as_str()) {
            println!("* {branch}");
        } else {
            println!("  {branch}");
        }
    }
    Ok(())
}

fn create(repo: &Repository, name: &str) -> Result<()> {
    let branch_ref = format!("refs/heads/{name}");
    if refs::resolve_ref(&repo.git_dir, &branch_ref).is_ok() {
        bail!("branch '{name}' already exists");
    }

    let base = match resolve_head(&repo.git_dir)? {
        HeadState::Branch { oid: Some(oid), .. } | HeadState::Detached { oid } => oid,
        HeadState::Branch { .. } => bail!("no commits yet to create a branch from"),
        HeadState::Invalid => bail!("HEAD is in an unknown state"),
    };

    refs::write_ref(&repo.git_dir, &branch_ref, &base).context("could not create branch")?;
    println!("Created branch {name}");
    Ok(())
}

fn delete_branch(repo: &Repository, name: &str) -> Result<()> {
    if let HeadState::Branch { short_name, .. } = resolve_head(&repo.git_dir)? {
        if short_name == name {
            bail!("cannot delete '{name}' — it is the current branch");
        }
    }

    let branch_ref = format!("refs/heads/{name}");
    if refs::resolve_ref(&repo.git_dir, &branch_ref).is_err() {
        bail!("no branch named '{name}'");
    }

    refs::delete_ref(&repo.git_dir, &branch_ref).context("could not delete branch")?;
    println!("Deleted branch {name}");
    Ok(())
}
