# t7 family quick wins — 2026-06-02

## t7 failure grouping (in-scope, failing > 0)

| Group | Files | Total failing | Notes |
|-------|-------|---------------|-------|
| Submodule | t7406, t7400, t7407, t7403, t7401, t7506, t7422, t7408, t7112, … | ~200+ | Depends on submodule update/sync/foreach |
| Commit porcelain | t7502, t7507, t7501-basic-functionality, t7500, t7509, t7514-trailers | ~130+ | commit -v, -F, authorship, trailers |
| Status / wtstatus | t7512, t7061, t7064, t7508, t7525, t7519 | ~50+ | porcelain v2, ignore, fsmonitor |
| Grep / difftool | t7810, t7800, t7814, t7818 | ~90+ | grep options, difftool drivers |
| Repack | t7700, t7701, t7703 | ~27 | geometric, unreachable |
| Reset | t7107, t7112, t7113, t7111 | ~38 | pathspec file, submodule, hooks |
| Clean | t7301, t7300 | ~16 | interactive clean, submodules |
| Merge | t7600–t7615 | ~15 | octopus, abort, autostash |
| Misc | t7001, t7005, t7010, t7450, t7900 | ~10 | mv, editor, setup, maintenance |

## This session — Status/Reset quick wins

### Root causes fixed

1. **t7060-wtstatus** — `commit --dry-run` exited before printing status when unmerged entries existed.
2. **t7103-reset-bare** — Mixed reset from inside `.git/` was rejected because `work_tree.is_none()`; use `is_bare()` instead.
3. **t7106-reset-unborn-branch** — `reset -p` on unborn branch failed resolving HEAD; use empty tree OID.
4. **t7065-status-rename** — passed on re-run (already green).

### Validation

```
./scripts/run-tests.sh t7060-wtstatus.sh t7103-reset-bare.sh t7106-reset-unborn-branch.sh t7065-status-rename.sh
→ all fully passing
```

## Merge group — octopus failure cleanup

### Root cause (t7607)

Multi-head octopus merge treated conflicts on non-final heads like a normal merge conflict
(writing `MERGE_HEAD`). Git's `git-merge-octopus.sh` aborts with exit 2 and restores state when
an intermediate head fails.

### Fix

In `do_octopus_merge`, when conflicts occur before the last merge head, restore the pre-merge
index/worktree, remove `ORIG_HEAD`, print Git's octopus failure messages, and exit 2.

### Validation

```
./scripts/run-tests.sh t7607-merge-state.sh → 1/1 passing
```
