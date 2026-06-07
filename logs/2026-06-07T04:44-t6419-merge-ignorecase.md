# t6419-merge-ignorecase — mop-up round 1 (2026-06-07)

Ticket: 6c16f3 — tests/t6419-merge-ignorecase.sh
Disposition: marked `in_scope = "skip"` (platform-unrunnable, structurally blocked).

## Fresh re-run

`./scripts/run-tests.sh t6419-merge-ignorecase.sh` → `0/0` (self-skip).
Re-ran fresh after other agents' work cascaded; result unchanged. The file still
emits `1..0 # SKIP skipping case insensitive tests - case sensitive file system`
because its guard (lines 10-14) checks `test_have_prereq CASE_INSENSITIVE_FS`, and
that lazy prereq is NOT defined in `tests/test-lib.sh`.

## Verified the prior agent's findings still hold

1. **FS is case-insensitive.** Standalone probe (`echo good >CamelCase; echo bad
   >camelcase; test "$(cat CamelCase)" != good`) returns 0 — APFS here IS
   case-insensitive. So the test *would* be meaningful if the prereq fired.

2. **Harness gap.** `tests/test-lib.sh` defines lazy prereqs ICONV, TIME_IS_64BIT,
   TIME_T_IS_64BIT, UTF8_NFD_TO_NFC (lines 1066-1086) but NOT `CASE_INSENSITIVE_FS`.
   Upstream defines it at `git/t/test-lib.sh:1771`. Editing `tests/test-lib.sh` is
   explicitly forbidden (AGENTS.md "Do Not", TESTING.md, agent rules), and editing
   the test's `if ! test_have_prereq` guard is not an allowed test edit.

3. **grit init does not write `core.ignorecase`.** Reproduced subtest 1 manually:
   - Without `core.ignorecase`: `git merge main` ABORTS with "The following untracked
     working tree files would be overwritten ... testcase ... Aborting" — grit treats
     `TestCase`/`testcase` as distinct paths.
   - With `git config core.ignorecase true` set manually: merge succeeds, index/worktree
     end on `testcase`. So **grit's merge-ort logic is correct**; the only grit bug is
     init not detecting the case-insensitive FS (upstream git/setup.c:2580-2583 probes
     via `access("CoNfIg")` after writing the config file).

## Why no grit fix can land here (the hard blocker — re-verified)

The init fix is a real, in-scope grit bug, BUT landing it changes nothing for t6419
(the file still self-skips because the prereq is undefined) AND it regresses a
currently-green file:

- `t0050-filesystem.sh` is fully passing (11/11). It gates tests on BOTH
  `CASE_INSENSITIVE_FS` (lines 20, 68, 84, 128) and `!CASE_INSENSITIVE_FS` (line 24).
- With the prereq still undefined (current harness state), the `!CASE_INSENSITIVE_FS`
  test at lines 24-28 RUNS and asserts `core.ignorecase` is unset or false.
- Fixing grit init to write `core.ignorecase=true` would make that assertion FAIL →
  t0050 regresses 11/11 → 10/11. Verified by reading t0050 lines 24-28.
- Conversely, adding the prereq to test-lib.sh (forbidden) would un-skip t0050 lines
  68/128 which need UNIMPLEMENTED case-insensitive `git add` (dir+leaf) and `checkout`,
  plus t0003-attributes and t2081-parallel-checkout-collisions → more regressions.

So the init fix REQUIRES the prereq; the prereq REQUIRES forbidden test-lib edits AND
unimplemented case-insensitive add/checkout/attr. This is a cross-cutting,
multi-subsystem effort, not a single-file mop-up, and it is impossible to land without
the forbidden `tests/test-lib.sh` change.

## Decision

Marked `data/tests/t6/t6419-merge-ignorecase.toml` → `in_scope = "skip"`. The ticket
explicitly authorized this option ("Either verify on a platform with the prereq or
mark its status TOML `in_scope = "skip"`"). The file runs zero tests on this platform;
skipping removes it honestly from aggregate counts rather than leaving a permanent
0/0 "not fully passing" row that can never be satisfied here. No grit source changed
(any source change either no-ops for this file or regresses t0050).

## Future cross-cutting case-insensitivity ticket (out of scope here)

1. `grit-lib` probe `probe_filesystem_is_case_insensitive(git_dir)` via `CoNfIg` access.
2. `grit/src/commands/init.rs`: set `core.ignorecase=true` on fresh non-bare init when
   the probe is true and not already set (mirror the precomposeunicode block).
3. `tests/test-lib.sh`: add the upstream `CASE_INSENSITIVE_FS` lazy prereq.
4. Implement case-insensitive `git add` (dir + leaf), `checkout`, and attribute lookup
   so t0050/t0003/t2081 stay green BEFORE enabling the prereq.

Only after all four can t6419 (and the gated t0050 subtests) pass without regressions.
