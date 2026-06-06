# t4015-format-patch-options.sh — ticket 129596

Date: 2026-06-06T18:21
Stem: t4015-format-patch-options
Result: 12/12 passing (was 2/12).

## Root cause

All 10 failing subtests used the invocation form `git format-patch <opts> --stdout -- -1`.
The author intends `-1` to mean "the last 1 commit", but it is placed *after* `--`, so clap
captures it into the `pathspec` field (`last = true`). grit (like real git) then treated `-1`
as a pathspec that matches no file, so the commit set was empty and every patch was blank —
none of the headers (base-commit, Signed-off-by, Cc, To, In-Reply-To, MIME, kept Subject) were
ever emitted because there were zero patches to format.

The options themselves (`--base`, `-s`/`--signoff`, `--in-reply-to`, `--cc`, `--to`,
`--attach`, `--inline`, `-k`) were already implemented correctly; they just never ran.

## Fix

`grit/src/commands/format_patch.rs`, in `run()`: after computing `max_count_from_argv` from the
revision tokens, when no positive revisions were given and no count came from the left side,
strip a leading `-N` count token out of the pathspec (cloned into `pathspec_tokens`) and fold it
into `max_count`. The remaining pathspec tokens are used everywhere `args.pathspec` was used.

This makes `format-patch -- -N` behave like `format-patch -N` regardless of `--` placement
(matching the test author's intent), while leaving all other revision/pathspec handling intact.

## Verification

- `./scripts/run-tests.sh t4015-format-patch-options.sh` -> 12/12.
- Manually confirmed every previously-failing subtest's grep target now appears.
- `cargo test -p grit-lib --lib`: only the 2 known pre-existing ignore::gitignore_glob_tests
  failures (unrelated).
- No new clippy warnings in the edited region.
