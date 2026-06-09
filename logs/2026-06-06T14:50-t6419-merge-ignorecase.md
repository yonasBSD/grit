# t6419-merge-ignorecase — work log (2026-06-06)

Ticket: 6c16f3 — tests/t6419-merge-ignorecase.sh
Status group: merge-ort (thread C)

## Summary

The file self-skips on this platform because the harness `tests/test-lib.sh` does
**not** define the standard upstream `CASE_INSENSITIVE_FS` lazy prereq. The file's
guard (`if ! test_have_prereq CASE_INSENSITIVE_FS; then skip_all=...; test_done; fi`)
therefore always fires → `1..0 # SKIP`.

I diagnosed the full chain and proved grit's merge machinery already handles
case-changing renames correctly, but landing a working version regresses several
currently-green files because of a tight coupling described below. Per the no-regression
and single-file scope rules I reverted my source changes and left the file at its
honest self-skip (0/0) baseline, with findings here and on the ticket for a future
cross-cutting case-insensitivity effort.

## Root cause chain (3 coupled issues)

1. **Harness gap.** `tests/test-lib.sh` defines lazy prereqs ICONV, TIME_*, UTF8_NFD_TO_NFC,
   but NOT `CASE_INSENSITIVE_FS` (upstream git/t/test-lib.sh:1771 defines it). Without it,
   `test_have_prereq CASE_INSENSITIVE_FS` falls through to the default case and returns
   missing. The probe itself works when run standalone (this FS IS case-insensitive APFS):
   `echo good >CamelCase && echo bad >camelcase && test "$(cat CamelCase)" != good` → 0.

2. **grit init bug (real, fixable).** `git init` does not detect a case-insensitive FS and
   never writes `core.ignorecase = true`. Upstream git/setup.c:2580-2583 probes by
   `access("CoNfIg")` after writing the `config` file. t6419 subtest 1 asserts
   `test $(git config core.ignorecase) = true`, so this is required.
   The fix is small and matches upstream — see "Proposed fix" below.

3. **Cross-file coupling (the blocker).** Enabling `CASE_INSENSITIVE_FS` globally (required —
   there is no per-file prereq mechanism, and editing the test's guard is not allowed) makes
   previously-skipped subtests in OTHER files run. Some need unimplemented grit features and
   regress currently-green files:
     - t0050-filesystem test 8 `add directory (with different case)` — needs case-insensitive
       directory matching in `git add` index insertion (`git add dir1/DIR2/b` → `dir1/dir2/b`).
     - t0050-filesystem test 13 `checkout with no pathspec and a case insensitive fs`.
     - t0003-attributes test `additional case insensitivity tests`.
     - t2081-parallel-checkout-collisions several tests.
   Additionally, the grit init fix in isolation (without the prereq) breaks t0050 test 2,
   the `!CASE_INSENSITIVE_FS` variant which asserts `core.ignorecase` is unset — because the
   FS really is case-insensitive but the harness still believes it is not.

   So: init fix REQUIRES the prereq; the prereq REQUIRES case-insensitive add/checkout/attrs.
   That is a multi-subsystem effort well beyond this single file.

## What I verified WORKS (with prereq forced on, locally)

With both the init fix and the prereq present, t6419 passes 2/2:
  ok 1 - merge with case-changing rename
  ok 2 - merge with case-changing rename on both sides
grit's merge-ort already resolves the case-changing rename and the index ends with
`testcase` (correct). So the merge logic is NOT the blocker.

## Proposed fix (for the cross-cutting case-insensitivity ticket)

1. `grit-lib/src/unicode_normalization.rs` — add a probe:
   ```rust
   pub fn probe_filesystem_is_case_insensitive(git_dir: &Path) -> bool {
       if !git_dir.join("config").exists() { return false; }
       git_dir.join("CoNfIg").exists()
   }
   ```
2. `grit/src/commands/init.rs` — after the config file is written, on fresh non-bare init,
   when `core.ignorecase` is not already set in higher-priority config and the probe is true,
   set `core.ignorecase = true` (mirror the existing precomposeunicode block).
3. `tests/test-lib.sh` — add the standard upstream `CASE_INSENSITIVE_FS` lazy prereq.
4. THEN implement case-insensitive `git add` (directory + leaf), checkout, and attribute
   lookup so t0050/t0003/t2081 stay green, before committing the prereq enablement.

## Decision

Left the ticket OPEN. No commit (any landing combination either leaves t6419 skipping or
regresses t0050/t0003/t2081). Reverted my local edits to init.rs, unicode_normalization.rs,
and test-lib.sh. The file's honest state is unchanged: 0/0 self-skip.
