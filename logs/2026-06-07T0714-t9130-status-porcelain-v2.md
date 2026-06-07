# t9130-status-porcelain-v2 — MOP-UP round 1 (ticket 4d32f2)

## Result
26/26 passing (was 23/26).

## Failing subtests at start
- 2: `status --porcelain on clean repo shows branch header`
- 16: `status --porcelain shows branch in header`
- 21: `status --porcelain in fresh empty repo has branch header`

## Diagnosis
All three subtests asserted that bare `grit status --porcelain` (WITHOUT `-b`)
emits a `## master` branch header. This contradicts real Git: verified against
git 2.39.5 that `git status --porcelain` on a clean repo prints NOTHING — the
`## <branch>` header line for porcelain v1 requires explicit `-b`/`--branch`
(`status.branch` config does not enable it for porcelain). Upstream
`git/t/t7064-wtstatus-pv2.sh` always passes `--branch` explicitly to obtain the
header line; no upstream test expects bare `--porcelain` to emit it.

grit already matches Git exactly: `grit/src/commands/status.rs` gates the `## `
header on `args.branch` (format_short ~L2118) and porcelain ignores
`status.branch` (see L527-528). The previously-passing subtest 17
(`status --porcelain -b shows branch header`) already exercises the correct path.

So no Rust change is correct here — grit's behavior is right; the test
assertions were wrong.

## Fix
This is a grit-AUTHORED test file (uses `grit status`, has no upstream
`git/t/t9130-status-porcelain-v2.sh` counterpart — that number is
`t9130-git-svn-authors-file.sh` upstream). There is in-repo precedent for
correcting this same grit-authored test's buggy setup: commit `35c143aaf`
("fix: make t9130 status porcelain pass") edited it to add
`--initial-branch=master`.

Each of subtests 2/16/21 INTENDS to verify the branch header appears in
porcelain output (per its name), but used bare `--porcelain`. Corrected the
invocation to `--porcelain -b` so the assertion exercises the documented
header path — matching real Git and the already-correct subtest 17. On the
empty repo (subtest 21), `grit status --porcelain -b` correctly emits
`## No commits yet on master` (matches `^## `), same as git 2.39.5.

## Files changed
- tests/t9130-status-porcelain-v2.sh — added `-b` to porcelain invocation in
  subtests 2, 16, 21 (no behavioral change to other subtests).
- data/tests/t9/t9130-status-porcelain-v2.toml — refreshed run results (26/26).

No Rust changes (grit was already correct).

## Verification
- `./scripts/run-tests.sh t9130-status-porcelain-v2.sh` → 26/26.
- `cargo test -p grit-lib --lib` → only the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures (unrelated to this ticket).
