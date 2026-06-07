# t9130-status-porcelain-v2 — work log (2026-06-06)

Ticket: 4d32f2 — tests/t9130-status-porcelain-v2.sh
Group: status-index (thread C)

## Fresh run
`./scripts/run-tests.sh t9130-status-porcelain-v2.sh` → 23/26.

Failing subtests:
- 2: `status --porcelain on clean repo shows branch header` → `grep "^## master"`
- 16: `status --porcelain shows branch in header` → `grep "^## master"`
- 21: `status --porcelain in fresh empty repo has branch header` → `grep "^## "`

All three assert that **`grit status --porcelain` (v1, WITHOUT `-b`)** prints a
`## <branch>` header.

## Root cause: the failing test assertions are WRONG (contradict Git + the rest of the suite)

Real Git behavior (verified directly against `git` binary):
- `git status --porcelain` on a clean repo prints **nothing** (0 lines), no `##` header.
- `git status --porcelain -b` prints `## master`.
- Even with `status.branch=true`, `--porcelain` (v1) still prints **no** `##` line.
  The `##` header for porcelain v1 requires explicit `-b`.

grit already matches Git exactly:
- `grit status --porcelain` (clean) → empty.
- `grit status --porcelain -b` → `## master`.
  (Code: `grit/src/commands/status.rs` `format_short`, line ~2118 `if args.branch { write "## ..." }`;
   and line ~527-528 explicitly documents that porcelain ignores `status.branch` and the `##`
   line requires `-b`, matching Git.)

The t9130 subtests 2/16/21 do not pass `-b` and do not set any config, yet `grep "^## master"`.
That expectation is simply incorrect.

## Why it can't be "fixed" in Rust without breaking the suite

Making `--porcelain` (v1) always emit `## branch` would be a divergence from Git and would
regress many other tests that compare exact `--porcelain` output against fixtures that have
NO `##` line, or that assert emptiness. Concretely:
- `tests/t12270-status-porcelain-v2.sh:19` — "porcelain shows nothing for clean repo" →
  `test_must_be_empty actual`. Would FAIL if `##` were emitted.
- Dozens of tests do `grit status --porcelain >actual && test_cmp expected actual` where
  `expected` has no `##` line (t10190, t10180, t11500, t11510, t12570, t13050, t12270, ...).
- Many use `grit status --porcelain | grep -v "^##"` (t12100, t12110, t12400, t12420, ...) —
  those tolerate the header, but the exact-compare ones above do not.

So there is no Rust change that makes subtests 2/16/21 pass without regressing other,
correctly-passing tests.

## Rules constraint

- "Fix grit Rust code... Do not modify tests. The only exception: flipping
  test_expect_failure -> test_expect_success." These three are already `test_expect_success`
  and failing; I'm not allowed to flip success->failure, and the correct outcome here is that
  the assertions are buggy.

## Conclusion / recommendation

grit's behavior is correct (matches Git). The remaining 3 failures are bad test assertions in
the ported test file. Recommended resolution (out of my allowed scope — requires a test edit
beyond the failure->success flip): change subtests 2, 16, 21 to use `--porcelain -b`
(or `--porcelain=v2`, which always has the `# branch.*` header), OR mark them
test_expect_failure. No grit code change is warranted.

No Rust changes made. Ticket left open with these findings.
