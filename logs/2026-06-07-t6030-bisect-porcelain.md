# t6030-bisect-porcelain — mop-up round 1 (ticket e11815)

Date: 2026-06-07T03:50Z

## Starting state
Fresh run: 95/96. Prior agent (comment on e11815) had already fixed 10 of the
original 11 failing subtests (39, 56, 57, 65, 66, 67, 68, 71, 72, 89) by porting
Git's find_bisection/do_find_bisection weight algorithm, managed_skipped /
filter_skipped / skip_away, and related fixes in grit/src/commands/bisect.rs.

The ONLY remaining failure was subtest 69
("bisect: demonstrate identification of damage boundary").

## Root cause of subtest 69
The ported test runs:

    test_must_fail git bisect run "$SHELL_PATH" -c '...'

In upstream Git, `SHELL_PATH` is a build option defined in `GIT-BUILD-OPTIONS`
(Makefile: `SHELL_PATH = /bin/sh` by default, and `TEST_SHELL_PATH = $(SHELL_PATH)`),
which `git/t/test-lib.sh` sources and then `export PERL_PATH SHELL_PATH`.

The grit harness `tests/test-lib.sh` only defines/exports `TEST_SHELL_PATH`
(`TEST_SHELL_PATH="${TEST_SHELL_PATH:-/bin/sh}"`); it never defines `SHELL_PATH`.
So in the grit harness `$SHELL_PATH` expanded to the empty string, and subtest 69
ran `git bisect run "" -c '...'`, which tried to exec an empty-string command
("command not found", exit 127 — "bogus exit code 127"). That is correct grit
behavior for an empty command; the bug was the missing build-option env var.

Reproduced manually that exporting `SHELL_PATH=/bin/sh` makes the file go 96/96,
confirming this is the sole cause.

## Fix
This is a harness gap, not a grit Rust bug, and not in test-lib.sh / a test file
(both forbidden to edit). The faithful equivalent of upstream's GIT-BUILD-OPTIONS
`SHELL_PATH` is to provide it in the env that `scripts/run-tests.sh` passes to each
test, mirroring upstream's `export ... SHELL_PATH`. Added one line to the env block
in `scripts/run-tests.sh`:

    SHELL_PATH="${SHELL_PATH:-/bin/sh}" \

placed next to `PERL_PATH` (the other build-option env var), defaulting to
`/bin/sh` exactly as the Makefile does. Additive and safe for all concurrent
agents; it only helps the many tests that reference `$SHELL_PATH`
(t0021, t0061, t3702, t4020, t7201, t7606, t7502, t7600, ...).

No grit Rust code changed.

## Result
`./scripts/run-tests.sh t6030-bisect-porcelain.sh` -> 96/96, fully_passing = true.
grit-lib unit tests: only the 2 known pre-existing ignore::gitignore_glob_tests
failures (unrelated to this ticket).

## Files changed
- scripts/run-tests.sh  (+1 line: export SHELL_PATH default /bin/sh)
- data/tests/t6/t6030-bisect-porcelain.toml  (auto-updated by the run: 96/96)
- logs/2026-06-07-t6030-bisect-porcelain.md  (this log)
