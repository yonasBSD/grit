# t8270-log-author-search

Ticket: c9b28b (# t8270-log-author-search)

## Initial state
- 28/29 passing. One failing subtest: #13 "log --author is case-insensitive by default".

## Failing subtest analysis (differential rule vs git 2.52.0)

Subtest #13 asserted that `git log --author=alice` and `git log --author=ALICE`
produce identical output, claiming `--author` is "case-insensitive by default".

Differential reproduction in /tmp on identical inputs:
- REAL git 2.52.0 (/opt/homebrew/bin/git): `--author=alice` -> 3 matches; `--author=ALICE` -> EMPTY.
- GRIT (target/release/grit): `--author=alice` -> 3 matches; `--author=ALICE` -> EMPTY.

Grit matches real git byte-for-byte. `--author` is case-SENSITIVE by default in
both. Case-insensitive matching only happens with `-i` / `--regexp-ignore-case`:
- REAL git `-i --author=ALICE` -> 3 matches; `-i alice` == `-i ALICE`.
- GRIT  `-i --author=ALICE` -> 3 matches; `-i alice` == `-i ALICE`.

Verdict: TEST BUG (test-authoring contradiction). The test's premise was wrong;
grit equals real git. Sanctioned fix: correct the test to pass `-i`, which is the
flag that actually makes author matching case-insensitive, and retitle to match.

## Fix
Changed subtest to use `git log -i --author=...` for both lower/upper case and
retitled to "log --author with -i is case-insensitive".

## Result
29/29 passing (full file run via scripts/run-tests.sh). fully_passing = true.
