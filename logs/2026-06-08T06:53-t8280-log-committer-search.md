# t8280-log-committer-search

Ticket: d4bacc (created + claimed)

## Starting state
28/29 passing. Failing subtest: #13 "log --committer is case-insensitive".

## Failing subtest analysis
The subtest asserted that `--committer` matching is case-insensitive by default:

```sh
git log --committer=dana --format="%cn" >lower &&
git log --committer=DANA --format="%cn" >upper &&
test_cmp lower upper
```

### Differential vs git 2.52.0 (/opt/homebrew/bin/git)
Built identical 6-commit repos with both grit (target/release/grit) and real git, ran both invocations:

- `--committer=dana` (lowercase): BOTH produce 3 matches (Dana Deploy, Dana Developer, Dana Developer).
- `--committer=DANA` (uppercase): BOTH produce EMPTY output.

So `--committer` is case-SENSITIVE by default in real git 2.52.0, and grit matches byte-for-byte.
This is a TEST-AUTHORING bug: case-insensitive grep requires the `-i` / `--regexp-ignore-case`
flag. Verified that with `-i`, BOTH real git and grit make lower == upper (3 matches each).

## Fix (test-body only, differential-verified)
Added `-i` to both invocations (and renamed the subtest title to match), which is the standard
git idiom for case-insensitive header grep. grit == real git for `-i --committer` too.

```sh
git log -i --committer=dana --format="%cn" >lower &&
git log -i --committer=DANA --format="%cn" >upper &&
test_cmp lower upper
```

## Result
29/29 passing. fully_passing = true. Classification: test-bug-fixed.
