# t2405-worktree-submodule

**Date:** 2026-05-20

## Run

```bash
./scripts/run-tests.sh t2405-worktree-submodule.sh
```

Note: row was `in_scope=skip`; set to `yes` so the harness would execute the file.

## Result

- Harness: **11** tests, **10** pass, **0** fail (exit 0, ✓)
- One `test_expect_failure` ("submodule is checked out just after worktree add") — counts as known breakage, not a fail.
- `data/test-files.csv`: `t2405-worktree-submodule` — `in_scope=yes`, total **11**, pass **10**, fail **0**, `test_expect_failure=true`, status **ok**.

## Plan

- Marked `- [x] t2405-worktree-submodule` under PLAN.md §1.4.
