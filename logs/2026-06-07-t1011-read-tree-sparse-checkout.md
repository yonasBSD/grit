# t1011-read-tree-sparse-checkout (ticket f92898)

## Goal
Make `tests/t1011-read-tree-sparse-checkout.sh` fully pass. Started at 22/23.

## Failing subtest
- not ok 21 - print warnings when some worktree updates disabled

The test does `git checkout -q top 2>actual` while on a detached HEAD (`init`).
With `-q`/`--quiet`, git suppresses both "Previous HEAD position was ..." and
"HEAD is now at ..." messages. grit still emitted
`Previous HEAD position was 2519212 init` on stderr, so the captured `actual`
had an extra trailing line vs `expected`.

## Root cause
In `grit/src/commands/checkout.rs`, `print_detached_checkout_leave_message`
emitted its lines via raw `eprintln!`, bypassing the `QUIET` thread-local that
`run()` sets from `args.quiet`. The sibling function
`print_detached_head_message_inner` correctly used the `checkout_eprintln!`
macro (which checks `QUIET`), but the "Previous HEAD position was ..." /
"Warning: you are leaving N commits behind ..." branch did not.

## Fix
Switched all `eprintln!` calls inside `print_detached_checkout_leave_message`
to `checkout_eprintln!` so the leave message honors `-q`/`--quiet`. The macro
is defined earlier in the same file (line ~450) and is in scope.

## Result
`./scripts/run-tests.sh t1011-read-tree-sparse-checkout.sh` -> 23/23, fully passing.
`cargo test -p grit-lib --lib`: 276 pass, only the 2 known pre-existing
`ignore::gitignore_glob_tests` failures remain (unrelated to this ticket).
