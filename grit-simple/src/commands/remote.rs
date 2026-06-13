//! `gs remote` — list remotes, or add one.

use anyhow::{bail, Context, Result};
use grit_lib::config::{ConfigFile, ConfigScope, ConfigSet};
use grit_lib::repo::Repository;

use crate::context;

/// `Some((name, url))` adds a remote; `None` lists them.
pub fn run(add: Option<(String, String)>) -> Result<()> {
    let repo = context::discover()?;
    match add {
        None => list(&repo),
        Some((name, url)) => add_remote(&repo, &name, &url),
    }
}

fn list(repo: &Repository) -> Result<()> {
    let config = ConfigSet::load(Some(&repo.git_dir), true).context("could not load config")?;
    let names = remote_names(&config);
    if names.is_empty() {
        println!("No remotes. Add one with: gs remote add <name> <url>");
        return Ok(());
    }
    for name in names {
        let url = config
            .get(&format!("remote.{name}.url"))
            .unwrap_or_else(|| "(no url)".to_owned());
        println!("{name}\t{url}");
    }
    Ok(())
}

fn add_remote(repo: &Repository, name: &str, url: &str) -> Result<()> {
    let config = ConfigSet::load(Some(&repo.git_dir), true).context("could not load config")?;
    if remote_names(&config).iter().any(|n| n == name) {
        bail!("remote '{name}' already exists");
    }

    let path = repo.git_dir.join("config");
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut cfg = ConfigFile::parse(&path, &content, ConfigScope::Local)
        .context("could not parse repository config")?;
    cfg.set(&format!("remote.{name}.url"), url)?;
    cfg.set(
        &format!("remote.{name}.fetch"),
        &format!("+refs/heads/*:refs/remotes/{name}/*"),
    )?;
    cfg.write().context("could not write repository config")?;

    println!("Added remote {name} → {url}");
    Ok(())
}

/// Distinct, sorted remote names from any `remote.<name>.*` config entry.
fn remote_names(config: &ConfigSet) -> Vec<String> {
    let mut names: Vec<String> = config
        .entries()
        .iter()
        .filter_map(|entry| {
            let rest = entry.key.strip_prefix("remote.")?;
            // `remote.<name>.<key>` — the name is everything before the last dot.
            rest.rsplit_once('.').map(|(name, _)| name.to_owned())
        })
        .collect();
    names.sort();
    names.dedup();
    names
}
