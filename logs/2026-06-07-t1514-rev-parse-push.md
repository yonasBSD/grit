# t1514-rev-parse-push.sh — work log (2026-06-07)

Ticket: 2fe874 (group revlist-log-tail / thread B)

## Starting state
8/9 passing. Failing subtest 8: `@{push} with push refspecs`.

Config in that subtest:
- `push.default = nothing`
- `remote.origin.push = refs/heads/*:refs/heads/magic/*`
- `topic` tracks `origin/main`

Expected: `git rev-parse --symbolic-full-name topic@{push}` => `refs/remotes/origin/magic/topic`.
grit died with `fatal: push.default is nothing; no push destination`.

## Root cause
In `grit-lib/src/rev_parse.rs::resolve_push_full_ref_for_branch`:

1. The `push.default == "nothing"` early-return ran BEFORE consulting the
   configured `remote.<name>.push` refspecs. Upstream git
   (`remote.c::branch_get_push_1`) checks `if (remote->push.nr) { ... }` and
   returns from that block before ever reaching the `push.default` switch, so a
   configured push refspec overrides even `push.default = nothing`.

2. `push_refspec_mapped_tracking` only handled exact
   `refs/heads/<branch>:refs/heads/<dest>` refspecs; the test uses a wildcard
   `refs/heads/*:refs/heads/magic/*`.

## Fix
- Reordered `resolve_push_full_ref_for_branch`: when the push remote has any
  `push` refspec (`remote_has_push_refspec`), resolve exclusively through it and
  return (Ok mapped, or Err "push refspecs for '<remote>' do not include
  '<branch>'"). Only when there are no push refspecs do we fall through to the
  `push.default` logic (including the `nothing` bail).
- Added `match_refname_with_pattern` mirroring git's function: supports
  `prefix*suffix` patterns and exact refspecs, substituting the matched glob
  into the replacement side.
- Rewrote `push_refspec_mapped_tracking` to apply the push refspec to
  `refs/heads/<branch>` (-> push dest), then map that dest to a local tracking
  ref via the remote's `fetch` refspecs (`map_dest_to_tracking`), mirroring
  git's `apply_refspecs(&remote->push, ...)` + `tracking_for_push_dest`.
  Falls back to the conventional `refs/heads/<x>` -> `refs/remotes/<remote>/<x>`
  mapping when no fetch refspec matches.

## Result
t1514-rev-parse-push: 9/9. `cargo test -p grit-lib --lib`: 276 passed, only the
2 known-preexisting `ignore::gitignore_glob_tests` failures remain (not mine).
