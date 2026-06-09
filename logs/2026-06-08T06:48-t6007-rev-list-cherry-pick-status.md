# t6007-rev-list-cherry-pick-status.sh — mop-up (test 7 fixed)

Date (UTC): 2026-06-08T06:48
Ticket: 104846
Result: 29/29 PASSING (was 28/29).

## Failing subtest (before fix)
Test 7: 'rev-list --left-right count with --count' (file lines 82-86)

```
git rev-list --left-right --count left...right >actual &&
echo "2\t2\t0" >expect &&
test_cmp expect actual
```

The test expected THREE tab-separated columns `2\t2\t0` from
`rev-list --left-right --count` WITHOUT `--cherry-mark`.

## Differential verification (vs real git 2.52.0 at /opt/homebrew/bin/git)
Reproduced the setup in /tmp with BOTH the harness grit
(target/release/grit) and /opt/homebrew/bin/git on identical inputs.

`rev-list --left-right --count left...right` (od -c, byte-for-byte):
- real git 2.52.0:  `2 \t 2 \n`   (2 columns)
- grit:             `2 \t 2 \n`   (2 columns)  -> IDENTICAL

`rev-list --left-right --count --cherry-mark left...right`:
- real git 2.52.0:  `2 \t 2 \t 0 \n`  (3 columns)
- grit:             `2 \t 2 \t 0 \n`  (3 columns)  -> IDENTICAL

The 3rd column (count_same) is gated on `left_right && cherry_mark`
(upstream git/builtin/rev-list.c ~lines 955-963). The test omits
`--cherry-mark`, so 2 columns is the correct, upstream-faithful output.

## Verdict: TEST-AUTHORING BUG (not a grit bug)
grit matches real git 2.52.0 byte-for-byte for both the with- and
without-`--cherry-mark` forms. The prior agent (ticket comments 1-4)
correctly diagnosed the root cause but treated test edits as disallowed
and left it BLOCKED. This task EXPLICITLY AUTHORIZES editing the test
body, so the fix is sanctioned.

## Fix applied (test body only)
Changed the expected value on line 84 from `2\t2\t0` to `2\t2`
(the value both real git and grit actually emit), with an explanatory
comment. This aligns with the upstream-faithful sibling
t6007-rev-list-cherry-pick-file.sh test 21 ('--count --left-right'
expects 2-col `1\t2`) and with t5310, which also assume 2-col output.

No grit source changes (grit was already upstream-correct).

Final run (plain full file): `./scripts/run-tests.sh t6007-rev-list-cherry-pick-status.sh`
-> 29/29.
