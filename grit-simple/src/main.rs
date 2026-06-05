//! `gi` — a small opinionated command line interface backed by `grit-lib`.

use std::collections::HashSet;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use grit_lib::config::ConfigSet;
use grit_lib::objects::{parse_commit, ObjectId, ObjectKind};
use grit_lib::refs;
use grit_lib::repo::Repository;
use grit_lib::state::{resolve_head, HeadState};

/// A simplified alternative to the Git-compatible `grit` command line.
#[derive(Debug, Parser)]
#[command(name = "gi", version, about = "A simple Grit-powered CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Top-level `gi` commands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Show the current branch and commits ahead of the target branch.
    #[command(alias = "sl")]
    Shortlog,
}

#[derive(Debug, Clone)]
struct TargetBranch {
    display_name: String,
    oid: ObjectId,
}

#[derive(Debug, Clone)]
struct CommitSummary {
    oid: ObjectId,
    subject: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Shortlog => shortlog(),
    }
}

fn shortlog() -> Result<()> {
    let repo = Repository::discover(None).context("not in a repository")?;
    let head = resolve_head(&repo.git_dir).context("could not resolve HEAD")?;
    let (branch_name, head_oid) = current_branch_and_oid(&head)?;

    println!("On {branch_name}");

    let Some(target) = find_target_branch(&repo)? else {
        println!("No target branch found (tried target.branch, origin/master, origin/main, master, main).");
        return Ok(());
    };

    let commits = commits_ahead_of(&repo, head_oid, target.oid)?;
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
        HeadState::Detached { .. } => bail!("HEAD is detached; gi shortlog needs a current branch"),
        HeadState::Invalid => bail!("HEAD is invalid"),
    }
}

fn find_target_branch(repo: &Repository) -> Result<Option<TargetBranch>> {
    for candidate in target_branch_candidates(repo)? {
        if let Some(oid) = resolve_branch_candidate(repo, &candidate) {
            return Ok(Some(TargetBranch {
                display_name: candidate,
                oid,
            }));
        }
    }
    Ok(None)
}

fn target_branch_candidates(repo: &Repository) -> Result<Vec<String>> {
    let config = ConfigSet::load(Some(&repo.git_dir), true).context("could not load config")?;
    let mut candidates = Vec::new();
    if let Some(target) = config.get("target.branch") {
        let trimmed = target.trim();
        if !trimmed.is_empty() {
            candidates.push(trimmed.to_owned());
        }
    }
    candidates.extend([
        "origin/master".to_owned(),
        "origin/main".to_owned(),
        "master".to_owned(),
        "main".to_owned(),
    ]);
    Ok(candidates)
}

fn resolve_branch_candidate(repo: &Repository, candidate: &str) -> Option<ObjectId> {
    for refname in candidate_refnames(candidate) {
        if let Ok(oid) = refs::resolve_ref(&repo.git_dir, &refname) {
            return Some(oid);
        }
    }
    None
}

fn candidate_refnames(candidate: &str) -> Vec<String> {
    if candidate.starts_with("refs/") || candidate == "HEAD" {
        return vec![candidate.to_owned()];
    }

    if let Some(remote_branch) = candidate.strip_prefix("origin/") {
        return vec![
            format!("refs/remotes/origin/{remote_branch}"),
            format!("refs/heads/{candidate}"),
        ];
    }

    vec![
        format!("refs/heads/{candidate}"),
        format!("refs/remotes/{candidate}"),
    ]
}

fn commits_ahead_of(
    repo: &Repository,
    head: ObjectId,
    target: ObjectId,
) -> Result<Vec<CommitSummary>> {
    let excluded = reachable_commits(repo, target)?;
    let mut seen = HashSet::new();
    let mut stack = vec![head];
    let mut commits = Vec::new();

    while let Some(oid) = stack.pop() {
        if !seen.insert(oid) || excluded.contains(&oid) {
            continue;
        }
        let commit = read_commit(repo, &oid)?;
        stack.extend(commit.parents.iter().copied());
        commits.push(CommitSummary {
            oid,
            subject: subject_line(&commit.message),
        });
    }

    Ok(commits)
}

fn reachable_commits(repo: &Repository, start: ObjectId) -> Result<HashSet<ObjectId>> {
    let mut reachable = HashSet::new();
    let mut stack = vec![start];

    while let Some(oid) = stack.pop() {
        if !reachable.insert(oid) {
            continue;
        }
        let commit = read_commit(repo, &oid)?;
        stack.extend(commit.parents.iter().copied());
    }

    Ok(reachable)
}

fn read_commit(repo: &Repository, oid: &ObjectId) -> Result<grit_lib::objects::CommitData> {
    let object = repo
        .odb
        .read(oid)
        .with_context(|| format!("could not read commit {oid}"))?;
    if object.kind != ObjectKind::Commit {
        bail!("object {oid} is a {}, not a commit", object.kind);
    }
    parse_commit(&object.data).with_context(|| format!("could not parse commit {oid}"))
}

fn subject_line(message: &str) -> String {
    message
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or("(no subject)")
        .to_owned()
}

fn short_oid(oid: &ObjectId) -> String {
    oid.to_hex().chars().take(7).collect()
}
