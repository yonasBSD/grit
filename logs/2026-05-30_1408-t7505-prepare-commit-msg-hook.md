# t7505-prepare-commit-msg-hook — cherry-pick hook wiring

Branch: wf/p4/t7505-prepare-commit-msg-hook
Date: 2026-05-30

## Result
- Before this session: 19/23 (prior commits on this branch already fixed
  commit.rs subtests 5/8/10/11/12 and merge.rs subtests 14/15/22).
- After: 22/23.
- Subtests newly fixed this session: 17, 18, 23 (all cherry-pick).
- Remaining failure: subtest 16 (rebase -i) — NOT a hook bug; it is the
  rebase-interactive conflict/continue engine looping "Rebasing (1/15)" ->
  "Rebasing (2/15)" and never maintaining .git/rebase-merge/done. rebase.rs
  already wires prepare-commit-msg (run_prepare_commit_msg_hook at
  rebase.rs:2084, commit_message_after_prepare_hook at 2104, editor path at
  3084). Per the diagnosis (HIGH conflict risk vs rebase plan files) this was
  left to the rebase plan owner; rebase.rs was NOT touched.

## Changes (grit/src/commands/cherry_pick.rs only)

1. Clean cherry-pick (subtests 17 & 23):
   New helper `run_cherry_pick_prepare_commit_msg_hook` mirrors
   sequencer.c:run_prepare_commit_msg_hook for the clean (non-conflict,
   non-amend) pick path:
   - write the message to COMMIT_EDITMSG
   - run prepare-commit-msg with arg1="message", GIT_EDITOR=":" via
     run_commit_hook + CommitHookEnv (the sequencer always passes
     editor_is_used=0 for a clean pick)
   - read the (possibly hook-rewritten) message back
   - on HookResult::Failed, print `error: 'prepare-commit-msg' hook failed`
     exactly once and bail with the sentinel "HOOK_FAILED" (the top-level
     error handler exits 1 without re-printing the hook name, so
     t7505's `grep -c prepare-commit-msg = 1` holds).
   Wired in just before create_cherry_pick_commit (the clean-commit path).

2. cherry-pick -e (subtest 18):
   New helper `finish_cherry_pick_via_commit` mirrors
   sequencer.c:run_git_commit. For `should_edit()` (i.e. `-e`), upstream sets
   msg_file=NULL and delegates to `git commit -e`; that spawned commit sees the
   leftover MERGE_MSG/CHERRY_PICK_HEAD state, so its prepare-commit-msg runs
   with arg1="merge" (builtin/commit.c:856-868) and the editor launches. We
   replicate by writing MERGE_MSG + CHERRY_PICK_HEAD on the staged index, then
   spawning `grit commit -n -e --allow-empty --no-gpg-sign` (the flags upstream
   derives from EDIT_MSG|ALLOW_EMPTY). Author is preserved via CHERRY_PICK_HEAD.
   Verified the `merge` result against real git 2.52.0 (it is the
   MERGE_MSG-exists -> hook_arg1="merge" quirk, not over-fitting).

## Quality gates
- cargo fmt --check: clean
- cargo test -p grit-lib --lib: 225 passed, 0 failed
- cargo clippy -p grit-lib -p grit-cli: no new warnings on changed lines
- Regression guards: t7503-pre-commit-and-pre-merge-commit-hooks 22/22,
  t5571-pre-push-hook 11/11 — no regressions.
- Cherry-pick regression sweep (no regressions; several improvements that
  reflect other branch work): t3500 4/4, t3501 17/21, t3502 12/12, t3503 6/6,
  t3505 17/17, t3506 10/11, t3507 27/44, t3508 9/14.
