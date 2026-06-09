# t9850-status-ignored-patterns — work log

Ticket: 8fbdf9. Subsystem group "status-index" (thread C).

## Starting state
Fresh run: 34/36 passing. Failing subtests:
- 28: `status --porcelain shows branch header` — expects `grep "^## master"` on bare `grit status --porcelain` (no `-b`).
- 29: `status on new branch shows correct branch` — expects `grep "^## test-branch"` on bare `grit status --porcelain` after `checkout -b test-branch`.

## Root-cause investigation

Both failing subtests assert that a *bare* `status --porcelain` (without `-b`/`--branch`)
emits a `## <branch>` header line. This contradicts real Git.

Verified against the actual `git` binary in /tmp scratch:
- `git status --porcelain`            -> empty (NO `## master` line)
- `git status --porcelain -b`         -> `## master`
- `git status --porcelain` with `status.branch=true` config -> still empty (porcelain ignores it)

Confirmed in the canonical C source `git/builtin/commit.c`:
- Line 1258-1259: `use_deferred_config` is set to *false* for `STATUS_FORMAT_PORCELAIN`
  and `STATUS_FORMAT_PORCELAIN_V2`.
- Lines 1275-1278: `s->show_branch` therefore stays at its default of 0 unless `-b`
  was explicitly passed. So `--porcelain` never prints the `## branch` header without `-b`.
- `git/wt-status.c:2173` (`wt_shortstatus_print`): the tracking/`## ` line is only printed
  `if (s->show_branch)`.

grit's behavior already matches Git exactly: `grit status --porcelain` emits no `##`
header; `grit status --porcelain -b` emits `## master`. The gating lives in
`grit/src/commands/status.rs`:
- `format_short` line ~2118: `if args.branch { write!(out, "## {branch}") ... }`
- `format_porcelain_v2` line ~1633: same `if args.branch` gate.
- Lines 526-537 deliberately make porcelain ignore `status.branch` config, matching
  the C `use_deferred_config == false` rule.

## Why subtests 28/29 cannot be fixed in grit

Making bare `status --porcelain` emit `## <branch>` would:
1. Diverge from real Git (compatibility regression), and
2. Break sibling test files in this same suite that assert the opposite, e.g.:
   - `tests/t12270-status-porcelain-v2.sh`:
     `porcelain shows nothing for clean repo` -> `grit status --porcelain` + `test_must_be_empty`
     and `short shows nothing for clean repo` -> `grit status -s` + `test_must_be_empty`.
   - `tests/t12570-status-rename-copy.sh`:
     `status --porcelain: clean repo is empty (matches git)` -> `test_must_be_empty`.
   These require bare `--porcelain` / `-s` to be empty on a clean repo — i.e. NO `##` line.

The "matches git" comparison subtests *within* t9850 all filter `^##` from both sides,
so they are unaffected either way; but the t12270/t12570 emptiness tests have no such filter
and would regress.

## Conclusion

Subtests 28 and 29 are mis-ported: they assert non-Git behavior (a `##` header from bare
`--porcelain`) that also conflicts with other tests in the suite. This is a test-design
defect, not a grit bug. Per the contract I may not edit test bodies (only flip
expect_failure->expect_success, which does not apply here). grit's `status --porcelain`
branch-header handling is correct and Git-compatible.

Final honest run: 34/36. Leaving the ticket open with these findings for the mop-up agent.
