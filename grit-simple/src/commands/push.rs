//! `gs push` — publish the current branch to its remote.
//!
//! No upstream ceremony: `gs push` sends the current branch to `origin` (or the
//! configured `branch.<name>.remote`) under the same name, creating it on the
//! remote if needed.

use anyhow::{bail, Context, Result};
use grit_lib::config::ConfigSet;
use grit_lib::push_report::PushRefStatus;
use grit_lib::state::{resolve_head, HeadState};
use grit_lib::transfer::PushRefSpec;

use crate::commands::auth;
use crate::context;
use crate::net;

pub fn run() -> Result<()> {
    let repo = context::discover()?;

    let (short_name, oid) = match resolve_head(&repo.git_dir)? {
        HeadState::Branch { short_name, oid: Some(oid), .. } => (short_name, oid),
        HeadState::Branch { .. } => bail!("no commits yet to push"),
        HeadState::Detached { .. } => bail!("HEAD is detached; gs push needs a branch"),
        HeadState::Invalid => bail!("HEAD is in an unknown state"),
    };

    let config = ConfigSet::load(Some(&repo.git_dir), true).context("could not load config")?;
    let remote = config
        .get(&format!("branch.{short_name}.remote"))
        .filter(|r| !r.trim().is_empty())
        .unwrap_or_else(|| net::DEFAULT_REMOTE.to_owned());
    let dst = config
        .get(&format!("branch.{short_name}.merge"))
        .filter(|m| m.starts_with("refs/"))
        .unwrap_or_else(|| format!("refs/heads/{short_name}"));

    let spec = PushRefSpec {
        src: Some(oid),
        dst,
        force: false,
        delete: false,
        expected_old: None,
        expect_absent: false,
    };

    // On an HTTPS auth failure, `gs auth` can refresh the token and we retry once.
    let outcome = match net::push(&repo, &config, &remote, std::slice::from_ref(&spec)) {
        Ok(outcome) => outcome,
        Err(err) => {
            let url = net::remote_url(&config, &remote).unwrap_or_default();
            if auth::offer_reauth(&err, &url)? {
                net::push(&repo, &config, &remote, std::slice::from_ref(&spec))?
            } else {
                return Err(err);
            }
        }
    };

    let mut rejected = false;
    for result in &outcome.results {
        let target = format!("{remote} {}", result.remote_ref);
        match result.status {
            PushRefStatus::Ok => println!("  pushed {short_name} → {target}"),
            PushRefStatus::UpToDate => println!("  {target} already up to date"),
            PushRefStatus::RejectNonFastForward => {
                rejected = true;
                eprintln!("  rejected {target}: not a fast-forward — run `gs pull` first");
            }
            _ => {
                rejected = true;
                let reason = result.message.clone().unwrap_or_else(|| "rejected".to_owned());
                eprintln!("  rejected {target}: {reason}");
            }
        }
    }

    if rejected {
        bail!("push rejected");
    }
    Ok(())
}
