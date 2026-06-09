# t5526-fetch-submodules — mop-up round 2 (session 2026-06-07, agent t5)

Ticket: 1d762d. File: tests/t5526-fetch-submodules.sh.

## Starting state
- Fresh re-run with current binary: **48/56** (8 failing: 32,33,34,35,36,39,55,56).
- Prior 5 agents (commits af9e2e4b2 … 68d9c1abe) drove this from 19 → 48 and
  ROOT-CAUSED every remaining failure. The lone genuine grit gap (#38, broken
  repository) was fixed by the previous agent in commit 68d9c1abe.

## What I did this session
1. Rebuilt `grit` release (`cargo build --release -p grit-cli -j4`) — clean
   (only pre-existing repack.rs `unused_mut` warnings, not mine).
2. Re-ran the file fresh: confirmed **48/56**.
3. INDEPENDENTLY RE-VERIFIED the root cause of the 8 remaining failures, end to end,
   with the *current* binary — did not just trust prior notes.

## Root cause (confirmed, not a grit bug)
The 8 remaining failures are a **test-file porting artifact**, not a grit defect.
`scripts/_wrap_cd_subshell.py` wrapped the bodies of tests 31, 54, 55, 56 in an
outer `( … )` subshell that upstream `git/t/t5526-fetch-submodules.sh` does NOT have:

- Test 31 (grit L594) vs upstream L558: upstream body is unwrapped, so its
  top-level `test_when_finished "rm expect.err.sub2"` runs. In the grit port the
  `test_when_finished` is registered *inside* the subshell and is lost on subshell
  exit → stale `expect.err.sub2` → `verify_fetch_result` adds spurious submodule2
  lines to the EXPECTED side for tests 32–36. Test 36 fails before its `.gitmodules`
  restore (super `.gitmodules` left empty), cascading to test 39.
- Tests 54/55/56 (grit L1287/1302/1318) vs upstream L1225/1238/1252: upstream
  bodies are unwrapped, so their top-level `test_when_finished "rm -fr src_clone"`
  runs. In the grit port that cleanup is lost → `src_clone` persists → tests 55/56
  fail with `destination path 'src_clone' already exists`.

## Proof the grit code is correct (56/56 when test matches upstream)
- Built `/tmp/t5526-unwrap.sh`: a copy of the real file with ONLY the spurious
  outer `( )` removed from tests 31, 54, 55, 56 (so they match upstream exactly).
- Ran it through the real harness with the CURRENT binary:
  `# passed all 56 test(s)` / `1..56`.
- (First unwrap attempt of mine had a script bug — a non-unique `.replace()`
  anchor clobbered test 2's closing paren, producing a shell `unexpected end of
  file`. Fixed the anchor to the unique `EOF`-heredoc terminator; clean 56/56.)
- Removed the temp file and re-pruned the catalog; real `data/tests` left at 1605.

## Conclusion / outcome
- **No grit Rust change is warranted.** grit's fetch-recursion behavior already
  matches git for every t5526 subtest. Making grit tolerate the stale `expect.err.sub2`
  or a pre-existing `src_clone` would be *wrong* (it would mask the exact behavior
  the test checks).
- The only fix is removing the spurious outer `( )` on tests 31, 54, 55, 56. That is
  a test-file edit outside my allowed scope (rule: the ONLY allowed test edit is
  `test_expect_failure` → `test_expect_success`). Five prior agents agreed; I did not
  override the rule.
- File stays at **48/56**. Ticket remains OPEN.

## MAINTAINER ACTION
Remove the outer `( )` wrapper on tests 31, 54, 55, 56 in
`tests/t5526-fetch-submodules.sh` (match upstream `git/t` L558 and L1225/1238/1252).
With the current grit binary this recovers tests 32–36, 39, 55, 56 → **56/56**.
