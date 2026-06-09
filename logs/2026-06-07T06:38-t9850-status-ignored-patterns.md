# t9850-status-ignored-patterns — mop-up round 1

Ticket: 8fbdf9. Fresh run: 34/36 (stable across re-runs; no cascade from other agents).

## Remaining failures (subtests 28 & 29)

- 28 `status --porcelain shows branch header`: asserts `grep "^## master"` on bare
  `grit status --porcelain` output (no `-b`).
- 29 `status on new branch shows correct branch`: same, after `checkout -b test-branch`,
  asserts `grep "^## test-branch"`.

Both require bare `--porcelain` (no `-b`/`--branch`) to emit a `## <branch>` header.

## Diagnosis — unfixable in grit Rust without violating Git compat AND regressing siblings

grit's current behavior is Git-correct and verified empirically:
- `grit status --porcelain`  (clean repo) -> empty (no `##`)
- `grit status --porcelain -b`            -> `## main`

Real Git only prints the `##` branch header in short/porcelain when `-b` is explicitly
passed: `git/builtin/commit.c` sets `use_deferred_config=false` for porcelain, and
`git/wt-status.c` gates the `##` line on `show_branch`. grit matches this exactly
(`grit/src/commands/status.rs` line 528 blocks `status.branch` config for porcelain;
line 2118 `format_short` gates the `## {branch}` write on `args.branch`).

### The contradiction with in-scope sibling files
The SAME command `grit status --porcelain` (bare) has mutually exclusive expected outputs:
- t9850/28-29:        MUST contain `## master` / `## test-branch`.
- t12270/19-21:       clean repo -> `test_must_be_empty` (no filter; any `##` fails). [32/32 currently]
- t12270/84-86:       `! grep "^## main$"` (explicitly forbids the header).
- t12570:             relies on the same no-header porcelain contract. [38/38 currently]

There is no repo/config/state discriminator grit could key on (both are default-branch
repos created the same way), so no grit Rust change can satisfy t9850/28-29 without
breaking t12270 (32/32) and t12570 (38/38) and diverging from Git.

Note: in t9850, `REAL_GIT=$(command -v git)` (line 9) runs AFTER `. ./test-lib.sh`
(line 7) prepends the grit `git` wrapper to PATH, so `REAL_GIT` is also grit — the
`## master` assertion is testing grit's own porcelain output, confirming the conflict
is purely about grit's (correct) bare-porcelain behavior.

## Conclusion
This is a test-porting defect in t9850: it assumes a non-Git grit behavior that
contradicts grit's own porcelain contract enforced by t12270/t12570. Resolving it
requires editing the t9850 test bodies (out of contract) or marking the file as a
porting defect. No grit Rust fix is possible. 34/36 is the correct max while honoring
Git compatibility and the no-test-edit contract. Confirms prior agent's finding with
additional verification (REAL_GIT=grit, sibling-file proof, empirical output).
