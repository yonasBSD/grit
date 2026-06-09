# t7406-submodule-update

Ticket: d7ea3d

## Starting state
58/70 (regression from recorded 67/70; a shared submodule URL-resolution change degraded it).
Failing at first run: 5, 27, 48, 49, 51, 52, 58, 62, 63, 64, 65, 66.

## Fix 1: relative-URL resolution for nested submodules
`resolve_submodule_super_url` (grit/src/commands/submodule.rs) used the submodule worktree
*path* as the base when resolving `../foo` relative URLs for nested repos, instead of the
submodule's own `remote.<default>.url`. C Git's `resolve_relative_url` always resolves against
`remote.<default>.url` (cwd only as fallback). Changed the nested branch to use
`default_remote_url_raw(repo_git_dir)` with a worktree fallback.
Fixed tests 5, 27, 49, 52, 62, 63.

## Fix 2: relative submodule gitlinks
Submodule clones (both `submodule update` clone path in
grit/src/commands/_submodule_run_update_inner.rs.inc and `clone --recurse-submodules` path in
grit/src/commands/clone.rs) used `grit clone --separate-git-dir`, which writes an *absolute*
`gitdir:` path (correct for top-level clone, t5601). C Git's submodule machinery
(`connect_work_tree_and_git_dir`, dir.c) writes a *relative* gitlink so a copied/moved
superproject keeps its submodule pointing at the copy. After each submodule clone, rewrite the
`.git` gitlink to relative via `write_submodule_gitfile`.
Fixed tests 64, 65, 66 (cp -r top-clean top-cloned then operating on the copy).

## Fix 3: `push origin :` (matching refspec) no-op success
Test 48 ran `git push origin :` on a detached HEAD with no matching branches. grit `bail!`ed with
"No refs in common and none specified". C Git (send-pack.c / transport.c) returns 0 in that case
and prints "Everything up-to-date". Changed grit/src/commands/push.rs to not bail when the
matching `:`/`+:` refspec matches nothing; only warn (to stderr) when the remote advertises no
refs at all, then fall through to the empty-updates "Everything up-to-date" path. Fixed test 48.

## Fix 4: submodule update no-op when already at recorded commit (ignore dirtiness)
Tests 51 and 58. grit's "already at recorded commit" fast path in
_submodule_run_update_inner.rs.inc additionally required a clean worktree; a dirty submodule
already at the recorded oid fell through to `pull --rebase`, which died with "cannot pull with
rebase: You have unstaged changes". C Git's update_submodule only runs the checkout/rebase/merge
procedure when `!oideq(oid, suboid) || force` (builtin/submodule--helper.c) — dirtiness is
irrelevant. Removed the `submodule_worktree_clean_for_update` requirement from the checkout-path
fast skip. Fixed 51; 58 was a cascade of 51's broken state (a dirty submodule with update=rebase).

## Fix 5: `submodule add` relative gitlink (t7400 regression repair)
Commit 27cd94c89 ("absolute separate-git-dir gitfile", in the common base) made
`clone --separate-git-dir` write an absolute `gitdir:` path. That regressed `submodule add`
(t7400.19) and the t7406 recursive tests, which need a *relative* gitlink. Added a
`write_submodule_gitfile` call after the clone in `submodule add` (grit/src/commands/submodule.rs)
to match C's `connect_work_tree_and_git_dir`. Restored t7400 to 124/124.

## Result
t7406: 70/70 (was 58/70 at start). t7400: 124/124 (regression repaired). No new clippy warnings;
grit-lib unit tests pass modulo the 2 known ignore::gitignore_glob failures.

## Out of scope (pre-existing, from 27cd94c89 in the common base, NOT regressed by this work)
- t7407-submodule-foreach 20/23 (TOML baseline 23 is stale, pre-dates 27cd94c89): `submodule
  status --recursive` shows `(remotes/origin/main)` instead of `(heads/main)` for recursively-
  updated submodules — the at-recorded fast path skips local-branch HEAD attachment. Separate bug.
- t5601-clone 112/115 (unchanged from baseline; the absolute top-level gitlink that 27cd94c89
  added is intentionally preserved by this work — only submodule gitlinks are rewritten relative).
