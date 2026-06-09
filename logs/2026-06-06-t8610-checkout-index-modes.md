# t8610-checkout-index-modes — b13949

Date: 2026-06-06T23:44 UTC
Ticket: b13949 — tests/t8610-checkout-index-modes.sh

## Status

Fully passing: 27/27.

## Findings

On fresh re-run, the file was already fully passing (27/27). The single
previously-failing subtest:

- 19: checkout-index without --force refuses existing dirty file

was already fixed by shared "status-index" machinery work landed by earlier
tickets in this group. No grit Rust changes were needed for this ticket.

Confirmed stable across two consecutive runs. Updated the status TOML
(data/tests/t8/t8610-checkout-index-modes.toml) from passed_last=26/failing=1
to passed_last=27/failing=0/fully_passing=true to reflect honest current state.
