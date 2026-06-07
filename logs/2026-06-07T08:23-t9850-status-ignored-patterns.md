# t9850-status-ignored-patterns — mop-up round 2

Ticket: 8fbdf9 (t9, test)
Date: 2026-06-07T08:23 UTC
Agent: schacon+claude-t5@gmail.com

## Fresh run

`./scripts/run-tests.sh t9850-status-ignored-patterns.sh` => **34/36** (stable across re-runs).
No cascade from other agents' fixes. Same two subtests failing as prior rounds:

- 28: `status --porcelain shows branch header` (line 315)
- 29: `status on new branch shows correct branch` (line 323)

## Diagnosis (confirms + hard-verifies prior agents' finding: UNFIXABLE test-design defect)

Both subtests run bare `grit status --porcelain` (no `-b`) and `grep "^## master"` /
`grep "^## test-branch"`. They require the `## <branch>` header to appear on bare
`--porcelain`.

### Real git does NOT do this — verified empirically against git 2.52.0

```
$ git status --porcelain                      # clean repo => EMPTY (no ## header)
$ git -c status.branch=true status --porcelain # still EMPTY (v1 porcelain ignores config)
$ git status --porcelain -b                    # ## master
```

grit behaves identically: bare `--porcelain` => empty; `--porcelain -b` => `## master`.
So grit already matches Git exactly; the test asserts behavior Git itself never produces.

### C ground truth

`git/builtin/commit.c:1256-1289` `finalize_deferred_config()`:
`use_deferred_config` is false when `status_format == STATUS_FORMAT_PORCELAIN`
(line 1258-1259), so `status.branch` config is ignored and `s->show_branch` (init -1)
is forced to 0 (line 1277-1278). `wt-status.c` gates the `##` line on `show_branch`,
so bare `--porcelain` never emits it. Only `-b` (`OPT_BOOL('b',"branch",...)`,
commit.c:1552) sets it.

### Note on REAL_GIT in this test

t9850 line 9 sets `REAL_GIT=$(command -v git)` AFTER sourcing test-lib.sh, which has
already prepended the grit `git` wrapper to PATH. So `$REAL_GIT` IS grit — the `##`
assertion in 28/29 tests grit's own output against an incorrect expectation, not real git.

## Why it cannot be fixed without regressing other in-scope files

Making bare `grit status --porcelain` emit `## <branch>` would diverge from Git AND break
sibling files that assert the OPPOSITE for the identical command (both clean repos, neither
sets `status.branch`):

- `tests/t12270-status-porcelain-v2.sh` (32/32 passing): line 19-21 `porcelain shows
  nothing for clean repo` => `test_must_be_empty`; line 84-86 bare `--porcelain` =>
  `! grep "^## main$"`; plus lines 61-62, 179-180, 199-200 `test_must_be_empty` on bare
  `--porcelain`.
- `tests/t12570-status-rename-copy.sh` (38/38 passing): relies on the same no-header
  behavior.

There is no repo/config discriminator that could satisfy both t9850/28-29 and t12270/t12570.
The commit contract forbids editing test bodies, and these are already `test_expect_success`.

## Conclusion

Subtests 28 & 29 are a **porting/test-design defect** (test bug, not grit bug). 34/36 is the
correct, Git-faithful maximum for grit. No Rust change is possible or appropriate. Leaving the
ticket OPEN per mop-up guidance with this finding recorded; recommend marking these two
subtests as a porting defect.

## Side observation (out of scope, not changed)

grit does not honor `status.branch=true` config for the human-readable (`-c status.branch`)
path either — `grit -c status.branch=true status --porcelain` is empty (correct for v1
porcelain). Not relevant to this ticket; no test in scope exercises it.
