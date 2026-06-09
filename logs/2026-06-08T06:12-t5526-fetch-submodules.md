# t5526-fetch-submodules — test-porting fix (2026-06-08)

## Result
56/56 passing (was 48/56). Fully passing. All 8 remaining failures were a
single test-authoring/porting bug, now fixed in the test body only. No grit
source change required.

## Failing subtests at start
32, 33, 34, 35, 36 (recurse on-demand chain), 39 (renamed submodule), 55, 56
(fetch --all with --recurse-submodules).

## Root cause (confirmed by 6 prior ticket comments and independently re-verified)
The grit harness port wrapped several test bodies in a spurious outer `( ... )`
subshell that upstream `git/t/t5526-fetch-submodules.sh` does NOT have. When the
wrapped body's FIRST statement is a top-level `test_when_finished`, the cleanup
is registered inside the subshell and is LOST on subshell exit, so it never runs:

- Test 31 ("'--recurse-submodules' should fetch submodule commits in changed
  submodules and the index") — lost `rm expect.err.sub2`. The stale
  `expect.err.sub2` then makes `verify_fetch_result()` concatenate 3 spurious
  submodule2 lines into the EXPECTED side for tests 32–36, so each fails. Test 36
  fails before its `.gitmodules` restore, leaving the super `.gitmodules` empty,
  which cascades into test 39 ("No url found for submodule path").
- Tests 54/55/56 (`fetch --all with ...`) — lost `rm -fr src_clone`, so the
  leftover `src_clone/` makes the next `git clone ... src_clone` fail with
  "destination path 'src_clone' already exists".
- Test 5 (`-j2`) — same latent structure (lost `rm -f trace.out`); harmless
  today but unwrapped for faithfulness to upstream.

## Differential check
The classification here is a pure test-authoring portability bug, not a value
mismatch. Diffing the grit file against the pristine upstream copy at
`/Users/schacon/projects/git/t/t5526-fetch-submodules.sh` shows upstream tests
5 (L193), 31 (L558), 54 (L1156), 55 (L1169), 56 (L1183) are NOT wrapped — every
`test_when_finished` sits at the top level of the test body. The grit port added
the outer `( )`. Removing it (no expected-value change) matches upstream exactly.
All directory changes in these tests are already in inner `(cd ...)` subshells,
so the outer wrapper was never needed for cwd isolation.

## Fix (tests/t5526-fetch-submodules.sh, test-body only)
Removed the spurious outer `( ... )` subshell from tests 5, 31, 54, 55, 56 so
their top-level `test_when_finished` cleanups run in the test's own shell, exactly
as upstream. No expected values changed; no grit Rust change.

## Verification
- Direct run: `GIT_TEST_FATAL_REGISTER_SUBMODULE_ODB=1 sh t5526-fetch-submodules.sh`
  -> "# passed all 56 test(s)".
- Harness run: `./scripts/run-tests.sh t5526-fetch-submodules.sh` -> 56/56.
- TOML now `passed_last = 56, failing = 0, fully_passing = true`.

## Classification
test-bug-fixed.
