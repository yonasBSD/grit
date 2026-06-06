# t5526-fetch-submodules — work log (2026-06-06/07)

Ticket 1d762d. Started at 41/56 (per prior-agent comments).

## Root-cause split (re-verified)
Fresh `-v` run showed the REAL failing set was: 32,33,34,35,36,38,39,40,41,42,43,44,45,55,56 (15).
(The ticket-runs log was stale from the original 19/56 scan.)

Two buckets:
- BUCKET A (test-porting bug, OUT OF ALLOWED EDITS): `_wrap_cd_subshell.py` over-wrapped tests 31
  and 54 in a `( )` subshell that upstream lacks. Their top-level `test_when_finished` cleanups
  (`rm expect.err.sub2` / `rm -fr src_clone`) are lost on subshell exit, so a stale `expect.err.sub2`
  poisons `verify_fetch_result` for 32-36 and a leftover `src_clone` breaks 55/56. Confirmed
  empirically (183-byte stale file after `--run=1-31`). I may only flip expect_failure→success, so I
  cannot un-wrap. A /tmp copy with the wrappers removed isolated the genuine grit bugs below.

## Genuine grit fixes made (all faithful to git submodule.c / submodule--helper.c)
1. `grit/src/commands/submodule.rs` `get_default_remote_for_path_in_super`: when a populated
   submodule has no `.gitmodules` entry, fall back to the submodule's own config
   (`repo_default_remote(&subrepo)`) instead of dying "could not get a repository handle". (#36)
2. `grit/src/fetch_submodule_recurse.rs` index loop: synthesize a name==path submodule for an
   index gitlink absent from `.gitmodules` but populated (git `get_non_gitmodules_submodule` /
   `default_name_or_path`); drop the index-path active gating (git's index task never checks
   `is_tree_submodule_active`). (#36)
3. `fetch_submodule_recurse.rs` `changed_submodule_git_dir`: resolve a changed submodule's gitdir
   by NAME (`.git/modules/<name>`) when the recorded path moved — handles `git mv` renames. (#39)
4. `grit/src/commands/fetch.rs` `--all`/`--multiple`: recurse into submodules per-remote (git's
   `fetch_multiple` runs each remote as its own recursing subprocess) instead of once at the end —
   two remotes → two "Fetching submodule" lines. (#55)
5. `fetch_submodule_recurse.rs` by-OID second pass: re-derive the remote by spawning
   `git submodule--helper get-default-remote <path>` (git `oid_fetch_tasks` branch), which also
   emits the GIT_TRACE line the tests assert and picks up custom remote names. (#40/#44)
6. `fetch.rs` FETCH_HEAD-only updates: record the fetched commit OID as a submodule-recursion tip
   (git `check_for_new_submodule_commits` runs on FETCH_HEAD too), so on-demand recursion fires for
   fetches with no tracking-ref destination. (#41/#42/#43/#45)

## Deferred
- #38 "fetching submodule into a broken repository": after `rm -r dst/sub/.git/objects`, grit
  status/diff/fetch must FAIL. Root: `diff.rs::submodule_porcelain_flags` — `read_submodule_head_oid`
  still returns Some but the HEAD commit object is unreadable so `sub_head_tree=None` → looks clean →
  exit 0. Fixing needs a fatal error threaded through the status/diff/fetch hot path; high regression
  risk for one test. Independent of the 39-45 chain (uses `dst`, not `downstream`). Left for mop-up.
