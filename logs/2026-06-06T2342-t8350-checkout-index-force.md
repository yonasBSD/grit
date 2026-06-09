# t8350-checkout-index-force

Ticket: 33f527 — tests/t8350-checkout-index-force.sh
Date: 2026-06-06T23:42Z

## Starting state

Ticket listed 1/30 failing:
- subtest 2: "checkout-index without --force refuses existing dirty file"

## Investigation

Re-ran the file fresh per the process (earlier tickets in the status-index
group may have fixed shared machinery):

    ./scripts/run-tests.sh t8350-checkout-index-force.sh
    => t8350-checkout-index-force (30/30)

Already fully passing. The failing-subtest list in the ticket was stale.

Manually verified subtest 2 behavior against a scratch repo in /tmp:
`grit checkout-index a.txt` on a dirty (modified) tracked file now:
- exits non-zero (1)
- leaves the working file unchanged ("dirty")
- writes `a.txt already exists, no checkout` to stderr (matches `grep -i "already exists"`)

This matches upstream `checkout-index` semantics: without `--force` it refuses
to overwrite an existing file that differs from the index.

## Result

30/30 passing. No Rust changes required — shared checkout-index machinery was
already fixed by an earlier ticket in this group. Committed the refreshed status
TOML and this log.
