//! `gs commit` — record the staged changes as a new commit.

use anyhow::{bail, Context, Result};
use grit_lib::config::ConfigSet;
use grit_lib::ident_resolve::IdentRole;
use grit_lib::objects::{serialize_commit, CommitData, ObjectId, ObjectKind};
use grit_lib::porcelain::status::{status, StatusOptions};
use grit_lib::progress::NullProgress;
use grit_lib::state::HeadState;
use grit_lib::{refs, write_tree::write_tree_from_index};
use time::OffsetDateTime;

use crate::commands::add;
use crate::context::{self, short_oid, subject_line};

pub fn run(message: Option<String>, all: bool) -> Result<()> {
    let repo = context::discover()?;

    if all {
        add::stage(&repo, &[])?;
    }

    let message = match message {
        Some(m) if !m.trim().is_empty() => m,
        _ => bail!("provide a commit message, e.g. gs commit \"what changed\""),
    };

    let model = status(&repo, &StatusOptions::default(), &mut NullProgress)
        .context("could not compute status")?;
    if model.staged.is_empty() {
        bail!("nothing staged to commit — use \"gs add\" first");
    }

    let (refname, short_name, parent) = match &model.head {
        HeadState::Branch {
            refname,
            short_name,
            oid,
        } => (refname.clone(), short_name.clone(), *oid),
        HeadState::Detached { .. } => {
            bail!("HEAD is detached; gs commit needs a branch")
        }
        HeadState::Invalid => bail!("HEAD is in an unknown state"),
    };

    let index = model.index;
    let tree = write_tree_from_index(&repo.odb, &index, "").context("could not write tree")?;

    let config = ConfigSet::load(Some(&repo.git_dir), true).context("could not load config")?;
    let now = OffsetDateTime::now_utc();
    let author = context::identity(&config, IdentRole::Author, "GIT_AUTHOR_DATE", now)?;
    let committer = context::identity(&config, IdentRole::Committer, "GIT_COMMITTER_DATE", now)?;

    let mut message = message.trim().to_owned();
    message.push('\n');
    let subject = subject_line(&message);

    let commit_data = CommitData {
        tree,
        parents: parent.into_iter().collect(),
        author,
        committer: committer.clone(),
        author_raw: Vec::new(),
        committer_raw: Vec::new(),
        encoding: None,
        message,
        raw_message: None,
    };
    let bytes = serialize_commit(&commit_data);
    let oid = repo
        .odb
        .write(ObjectKind::Commit, &bytes)
        .context("could not store commit")?;

    let old = parent.unwrap_or_else(ObjectId::zero);
    let reflog_msg = if parent.is_some() {
        format!("commit: {subject}")
    } else {
        format!("commit (initial): {subject}")
    };
    refs::write_ref(&repo.git_dir, &refname, &oid).context("could not update branch")?;
    refs::append_reflog(
        &repo.git_dir,
        &refname,
        &old,
        &oid,
        &committer,
        &reflog_msg,
        false,
    )?;
    refs::append_reflog(
        &repo.git_dir,
        "HEAD",
        &old,
        &oid,
        &committer,
        &reflog_msg,
        false,
    )?;

    let count = model.staged.len();
    println!("[{short_name} {}] {subject}", short_oid(&oid));
    println!(
        "{count} change{} committed",
        if count == 1 { "" } else { "s" }
    );
    Ok(())
}
