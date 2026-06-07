# t6434-merge-with-no-common-ancestor — MOP-UP ROUND 2 re-verify (2026-06-07)

Ticket: 1e3f8b. Branch: grit-t5-progress.

## Fresh run result
`./scripts/run-tests.sh t6434-merge-with-no-common-ancestor.sh` → **1/3** (tests 2 & 3 fail).
Built release binary fresh (`cargo build --release -p grit-cli -j 4`) before running. No cascade
from other agents' fixes changed the outcome.

## Root cause (confirmed, identical to two prior agents)
This is the **cwd-persistence harness pitfall** documented in TESTING.md ("the `cd repo` trap"), NOT a
grit bug. The ported test file `tests/t6434-merge-with-no-common-ancestor.sh`:

- Test 1 (`setup two diverged branches`) runs `git init ancestor-test && cd ancestor-test && …`
  with a **bare `cd` (no subshell)**, so the harness shell is left **inside** `ancestor-test/`.
- Tests 2 and 3 each begin with a bare `cd ancestor-test`. Because the shell is already inside
  `ancestor-test/`, that `cd` fails with `cd: ancestor-test: No such file or directory`
  (test-lib.sh), and the block aborts **before any grit command runs**.

TAP confirms tests 2 & 3 die at the leading `cd ancestor-test` line; no grit invocation is reached.

## grit verified fully correct (manual repro in /tmp)
Reproduced the exact test sequence by hand with the release binary:
- `grit checkout left` + `grit merge right -m "merge right"` → **exit 0**, "Merge made by the 'ort'
  strategy", and `base`, `left-file`, `right-file` all present on disk (satisfies all three
  `test_path_is_file` assertions of test 2).
- `grit merge-base left-tip right-tip` == `grit rev-parse base` (byte-identical), satisfying test 3.

So both failing subtests would pass if they ever reached grit. grit needs no change.

## Disposition: NO grit change possible
The only fix is a **test-file edit** — wrapping the cd-using bodies in subshells
(`scripts/_wrap_cd_subshell.py`). That is forbidden by the ticket hard rule ("Do NOT modify test
files — the ONLY allowed test edit is flipping test_expect_failure -> test_expect_success for a bug
you actually fixed"). The wrap was denied for two prior agents, and the wrapper is independently
risky (it broke t5526's `test_when_finished`, commit fe250ebf7). Same disposition as sibling
cwd-trap tickets: t6432 (98a5997f5 / fd8afd7d9), t6435 (c9c4dcb6a / fc7254e8b), and prior t6434
passes (98a5997f5, 91831aefe).

Leaving ticket OPEN / blocked for a human or an agent explicitly permitted to apply the subshell
wrapper to the test file. No Rust change made.
