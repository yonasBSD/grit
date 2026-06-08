# t12040-config-unset-all — work log (2026-06-08)

Ticket: 78b39a (created this run; no prior ticket, none closed).
Subsystem group: mop-up, thread D.

## Baseline
- Built: `cargo build --release -p grit-cli -j 4` (only pre-existing repack.rs warnings).
- Run: `./scripts/run-tests.sh t12040-config-unset-all.sh` -> 32/33.
- Only failing subtest: 27 "config -z uses NUL delimiters".

## Failing subtest 27
```
(cd repo && grit config -z --list >../actual) &&
tr "\0" "\n" <actual >actual_lines &&
grep "user.email=t@t.com" actual_lines
```

## Diagnosis — TEST BUG, not grit bug
git/Documentation/git-config.adoc, `-z`/`--null`:
  "always end values with the null character (instead of a newline).
   Use newline instead as a delimiter between key and value."

So `-z` record format is `key\nvalue\0`, NOT `key=value\0`. After
`tr "\0" "\n"`, key and value are on SEPARATE lines, so the grep for
`user.email=t@t.com` (with an `=`) can never match.

### Proof against real git
With isolated HOME so only local config is read:
```
/usr/bin/git config -z --local --list | tr '\0' '\n' | grep 'user.email=t@t.com'
# -> exit 1 (no match)
```
Real git emits:
```
user.email
t@t.com
```
i.e. separate lines. The grep fails against real git too.

### Grit matches git
`grit config -z --list` output is byte-identical to git's `key\nvalue\0`.
Confirmed: `grit config -z --list | tr '\0' '\n' | grep -A1 '^user.email$'`
yields `user.email` then `t@t.com`.

## Conclusion
The assertion is inconsistent with git's documented `-z` format. Making grit
pass it would require emitting `key=value\0`, violating the documented contract
and breaking secure parsing of values containing newlines. Per agent rules I do
not modify the test (only permitted edit is expect_failure -> expect_success,
which does not apply). Marking ticket blocked with proof.

Result: 32/33, blocked on subtest 27 (test/authoring bug).
