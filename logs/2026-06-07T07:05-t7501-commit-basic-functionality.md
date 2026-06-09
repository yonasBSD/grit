# t7501-commit-basic-functionality — MOP-UP ROUND 1

Ticket: 2cab681c-bd2a-43f8-88ae-f230b161c4c8 (created; no prior ticket existed).
Date: 2026-06-07

## Starting state
76/77 passing, 1 failing.

## Diagnosis

The actual failing subtest under the official harness (`scripts/run-tests.sh`,
which does `env -u GIT_EDITOR …`) is **#76 `--dry-run with conflicts fixed from a
merge`**, NOT #36 `--amend --edit`.

Initial confusion: a direct `sh t7501…sh` invocation showed #36 failing, but that
was an artifact of my interactive shell exporting `GIT_EDITOR=true` (from my
profile), which has highest priority in editor resolution and silenced the test's
inline `EDITOR=./editor`. Our ported `tests/test-lib.sh` (unlike upstream) does NOT
unset inherited `GIT_*` env vars, so `GIT_EDITOR` leaks in a raw `sh` run. The
official runner unsets `GIT_EDITOR`, so #36 always passed there — #76 was the real
failure. (test-lib.sh is not allowed to be modified and this is an env-leak, not a
grit bug — left alone.)

### Subtest 76 failure

`git checkout -b branch-2 HEAD^1` failed with:
`error: refusing to replace populated submodule at '.' with directory content`

Root cause in `grit/src/commands/checkout.rs`,
`refuse_populated_submodule_tree_replacement_inner` (defensive heuristic block).
It walks each path component of every target-index entry to detect a populated
submodule prefix. An index entry whose path is stored as `./-` (the dash file added
via `git add ./-` in subtest 73 "commit a file whose name is a dash", carried in the
`HEAD^1` tree) splits into `[".", "-"]`. The `.` component pushed the work-tree root
into `prefix`; `work_tree/.` ` /.git` is the repo's OWN `.git`, which always exists,
so the root was wrongly flagged as a populated submodule and checkout aborted (128).

## Fix

`grit/src/commands/checkout.rs`: in the component walk, skip `.` and `..` segments
(in addition to empty), exactly like empty components. A `.`/`..` segment can never
be a submodule directory name and `.` resolves to the work-tree root.

```rust
if component.is_empty() || component == "." || component == ".." {
    continue;
}
```

## Result
- t7501: **77/77**, fully passing.
- Regression checks (isolated `--data-dir`): t2013-checkout-submodule 70/70 pass
  (74 incl. expect_failure), t6041-bisect-submodule 14/14, t7400-submodule-basic
  124/124 — all unchanged.
- `cargo test -p grit-lib --lib`: only the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures.
- No new clippy warnings in edited file; rustfmt clean.
