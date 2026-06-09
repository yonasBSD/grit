# t8280-log-committer-search

Date: 2026-06-06T19:32 UTC
Ticket: 1e9200 (t8280-log-committer-search: subtests failing)

## Starting state
28/29 passing. Failing subtest #13 "log --committer is case-insensitive":

```
git log --committer=dana --format="%cn" >lower   # 3 matches (Dana x3)
git log --committer=DANA --format="%cn" >upper    # 0 matches in grit
test_cmp lower upper                               # FAILED
```

## Investigation
- No upstream `git/t/t8280-*.sh` exists; this is a grit-suite synthetic test.
- Real Git 2.52.0: `--committer`/`--author` are case-SENSITIVE by default
  (`-i`/`--regexp-ignore-case` enables case-insensitivity). So real git also
  returns empty for `--committer=DANA`. grit already matched real-git behavior.
- However the grit test SUITE deliberately asserts case-insensitive
  author/committer matching by default: the sibling file
  `tests/t8270-log-author-search.sh` line 154 has the identical
  `'log --author is case-insensitive by default'` as test_expect_success, and
  was also stuck at 28/29.
- Decision: honor the grit suite's chosen semantics — make `--author` and
  `--committer` identity filters case-insensitive by DEFAULT, while keeping
  `--grep` case-sensitive by default (required by t4202-log lines 267/269/332
  which match `--grep=sec` -> "second" but not "Second").

## Fix
`grit/src/commands/log.rs`:
- New helper `ident_pattern_ignore_case(regexp_ignore_case) -> bool` returning
  `true` (author/committer always case-insensitive in grit).
- Both author/committer regex build sites now use it instead of
  `args.regexp_ignore_case`:
  - main path (~line 4946/4952, `build_grep_regex`)
  - --graph path (~line 2892/2905, `RegexBuilder`)
- `--grep` regexes unchanged (still `grep_ignore_case = args.regexp_ignore_case`).
- Updated the explanatory comment that previously claimed all three were
  case-sensitive by default.

## Result
- t8280-log-committer-search: 29/29 PASS
- t8270-log-author-search: 29/29 PASS (shared-machinery fix; was 28/29)
- t4202-log: 119/149 (unchanged from HEAD — no `--grep` regression)
- Manually verified: `--grep=sec` -> "second", `--grep=Sec` -> "Second"
  (case-sensitive preserved); `-i --grep=sec` -> "Second".

## Notes for next agent
The fix is shared machinery in log.rs; it intentionally diverges from upstream
git 2.52.0 (which is case-sensitive) to satisfy the grit test suite's stated
intent across t8270/t8280.
