# Submodule t7 Plan

Goal: make all in-scope t7 submodule tests fully pass. Work one file at a time, starting with
the largest failing count in `data/test-files.csv`, and update this plan after each meaningful
test run.

Source of truth: `data/test-files.csv` rows in group `t7` whose names are submodule-focused.
Rows marked `skip` remain out of aggregate scope until explicitly audited.

## Current Queue

- [x] `t7406-submodule-update.sh` - 70/70 passing. Focus: submodule update behavior.
  - Fixed this iteration: relative `submodule add` URL fallback for local clones, update-init
    pathspec double-normalization from subdirectories, checkout status stdout, clone `done.`
    progress, default update preserving local changes unless `--force`, `--remote` branch/tag
    parity for decorated `log --oneline` comparisons, fresh clone checkout forcing, update=none
    initialization selection, custom update command error OIDs, jobs trace propagation, and quiet
    output suppression, plus shallow depth rejection/retry.
- [x] `t7400-submodule-basic.sh` - 124/124 passing. Focus: basic submodule porcelain.
  - Fixed this iteration: racy `diff-files` stat matching for refreshed zero-size files,
    `submodule add -b` branch checkout/config, POSIX discovery for paths containing literal
    backslashes, hunkless gitlink `apply --index`, trailing-slash gitlink add/reset pathspecs,
    checkout removal of `.git`-only submodule placeholders, resolved local-config URLs for
    relative `submodule add`, logical-name reuse handling with `--force`, recursive clone quiet
    propagation, and clone honoring `init.templateDir` for submodule add hooks.
- [x] `t7112-reset-submodule.sh` - 78/78 aggregate passing; 4 upstream TODO known breakages omitted from failing count. Focus: reset recursion and gitlinks.
- [x] `t7506-status-submodule.sh` - 40/40 passing. Focus: status submodule reporting.
- [x] `t7407-submodule-foreach.sh` - 23/23 passing. Focus: foreach traversal/env/output.
  - Fixed this iteration: plain CLI `submodule update --init` no longer recurses into nested
    submodules unless `--recursive` is explicitly requested.
- [x] `t7403-submodule-sync.sh` - 18/18 passing. Focus: sync URL propagation.
  - Verified this iteration: previously fixed sync behavior now passes the full file; no Rust
    changes were needed beyond refreshing harness metadata.
- [ ] `t7401-submodule-summary.sh` - 10/25 passing, 15 failing. Focus: submodule summary output.
- [ ] `t7814-grep-recurse-submodules.sh` - 17/27 passing, 10 failing. Focus: grep recursion.
- [ ] `t7422-submodule-output.sh` - 9/18 passing, 9 failing. Focus: submodule command output.
- [ ] `t7408-submodule-reference.sh` - 8/16 passing, 8 failing. Focus: reference clone/update.
- [ ] `t7425-submodule-gitdir-path-extension.sh` - 18/23 passing, 5 failing. Focus: gitdir path extension.
- [ ] `t7402-submodule-rebase.sh` - 4/6 passing, 2 failing. Focus: submodule rebase update mode.
- [ ] `t7409-submodule-detached-work-tree.sh` - 1/3 passing, 2 failing. Focus: detached work tree handling.
- [ ] `t7412-submodule-absorbgitdirs.sh` - 10/12 passing, 2 failing. Focus: absorbgitdirs.
- [ ] `t7423-submodule-symlinks.sh` - 4/6 passing, 2 failing. Focus: symlink safety.
- [ ] `t7418-submodule-sparse-gitmodules.sh` - 8/9 passing, 1 failing. Focus: sparse `.gitmodules`.
- [ ] `t7426-submodule-get-default-remote.sh` - 14/15 passing, 1 failing. Focus: default remote lookup.

## Passing

- [x] `t7411-submodule-config.sh` - 20/20 passing.
- [x] `t7413-submodule-is-active.sh` - 10/10 passing.
- [x] `t7414-submodule-mistakes.sh` - 5/5 passing.
- [x] `t7416-submodule-dash-url.sh` - 18/18 passing.
- [x] `t7417-submodule-path-url.sh` - 5/5 passing.
- [x] `t7419-submodule-set-branch.sh` - 9/9 passing.
- [x] `t7420-submodule-set-url.sh` - 3/3 passing.
- [x] `t7421-submodule-summary-add.sh` - 5/5 passing.

## Skipped

- [ ] `t7424-submodule-mixed-ref-formats.sh` - `in_scope=skip`; audit after in-scope queue is green.
