# t4067-diff-partial-clone — verification (no code change required)

Date: 2026-05-29
Branch: wf/p2/t4067-diff-partial-clone
Base: 7a7844cb7

## Task premise
The orchestrator flagged `tests/t4067-diff-partial-clone.sh` as "CURRENTLY TIMES OUT (hangs)" and
asked to first fix the hang/infinite-loop, then make the subtests pass.

## Finding
The premise does NOT reproduce against the current `target/release/grit` (rebuilt fresh in this
worktree). The file passes 9/9 with no hang.

### Evidence
- Official harness: `./scripts/run-tests.sh t4067-diff-partial-clone.sh --output-csv /tmp/wf-t4067.csv --no-catalog --quiet`
  produced row `t4067-diff-partial-clone t4 yes 9 9 0 true ok 0` (status=ok, NOT timeout).
- Direct verbose run completed in ~1s: `# Tests: 9  Pass: 9  Fail: 0  Skip: 0` — `# passed all 9 test(s)`.
  A 90s `timeout` wrapper never fired.

### Why it works (already implemented, no change needed)
The diff-on-partial-clone batched-blob prefetch is wired in `grit/src/commands/diff.rs`
(`prefetch_promisor_for_diff_entries` calls) backed by `grit/src/commands/promisor_hydrate.rs`,
`grit-lib/src/promisor.rs`, and `grit-lib/src/odb.rs`. `grit diff` over a `blob:limit=0` partial
clone performs exactly ONE batched fetch negotiation (single `fetch> done`), satisfying the
`test_line_count = 1 done_lines` assertions; the no-fetch path (`--raw -M` without break-rewrites)
correctly performs zero fetches (`test_path_is_missing trace`).

## Quality gates (this worktree)
- `cargo build --release -p grit-cli`: ok (incremental, warm cache).
- `cargo fmt`: no changes (no edits made).
- `cargo test -p grit-lib --lib`: 204 passed; 0 failed.
- Regression guards: `t0410-partial-clone` 38/38 (5 skip), `t1022-read-tree-partial-clone` 1/1. No regressions.

## Conclusion
No Rust changes were necessary; the test is already green. This worktree contains only this
verification log. A genuine future hang here would be a fetch-negotiation stall in
`grit-lib/src/promisor.rs`, not in the test.
