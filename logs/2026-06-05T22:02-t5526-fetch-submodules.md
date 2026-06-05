# t5526-fetch-submodules — submodule fetch recursion

Ticket 1d762d. Started 22/56 passing.

## Root causes found & fixed

1. **`git submodule add` of an existing repo detached the source repo's HEAD.**
   For the "Adding existing repo" path, grit ran `checkout_submodule_worktree` which did a
   `checkout <oid>` and detached HEAD (to a bare OID), leaving `refs/heads/<branch>` stale.
   Git (`builtin/submodule--helper.c:add_submodule`) only runs `checkout -f` on the *clone*
   path; the existing-repo path leaves the worktree and its branch HEAD untouched.
   Fix (grit/src/commands/submodule.rs): track `did_clone`; only run the post-add worktree
   checkout when we cloned. This was the big one — with a detached source HEAD, later
   `add_submodule_commits` advanced HEAD but not the branch, so the downstream submodule fetch
   found "no new commits" and printed nothing (subtests 2,4,5,8,... all failed).

2. **`From <url>` header / stored clone URL wrong (`/.` vs verbatim).**
   `git clone .` must store the remote URL as `<cwd>/.` (Git `absolute_pathdup`: prepend cwd,
   no normalization). grit's `setup_origin_remote` / `setup_origin_remote_bare` used
   `source_path.canonicalize()` which stripped the trailing `.` and resolved symlinks. The
   `From` line had a compensating hack in fetch.rs that *appended* `/.` to the canonicalized
   remote git_dir — correct for the super (`From <pwd>/.`) but WRONG for submodules
   (`From <pwd>/submodule/.` instead of `<pwd>/submodule`).
   Fix:
   - clone.rs: new `absolute_clone_source_url()` = `cwd.canonicalize().join(literal source)`,
     used by `setup_origin_remote{,_bare}`. Preserves `.`/`./` like Git; symlink-resolved cwd
     matches Git's getcwd.
   - fetch.rs `resolve_fetch_from_line_url`: just return `normalize_fetch_url_display(raw_url)`
     (the configured URL, trimmed of trailing `/` and `.git`) — Git's actual behavior.

3. Build fix: clone.rs:6723 `UnpackOptions` initializer was missing the new
   `shallow_boundaries` field added by another agent; added `..Default::default()` to match the
   other call sites (bundle.rs, bundle_uri.rs) so the tree builds.

## Result
34/56 passing (was 22).

## NOTE / hazard
fetch.rs is being concurrently edited by another agent — my `resolve_fetch_from_line_url`
change was reverted once mid-run by their commit landing on disk; re-applied. If t5526
regresses to the `/.`-on-submodule failure, re-apply that one-liner.

## Round 2 fixes (→ 37/56)

4. **`submodule.recurse` config now triggers recursive fetch** (fetch.rs
   `fetch_recurse_submodules_mode`): scan config entries in order, last of
   {`fetch.recurseSubmodules`, `submodule.recurse`} wins (matches git fetch_config_callback).

5. **Recursion into a submodule whose work tree is gone** (index has no submodule but a
   newly-fetched super commit changes one — subtests 27/28/31). Three parts:
   - `get_default_remote_from_git_dir()` in submodule.rs: read the default remote from the
     module git dir directly when the work tree is absent (the old `get_default_remote_for_path`
     walked the work tree and died "could not get a repository handle").
   - fetch_submodule_recurse.rs: `--work-tree=.` must be a *global* git option (before `fetch`),
     not a fetch arg, else "unexpected argument '--work-tree'".
   - Separate the `at commit <oid>` *display* (from the changed-submodule super_oid) from the
     actual fetch: do a NORMAL fetch first (default refspec), then a by-OID follow-up only if the
     needed commits are still missing (git's two-pass index/changed + oid_fetch_tasks). Passing
     the oids on the first fetch wrongly produced `-> FETCH_HEAD` instead of `-> origin/sub`.
   This got 52/53 (name-conflicted submodules) passing too.

## Round 3 (→ 39/56): nested deep "at commit" annotation

fetch_submodule_recurse.rs index loop: an index-listed gitlink whose **work tree is absent**
falls through (in git) to the changed task and is annotated `at commit <super_oid>`. Gate
`at_commit` on `!populated` so a nested deepsubmodule reached while its parent submodule is itself
unpopulated (`--work-tree=.`) shows `at commit <sub_head>` (fixes 27/28) without regressing the
normal populated case (20/22/23/25 stay un-annotated). Net 37→39.

## Regression fix: t7400 #96 (submodule add reactivation)

My `did_clone` change exposed a latent bug: `git submodule add --force` of a path whose
`.git/modules/<name>` already exists (re-adding a `git rm`-ed submodule) was doing
remove-module-dir + re-clone. Git instead REUSES the existing module dir (no clone — the source
URL need not even exist), drops the stale `index`, connects the work tree, and checks out
(builtin/submodule--helper.c `clone_submodule`: it only clones `if (!file_exists(sm_gitdir))`).
Rewrote the clone block in submodule.rs to reactivate an existing module dir instead of
removing+cloning. t7400 back to 124/124.

## Remaining failures (17): 30-36, 38-45, 55, 56

- **30** (`setup downstream branch with other submodule`) fails on
  `git checkout --recurse-submodules super`: "pathspec '<oid>' did not match" / "failed to
  checkout submodule at 'submodule' to <oid>". This is a **checkout --recurse-submodules** bug
  (submodule needs a commit that `submodule update --init` didn't fetch), NOT fetch — and it
  cascades into 31-36 (downstream left in an inconsistent state). Fixing checkout recursion
  would likely unblock most of 31-36.
- **27/28** (changed-but-not-in-index, deep nested): the *deep* submodule reached via the nested
  fetch is shown as `Fetching submodule .../deepsubmodule` (no `at commit`) but git shows
  `at commit <sub_head>`. In git the deep module in the nested fetch is processed via the
  *changed* path (not the index path) so it gets the annotation. grit's nested fetch sees deep
  in the submodule's git-dir index and uses the index path. Needs: in the nested fetch, when a
  submodule is BOTH in the index AND changed by the parent's new commits, prefer the
  changed-path annotation.
- **40-45** (on-demand outside standard refspec / FETCH_HEAD / custom remote): the by-OID
  follow-up exists now but these need the FETCH_HEAD recording and custom-remote-name plumbing
  verified.
- **38** broken-repo handling, **39** renamed submodule, **55/56** `fetch --all` recursion.
