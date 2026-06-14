//! `gs fetch` — download refs and objects from a remote (default `origin`).

use anyhow::{Context, Result};
use grit_lib::config::ConfigSet;

use crate::commands::auth;
use crate::context::{self, short_oid};
use crate::net;

pub fn run(remote: Option<String>) -> Result<()> {
    let repo = context::discover()?;
    let config = ConfigSet::load(Some(&repo.git_dir), true).context("could not load config")?;
    let remote = remote.unwrap_or_else(|| net::DEFAULT_REMOTE.to_owned());

    let refspecs = net::fetch_refspecs(&config, &remote);
    // On an HTTPS auth failure, `gs auth` can refresh the token and we retry once.
    let outcome = match net::fetch(&repo, &config, &remote, refspecs.clone()) {
        Ok(outcome) => outcome,
        Err(err) => {
            let url = net::remote_url(&config, &remote).unwrap_or_default();
            if auth::offer_reauth(&err, &url)? {
                net::fetch(&repo, &config, &remote, refspecs)?
            } else {
                return Err(err);
            }
        }
    };

    let mut updated = 0;
    for update in &outcome.updates {
        if update.old_oid == update.new_oid {
            continue;
        }
        let Some(local) = &update.local_ref else {
            continue;
        };
        updated += 1;
        let from = update.old_oid.as_ref().map_or_else(|| "new".to_owned(), short_oid);
        let to = update.new_oid.as_ref().map_or_else(|| "deleted".to_owned(), short_oid);
        println!("  {local}  {from} → {to}");
    }

    if updated == 0 {
        println!("Already up to date with {remote}.");
    } else {
        println!("Fetched {updated} update{} from {remote}.", plural(updated));
    }
    Ok(())
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}
