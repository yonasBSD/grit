//! `gs init` — create a new, empty repository.

use std::path::PathBuf;

use anyhow::{Context, Result};
use grit_lib::repo::init_repository;

pub fn run(path: Option<String>, bare: bool) -> Result<()> {
    let path = PathBuf::from(path.unwrap_or_else(|| ".".to_owned()));

    let repo = init_repository(&path, bare, "main", None, "files")
        .with_context(|| format!("could not initialize a repository at {}", path.display()))?;

    let kind = if bare {
        "bare repository"
    } else {
        "repository"
    };
    println!("Initialized empty {kind} in {}", repo.git_dir.display());
    Ok(())
}
