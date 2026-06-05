# t3425 rebase topology merges

## 2026-06-05

- Claimed ticket `8645d0`.
- Target test: `tests/t3425-rebase-topology-merges.sh`.
- Initial ticket snapshot said 12/13 failing, covering non-linear rebase linearization across
  `--apply`, `-m`, and `-i`.
- Fresh harness reproduction: 1/13.
- Direct verbose reproduction for subtest 2 showed `git rebase --apply e w` replaying commits
  `n`, `o`, then merge commit `w`. The replay of `w` became empty and aborted.
- Root cause: ordinary rebases without `--rebase-merges` kept merge commits in the replay list.
- Patch: filter merge commits out of non-root, non-`--rebase-merges` replay lists before todo
  generation.
- After rebuilding, harness improved to 9/13.
- Direct verbose reproduction then showed the first remaining failure in interactive mode:
  `git rebase -i e w` checked out `w` but printed `HEAD is up to date` after the unchanged todo.
- Root cause: the interactive unchanged-todo fast path considered any `upstream` ancestor of `HEAD`
  up to date, even when the todo still contained commits to replay.
- Patch: only use that interactive up-to-date fast path when the computed replay list is empty.
- The next direct run showed interactive replay trying to apply base commit `a`; the no-cherry
  commit collector walked the first-parent chain until a merge base, which fails when the upstream
  was merged as a second parent.
- Patch: make the no-cherry collector use a proper `upstream..head` rev walk and reverse it to
  oldest-first order.
- Final targeted harness: `t3425-rebase-topology-merges.sh` passed 13/13.
- Regression harness: `t3431-rebase-fork-point.sh` still passed 26/26.
- Nearby incomplete harness: `t3421-rebase-topology-linear.sh` reported 49/64, matching its open
  incomplete status rather than a new blocker for this ticket.
- Validation:
  - `cargo fmt` passed.
  - `cargo check -p grit-cli` passed with the existing `diff.rs` `ext_total` warning.
  - `cargo clippy --fix --allow-dirty` exited 0 with the repository's existing warning backlog and
    failed unrelated autofix report.
  - `cargo test -p grit-lib --lib` reported the known baseline: 252 passed, 2 ignore glob tests
    failed.
