# t1060-object-corruption — clone checkout failure leaves repo intact

Ticket: dd02cf

## Problem
Subtest 12 ("error detected during checkout leaves repo intact") failed. When
`git clone --local` hits a corrupt object during the worktree checkout phase,
git leaves the partial repository on disk (`<dest>/.git`, index, objects) so the
user can inspect and retry. grit instead removed the entire destination
directory via its `JunkGuard` Drop cleanup.

## Root cause
Upstream `builtin/clone.c` sets `junk_mode = JUNK_LEAVE_REPO` immediately before
calling `checkout()`. From that point a failure leaves the repo on disk and only
prints a warning; on success it becomes `JUNK_LEAVE_ALL`. grit's `JunkGuard` only
had a binary disarmed/armed state, so any failure after the objects were written
(including checkout) still deleted everything.

## Fix
`grit/src/commands/clone.rs`:
- Added `enum JunkMode { LeaveNone, LeaveRepo, LeaveAll }` mirroring git's
  `enum junk_mode`.
- Replaced `JunkGuard.disarmed: bool` with `mode: JunkMode`. Drop:
  - `LeaveAll` -> no-op (success / disarm)
  - `LeaveRepo` -> print git's "Clone succeeded, but checkout failed." warning,
    leave everything on disk
  - `LeaveNone` -> remove git dir + work tree (existing behavior)
- Added `JunkGuard::leave_repo()`; `disarm()` now sets `LeaveAll`.
- Call `junk_guard.leave_repo()` in `run()` right before the worktree checkout
  block, matching git's `junk_mode = JUNK_LEAVE_REPO` placement.

Only the main `run()` clone path uses `JunkGuard` (the `--local`/file-URL path);
the other transport run_* fns have their own cleanup and were not touched.

## Result
- Manual repro: `clone --local` of a bit-corrupted repo now exits 1, prints the
  warning, and keeps `corrupt-checkout/.git`.
- t1060: 16/17, failing=0, fully_passing=true. The remaining "not ok 14" is a
  `test_expect_failure` (`# TODO known breakage` — git itself does not detect
  misnamed objects on `--local` clone), not a real failure.
- `cargo test -p grit-lib --lib`: 276 pass; only the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures (unrelated to this ticket).
