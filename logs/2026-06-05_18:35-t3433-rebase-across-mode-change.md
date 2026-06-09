# t3433 rebase across mode change

## 2026-06-05

- Claimed ticket `452c68`.
- Target test: `tests/t3433-rebase-across-mode-change.sh`.
- Ticket snapshot says 1/4 pass; remaining failures cover apply and merge rebases across a branch
  where a directory was replaced by a symlink.
- Fresh harness reproduction: 1/4.
- Direct trace showed the first pick adding `unrelated` after rebasing onto a branch with `DS`
  as a symlink to `unrelated`. Grit added `unrelated` but left `DS` as an unmerged stage-2 entry.
- Root cause: the replay merge directory/file pre-pass treated a file/symlink on one side and
  directory descendants on the other side as a conflict even when those descendants were unchanged
  from the base. Git resolves that case by taking the file/symlink side and dropping the unchanged
  descendants.
- Patch: when the directory side matches the base and the file side has no descendant entries,
  resolve the pre-pass cleanly by keeping the file/symlink entry and marking the base/directory
  descendants handled.
- Final targeted harness: `t3433-rebase-across-mode-change.sh` passed 4/4.
- Related rebase regression harnesses still passed:
  - `t3425-rebase-topology-merges.sh`: 13/13.
  - `t3431-rebase-fork-point.sh`: 26/26.
- Validation:
  - `cargo fmt` passed.
  - `cargo check -p grit-cli` passed with the existing `diff.rs` `ext_total` warning.
  - `cargo clippy --fix --allow-dirty` exited 0 with the repository's existing warning backlog and
    failed unrelated autofix report.
  - `cargo test -p grit-lib --lib` reported the known baseline: 252 passed, 2 ignore glob tests
    failed.
