# t8650-cat-file-batch-extra — test-body portability fix

Ticket: 86410b

## Failing subtest at start
- 26/27 passing. Only subtest 4 'cat-file --batch content matches cat-file -p' failed.

## Root cause (TEST bug, not grit)
The body did:
```
tail -n +2 batch_out | head -n -1 >actual
```
`head -n -1` (print all but last N lines) is a GNU coreutils extension; macOS
BSD `head` rejects it: `head: illegal line count -- -1`. Pure portability bug
in the test body.

## Differential verification vs git 2.52.0 (/opt/homebrew/bin/git)
`echo "$blob" | git cat-file --batch` output is byte-for-byte IDENTICAL between
grit (target/release/grit) and real git: header `<oid> blob 8\n`, content
`updated\n`, then a trailing `\n` that --batch appends. So `tail -n +2` drops
the header leaving `updated\n\n`, and the final line must be stripped to match
`cat-file -p`. grit == real git, so this is a sanctioned test-body fix.

## Fix
Replaced the non-portable `head -n -1` with portable `sed "\$d"` (delete last
line). Verified the replacement produces a passing `test_cmp` under BOTH grit
and real git on identical inputs.

## Result
27/27 passing via `./scripts/run-tests.sh t8650-cat-file-batch-extra.sh`.

Classification: test-bug-fixed.
