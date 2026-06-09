# t1092-sparse-checkout-compatibility.sh — ticket 03ecca

Date: 2026-06-08
Agent: schacon+opus-t5@gmail.com (thread B, mop-up)

## Starting state
- Ticket 03ecca reported a REGRESSION: previously closed at 106/106 (ticket
  0e0b0d, 2026-06-05), then regressed to 104/106 with two stable failures:
  - 11: checkout with modified sparse directory
  - 18: diff with renames and conflicts
- TOML `data/tests/t1/t1092-sparse-checkout-compatibility.toml` recorded
  passed_last = 104, failing = 2 (description) / 105,1 at claim time.

## Investigation
- Claimed ticket, set in-progress, read all comments on the closed predecessor
  ticket 0e0b0d (full history of the 106/106 build-up).
- Rebuilt the release binary fresh with the canonical command:
  `cargo build --release -p grit-cli -j 4` (succeeded; only pre-existing
  unused_mut warnings in repack.rs, not mine).
- Ran `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh`.

## Result
- Fresh run: 106/106. Re-ran once more for stability: 106/106 again.
- I made NO grit Rust source changes. The regression was caused by a STALE
  `tests/grit` binary (the runner copies target/release/grit at run start, and
  a concurrent agent had swapped in an older binary mid-run, per the known
  harness caveat). Rebuilding the current source produced a binary that passes
  subtests 11 and 18 cleanly. The underlying source was already correct.
- `cargo test -p grit-lib --lib`: 276 passed, 2 failed — only the two known
  pre-existing `ignore::gitignore_glob_tests` failures (not ignore-related to
  this ticket).

## Files
- Only my own change: `data/tests/t1/t1092-sparse-checkout-compatibility.toml`
  updated by the runner to 106/106 fully_passing = true.
- All other dirty files in the tree belong to concurrent agents and were left
  untouched.
