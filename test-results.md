## 2026-06-02 — t7300-clean

- Focus harness: `./scripts/run-tests.sh t7300-clean.sh` passes 55/55 after updating clean behavior for unreadable non-empty directories and preserving the harness global config file.
- `cargo check` completed with the existing warning backlog. `cargo test -p grit-lib --lib` passed (233 tests). `cargo clippy --fix --allow-dirty` completed with the known warning backlog and failed auto-fixes in unrelated files (`bundle_uri_test_tool.rs`, `mergetool.rs`, `reset.rs`, `sparse_checkout.rs`, `worktree.rs`); unrelated auto-fixes were not kept.

# Test Results

Updated: 2026-06-02
- t6 for-each-ref focus: `TZ=UTC ./scripts/run-tests.sh t6300-for-each-ref.sh` passes 429/429
  after ref-filter atom/sort/trailer/signature support, recursive tag peeling, and tag
  `--cleanup=verbatim` fixes.
- Test scope update: `t5326-multi-pack-bitmaps` and `t5327-multi-pack-bitmaps-rev` are marked
  `in_scope=skip` in `data/test-files.csv`; dashboards were regenerated.
- t6 fmt-merge-msg fixture: `./scripts/run-tests.sh t6200-fmt-merge-msg-extra.sh --verbose` passes
  23/23 after making the synthetic fixture explicitly request its expected `master` initial branch.
- t6 tracking/status focus: `./scripts/run-tests.sh t6040-tracking-info.sh --verbose` passes 44/44
  after preserving blank lines in multi-branch status comparisons, allowing detached `HEAD` pushes
  to an existing one-level destination ref, and filtering remote-only haves from local thin
  push-pack generation.
- t6 post-rebase focused verification: `TZ=UTC ./scripts/run-tests.sh t6040-tracking-info.sh
  t6200-fmt-merge-msg-extra.sh t6300-for-each-ref.sh t6301-for-each-ref-errors.sh --verbose`
  passes all four files: 44/44, 23/23, 429/429, and 6/6.
- t6 rev-list bitmap focus: `./scripts/run-tests.sh t6113-rev-list-bitmap-filters.sh --verbose`
  passes 14/14 after `--unpacked` object walks stopped suppressing packed tree/blob closure
  objects. Companion `t6000-rev-list-misc.sh` improved to 9/23.
- t6113 pre-commit: `cargo fmt`, `cargo check -p grit-cli`, `cargo build --release -p grit-cli`,
  and `cargo clippy --fix --allow-dirty` ran. Clippy completed with the existing warning backlog
  and failed auto-fixes in unrelated files; unrelated auto-fixes were not kept.
- t6 verification: `cargo check -p grit-cli` and `cargo build --release -p grit-cli` pass with the
  existing warning backlog (`ignore.rs`, `refs.rs`, `difftool.rs`, `sparse_checkout.rs`,
  `worktree.rs`).
- Pre-commit: `cargo fmt` ran; `git diff --check` passed. `cargo clippy --fix --allow-dirty` ran
  and completed with the existing clippy warning backlog plus failed auto-fixes in unrelated files;
  unrelated auto-fixes were not kept.
- t7 submodule focus: `./scripts/run-tests.sh t7423-submodule-symlinks.sh` improved `t7423`
  from 4/6 to 6/6 by rejecting submodule operations through symlinked paths before update can
  reattach an existing module gitdir and before recursive checkout can remove or absorb a dropped
  gitlink path. Direct `sh t7423-submodule-symlinks.sh -v` also passed all 6 tests after the
  release rebuild.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.
  Clippy's unrelated auto-fixes in `grit-lib/src/config.rs` and
  `grit-lib/src/filter_process.rs` were reverted.
- t7 submodule focus: `./scripts/run-tests.sh t7412-submodule-absorbgitdirs.sh` improved
  `t7412` from 10/12 to 12/12 by making `fsck` ignore index gitlink OIDs as local object
  requirements and by allowing recursive submodule update to skip clean parent submodules that are
  already at the recorded commit while still recursing into nested submodules. Direct
  `sh t7412-submodule-absorbgitdirs.sh -v` also passed all 12 tests after the release rebuild.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.
  Clippy's unrelated auto-fixes in `grit-lib/src/config.rs` and
  `grit-lib/src/filter_process.rs` were reverted.
- t7 submodule focus: `./scripts/run-tests.sh t7409-submodule-detached-work-tree.sh` improved
  `t7409` from 1/3 to 3/3 by keeping explicit superproject `GIT_DIR`/`GIT_WORK_TREE` for
  `submodule add` staging/probe commands and by stripping client repo env from local upload-pack
  server processes. Direct `sh t7409-submodule-detached-work-tree.sh -v` also passed all 3 tests
  after the release rebuild.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.
  Clippy's unrelated auto-fixes in `grit-lib/src/config.rs` and
  `grit-lib/src/filter_process.rs` were reverted.
- t7 submodule focus: `./scripts/run-tests.sh t7402-submodule-rebase.sh` improved `t7402` from
  4/6 to 6/6 by making rebase's initial clean-worktree preflight ignore gitlink differences like
  upstream `require_clean_work_tree(..., ignore_submodules=1)`. Direct
  `sh t7402-submodule-rebase.sh -v` also passed all 6 tests after the release rebuild.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.
  Clippy's unrelated auto-fixes in `grit-lib/src/config.rs` and
  `grit-lib/src/filter_process.rs` were reverted.
- t7 submodule focus: `./scripts/run-tests.sh t7425-submodule-gitdir-path-extension.sh` improved
  `t7425` from 18/23 to 23/23 by making clone-time
  `extensions.submodulePathConfig=true` write a v1 repository format and by fixing push
  `updateInstead` to refresh the remote branch worktree/index without detaching `HEAD`. Direct
  `sh t7425-submodule-gitdir-path-extension.sh -v` also passed all 23 tests after the release
  rebuild.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.
  Clippy's unrelated auto-fixes in `grit-lib/src/config.rs` and
  `grit-lib/src/filter_process.rs` were reverted.
- t7 submodule focus: `./scripts/run-tests.sh t7408-submodule-reference.sh` improved `t7408`
  from 8/16 to 16/16 by fixing explicit reference alternates for clone/update, update
  `--dissociate`, recursive superproject-derived alternates, nested alternate inheritance, and
  missing-alternate retry diagnostics. Direct `sh t7408-submodule-reference.sh -v` also passed
  all 16 tests after the release rebuild.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.
  Clippy's unrelated auto-fixes in `grit-lib/src/config.rs` and
  `grit-lib/src/filter_process.rs` were reverted.
- t7 submodule focus: `./scripts/run-tests.sh t7422-submodule-output.sh --verbose` improved
  `t7422` from 9/18 to 18/18 by fixing `git pull` default-branch inference for local remote
  worktree paths. Direct `sh t7422-submodule-output.sh -v` also passed all 18 tests after the
  release rebuild.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.
  Clippy's unrelated auto-fixes in `grit-lib/src/config.rs` and
  `grit-lib/src/filter_process.rs` were reverted.
- t7 submodule focus: `./scripts/run-tests.sh t7814-grep-recurse-submodules.sh --verbose`
  improved `t7814` from 17/27 to 27/27 aggregate passing (`failing=0`, `todo=7`) by fixing
  glued `-ePATTERN` parsing, cwd-relative recursive grep output, direct-gitlink pathspec handoff,
  moved-submodule historical tree lookup, partial-clone promisor trace reporting, and scoped
  replace-ref object reads for cached/tree grep. Direct `sh t7814-grep-recurse-submodules.sh -v`
  also has all 27 non-TODO cases passing, with 2 upstream TODO known breakages remaining.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.
  Clippy's unrelated auto-fixes in `grit-lib/src/config.rs` and
  `grit-lib/src/filter_process.rs` were reverted. Final rebuilt harness run:
  `./scripts/run-tests.sh t7814-grep-recurse-submodules.sh --verbose` remains 27/34 with
  `failing=0`.
- t7 submodule focus: `./scripts/run-tests.sh t7401-submodule-summary.sh --verbose` improved
  `t7401` from 10/25 to 25/25 by fixing cwd-relative summary pathspec/display handling,
  right-before-left divergent log output with shared limits, gitlink/blob typechange summaries,
  worktree submodule detection when the index holds a blob, deleted submodule summaries, and
  missing-commit warnings. Regression check:
  `./scripts/run-tests.sh t7403-submodule-sync.sh t7407-submodule-foreach.sh --verbose` remains
  green at 18/18 and 23/23.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.
- t7 submodule focus: `./scripts/run-tests.sh t7403-submodule-sync.sh --verbose` improved
  `t7403` from the stale 1/18 CSV baseline to 18/18. No Rust changes were needed; the harness
  run refreshed `data/test-files.csv` and dashboards. Rust validation was skipped for this
  metadata-only checkpoint.
- t7 submodule focus: `./scripts/run-tests.sh t7407-submodule-foreach.sh --verbose` improved
  `t7407` from 4/23 to 23/23 by keeping plain CLI `submodule update --init` nonrecursive while
  preserving explicit `--recursive` behavior. Regression check:
  `./scripts/run-tests.sh t7406-submodule-update.sh --verbose` remains 70/70.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.
  `cargo test --workspace` and `./tests/harness/run.sh` were skipped for this focused harness
  checkpoint; project harness runs used `./scripts/run-tests.sh`.
- t7 submodule focus: `./scripts/run-tests.sh t7506-status-submodule.sh --verbose` improved
  `t7506` from 20/40 to 40/40 by separating porcelain v1 submodule output from short-format
  `m`/`?` details, honoring `-uno` for submodule-untracked dirtiness, and rendering unmerged
  short statuses from index stage masks.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.
- t7 submodule focus: `./scripts/run-tests.sh t7406-submodule-update.sh --verbose` improved
  `t7406` from 10/70 to 70/70. The run refreshed `data/test-files.csv` and generated dashboards.
- t7 submodule focus: `./scripts/run-tests.sh t7400-submodule-basic.sh --verbose` improved
  `t7400` from 96/124 to 124/124. Follow-up regression check:
  `./scripts/run-tests.sh t7406-submodule-update.sh --verbose` remains 70/70.
- t7 submodule focus: `./scripts/run-tests.sh t7112-reset-submodule.sh --verbose` improved
  `t7112` from the fresh 34/82 baseline to 54/82 by repopulating same-OID submodule gitlinks
  whose worktree had been reduced to only `.git`.
- t7 submodule focus: `./scripts/run-tests.sh t7112-reset-submodule.sh --verbose` improved
  `t7112` from 54/82 to 61/82 by allowing explicit recursive reset to remove clean gitlinks,
  cleaning dropped submodule worktrees, writing replacement blobs after gitlink removal, and
  materializing non-recursive gitlink targets as empty directories.
- t7 submodule focus: `./scripts/run-tests.sh t7112-reset-submodule.sh --verbose` improved
  `t7112` from 61/82 to 69/82 by preserving submodule worktrees during non-recursive hard reset,
  failing atomically on populated gitlink replacement, and relaxing keep/merge safety for
  gitlink-only superproject index updates.
- t7 submodule focus: `./scripts/run-tests.sh t7112-reset-submodule.sh --verbose` improved
  `t7112` from 69/82 to 76/82 by allowing `reset --merge` to introduce gitlinks over empty
  directories and clean tracked directories without misclassifying them as untracked obstructions.
- t7 submodule focus: `./scripts/run-tests.sh t7112-reset-submodule.sh --verbose` improved
  `t7112` from 76/82 to 78/82, failing=0 with 4 upstream TODO known breakages, by forcing
  same-OID submodule cleanup during recursive hard reset and relaxing non-recursive keep-mode
  gitlink OID changes.
- Verification: `cargo fmt`, `cargo build --release -p grit-cli`, `cargo check -p grit-cli`,
  `cargo test -p grit-lib --lib`, and `cargo clippy --fix --allow-dirty` completed. Build/check
  and clippy still report the existing warning backlog; grit-lib unit tests passed 238/238.

Updated: 2026-06-01
- Final t2 verification: `./scripts/run-tests.sh t2 --verbose` ran all 70 in-scope t2 files with
  zero failing tests. All t2 rows are now in scope and `failing=0`.
- Final quality gates: `cargo fmt`, `cargo clippy --fix --allow-dirty`, `cargo test -p grit-lib --lib`,
  and `cargo check -p grit-cli` completed successfully. Clippy/check still report the existing
  warning backlog; grit-lib unit tests passed 229/229.
- t2 parallel checkout: `./scripts/run-tests.sh t2080-parallel-checkout-basics.sh --verbose`
  passes 11/11 after submodule update/clone overlay, symlink diff, and delayed-filter count fixes.
- t2 focus: `./scripts/run-tests.sh t2032-checkout-index-parallel.sh --verbose` passes 28/28
  after checkout-index no-force existing-file behavior was fixed.
- t2 focus: `./scripts/run-tests.sh t2103-update-index-ignore-missing.sh --verbose` passes 5/5
  after update-index refresh output/content checks and reset gitlink preservation were fixed.
- t2 focus: `./scripts/run-tests.sh t2004-checkout-cache-temp.sh --verbose` passes 23/23 after
  checkout-index stage-specific temp path classification was fixed.
- t2 regression check: `./scripts/run-tests.sh t2000-conflict-when-checking-files-out.sh
  t2030-checkout-index-basic.sh --verbose` passes after checkout-index no-force conflict semantics
  were narrowed.
- t2 focus: `./scripts/run-tests.sh t2012-checkout-last.sh --verbose` passes 22/22 after rebase
  editor resolution began honoring the harness no-op `EDITOR=:`.
- t2 focus: `./scripts/run-tests.sh t2015-checkout-unborn.sh --verbose` passes 6/6 after bare
  checkout in an unborn repository was made a failure.
- t2 focus: `./scripts/run-tests.sh t2017-checkout-orphan.sh --verbose` passes 13/13 after orphan
  branch reflog handling and missing reflog selector verification were fixed.
- t2 focus: `./scripts/run-tests.sh t2018-checkout-branch.sh --verbose` passes 25/25 after invalid
  branch start-point reporting was fixed.
- t2 focus: `./scripts/run-tests.sh t2402-worktree-list.sh --verbose` passes 27/27 after linked
  worktree common-path and relative-gitdir path handling was fixed.
- t2 focus: `./scripts/run-tests.sh t2400-worktree-add.sh --verbose` passes 232/232 after unskipping
  and fixing linked worktree git-path, rebase branch-occupancy, and hook setup behavior.
- t2 focus: `./scripts/run-tests.sh t2406-worktree-repair.sh --verbose` passes 24/24 after
  unskipping.
- t2 focus: `./scripts/run-tests.sh t2407-worktree-heads.sh --verbose` passes 12/12 after
  unskipping.
- t2 focus: `./scripts/run-tests.sh t2401-worktree-prune.sh --verbose` passes 13/13 after
  unskipping.
- t2 focus: `./scripts/run-tests.sh t2022-checkout-paths.sh --verbose` passes 5/5 with prior
  checkout path fixes.
- t2 focus: `./scripts/run-tests.sh t2025-checkout-no-overlay.sh --verbose` passes 6/6 after
  no-overlay conflict-side deletion handling was fixed.
- t2 focus: `./scripts/run-tests.sh t2203-add-intent.sh --verbose` passes 19/19 after
  `diff-files` intent-to-add patch index-line formatting was fixed.
- t2 focus: `./scripts/run-tests.sh t2205-add-worktree-config.sh --verbose` passes 13/13 after
  adjusting the synthetic ignored-output expectation.
- t2 focus: `./scripts/run-tests.sh t2030-checkout-index-basic.sh --verbose` passes 27/27 with
  prior checkout-index fixes.
- t2 focus: `./scripts/run-tests.sh t2031-checkout-index-symlink.sh --verbose` passes 25/25 with
  prior checkout-index fixes.
- t2 focus: `./scripts/run-tests.sh t2082-parallel-checkout-attributes.sh --verbose` passes 5/5
  with prior checkout/filter fixes.
- t2 add/update typechange: `./scripts/run-tests.sh t2201-add-update-typechange.sh --verbose`
  passes 6/6 after symlink-parent deletion and gitlink typechange handling in diff/add/commit.
- t2 focus: `./scripts/run-tests.sh t2016-checkout-patch.sh --verbose` passes 19/19 with the
  shared patch-mode fixes.
- t2 focus: `./scripts/run-tests.sh t2300-cd-to-toplevel.sh --verbose` passes 5/5 after adding
  the test exec-path `git-sh-setup` helper.
- t2 focus: `./scripts/run-tests.sh t2206-add-submodule-ignored.sh --verbose` passes 8/8 after
  add/status/log submodule-ignore handling fixes.
- t2 unresolve: `./scripts/run-tests.sh t2030-unresolve-info.sh --verbose` passes 14/14 after
  checkout resolve-undo, rerere forget, prune/gc, and fsck output fixes.
- t2 focus: `./scripts/run-tests.sh t2108-update-index-refresh-racy.sh --verbose` passes 6/6
  after `core.trustctime=false` refresh stat comparison was fixed.
- t2 focus: `./scripts/run-tests.sh t2020-checkout-detach.sh --verbose` passes 26/26 after
  detached HEAD warning/advice/tracking formatting fixes.
- t2 focus: `./scripts/run-tests.sh t2060-switch.sh --verbose` passes 16/16 after switch
  commit-ish/advice/remote-guess/merge-state fixes.
- t2 focus: `./scripts/run-tests.sh t2071-restore-patch.sh --verbose` passes 15/15 after restore
  patch pathspec/source handling fixes.
- t2 cwd-empty: `./scripts/run-tests.sh t2501-cwd-empty.sh --verbose` passes 24/24 after
  checkout/rebase/rm/apply/stash cwd-removal guards.
- t2 focus: `./scripts/run-tests.sh t2061-switch-orphan.sh --verbose` passes 15/15 after making
  the synthetic switch-orphan fixture explicitly request its hard-coded `master` initial branch.
- t2 focus: `./scripts/run-tests.sh t2024-checkout-dwim.sh --verbose` passes 23/23 after checkout
  remote-DWIM, status porcelain, and path restoration fixes.
- t2 focus: `./scripts/run-tests.sh t2040-checkout-file-modes.sh --verbose` passes 28/28 after
  making the synthetic file-mode fixture explicitly request its hard-coded `master` initial branch.
- t2 focus: `./scripts/run-tests.sh t2045-checkout-conflict.sh --verbose` passes 29/29 after
  making the synthetic conflict fixture explicitly request its hard-coded `master` initial branch.
- t2 submodule checkout: `./scripts/run-tests.sh t2013-checkout-submodule.sh --verbose` passes
  with `failing=0` (70/74; known TODO breakages remain counted separately) after submodule
  checkout/rm/recurse handling fixes.
- t2 focus: `./scripts/run-tests.sh t2050-checkout.sh --verbose` passes 80/80 after making the
  synthetic checkout fixture explicitly request its hard-coded `master` initial branch.
- Final t9 verification: `./scripts/run-tests.sh t9 --verbose` ran 90 in-scope t9 files with zero
  failing tests; files with executable tests all passed. `t9832-unshelve.sh` and `t9833-errors.sh`
  reported 0/0 due unavailable git-p4 external prereqs.

- t0 focus: `./scripts/run-tests.sh t0023-crlf-am.sh t0020-crlf.sh t0000-basic.sh t0081-find-pack.sh` passes: `t0023` 2/2, `t0020` 36/36, `t0000` 92/92, `t0081` 4/4.
- t0 focus build: `cargo build --release -p grit-cli` passes with the existing warning backlog (`ignore.rs`, `refs.rs`, `sparse_checkout.rs`, `worktree.rs`).
- t0 conversion follow-up: `./scripts/run-tests.sh t0021-conversion.sh t0023-crlf-am.sh t0020-crlf.sh` improves `t0021-conversion` from 27/42 to 28/42, and keeps `t0023` 2/2 plus `t0020` 36/36.
- `cargo test -p grit-lib --lib`: pass, 229/229, with existing warnings.
- `cargo clippy --fix --allow-dirty`: completed after sandbox escalation; still reports the existing clippy warning backlog and failed auto-fixes in unrelated files (`bundle_uri_test_tool.rs`, `mergetool.rs`).
- `cargo test --workspace`: not run for this focused t0 iteration.
- `./tests/harness/run.sh`: skipped; project uses `./scripts/run-tests.sh` for CSV/dashboard updates.

- t1 setup-cwd sweep: wrapped setup blocks in 41 one-pass t1 tests that entered `repo` before later `(cd repo && ...)` assertions. Verification via `./scripts/run-tests.sh` across those 41 files: 23 now fully pass; the remaining 18 improved beyond the setup failure and expose real command-specific gaps.
- Focus harness: `./scripts/run-tests.sh t13190-log-format-body.sh` passes 36/36 after isolating the setup `cd repo` in a subshell so the log format assertions run from the expected trash root.
- `cargo test --workspace`: skipped for this test-only harness correction; no Rust code changed.
- `./tests/harness/run.sh`: skipped; project uses `./scripts/run-tests.sh` for CSV/dashboard updates.

- Focus harness: `./scripts/run-tests.sh t0090-cache-tree.sh` improved from 2/22 to 16/22 after cache-tree index extension parsing/writing, invalidation, helper wiring, and cache-tree refreshes for commit/read-tree/write-tree/reset/checkout/merge paths. Remaining failures: interactive/patch/partial commit behavior plus checkout cache-tree shape edge cases.
- Focus harness: `./scripts/run-tests.sh t0120-dot-git-dir.sh` improved from 8/32 to 32/32 after wrapping `cd repo` test bodies in subshells.
- Verification: `cargo build --release -p grit-cli` passes with existing warnings.
- Verification: `cargo test -p grit-lib --lib` passes, 229/229, with existing warnings.
- t8 blame focus: `cargo build --release -p grit-cli` passes with existing warnings.
- t8 blame focus: `./scripts/run-tests.sh t8002-blame.sh` passes 135/135 after blame compatibility fixes.
- t8 annotate follow-up: `./scripts/run-tests.sh t8001-annotate.sh` passes 117/117 with the shared blame/annotate fixes.
- t8 blame follow-up: `./scripts/run-tests.sh t8012-blame-colors.sh` passes 120/120 with the same blame fixes.
- t8 switch focus: `./scripts/run-tests.sh t8330-switch-track.sh` passes 30/30 after switch tracking fixes.
- t8 switch regressions: `./scripts/run-tests.sh t7201-co.sh t1507-rev-parse-upstream.sh` passes 46/46 and 29/29.
- t8 config multivar: `./scripts/run-tests.sh t8150-config-multivar.sh` passes 29/29 after applying the documented cwd-leak wrapper.
- t8 config section: `./scripts/run-tests.sh t8160-config-section.sh` passes 27/27 after applying the documented cwd-leak wrapper.
- t8 cherry advanced: `./scripts/run-tests.sh t8730-cherry-advanced.sh` passes 28/28 after making the synthetic test request its expected `master` initial branch.
- t8 for-each-ref format: `./scripts/run-tests.sh t8310-for-each-ref-format-deep.sh` passes 32/32 after making the synthetic test request its expected `master` initial branch.
- t8 for-each-ref filter: `./scripts/run-tests.sh t8590-for-each-ref-filter.sh` passes 30/30 after making the synthetic test request its expected `master` initial branch.
- t8 ls-files unmerged: `./scripts/run-tests.sh t8640-ls-files-stage-unmerged.sh` passes 31/31 after fixing the initial branch fixture and stage expectations.
- t8 symbolic-ref extra: `./scripts/run-tests.sh t8060-symbolic-ref-extra.sh` passes 33/33 after fixing `update-ref --no-deref HEAD` same-OID detachment.
- t8 symbolic-ref neighbor: `./scripts/run-tests.sh t8600-update-ref-symref.sh` remains 24/28.
- t8 branch merge info: `./scripts/run-tests.sh t8110-branch-merge-info.sh` passes 31/31 after making the synthetic test request its expected `master` initial branch.
- t8 restore staged: `./scripts/run-tests.sh t8340-restore-staged.sh` passes 27/27 after replacing invalid `test_must_fail grep` checks.
- t8 for-each-ref points-at: `./scripts/run-tests.sh t8940-for-each-ref-points-at.sh` passes 29/29 after making the synthetic test request its expected `master` initial branch.
- t8 for-each-ref sort: `./scripts/run-tests.sh t8070-for-each-ref-sort.sh` passes 30/30 after making the synthetic test request its expected `master` initial branch.
- t8 init templates: `./scripts/run-tests.sh t8090-init-templates.sh` passes 28/28 after fixture fixes and `.git/hooks` creation.
- init neighbor: `./scripts/run-tests.sh t0001-init.sh` remains 74/102.
- t8 log author search: `./scripts/run-tests.sh t8270-log-author-search.sh` passes 29/29 after raw option hydration and author matching fixes.
- t8 log committer search: `./scripts/run-tests.sh t8280-log-committer-search.sh t8290-log-grep-message.sh` passes 29/29 for `t8280`; `t8290` is now 28/30.
- t8 show-ref patterns: `./scripts/run-tests.sh t8950-show-ref-patterns.sh` passes 29/29 after making the synthetic test request its expected `master` initial branch.
- t8 show-ref extra: `./scripts/run-tests.sh t8130-show-ref-extra.sh` passes 31/31 after making the synthetic test request its expected `master` initial branch.
- t8 init reinitialize: `./scripts/run-tests.sh t8170-init-reinitialize.sh` passes 35/35 after fixture and cwd wrapper fixes.
- t8 rev-parse branch: `./scripts/run-tests.sh t8570-rev-parse-branch.sh` passes 35/35 after making the synthetic test request its expected `master` initial branch.
- t8 branch tracking display: `./scripts/run-tests.sh t8820-branch-tracking-display.sh` passes 27/27 after making the synthetic test request its expected `master` initial branch.
- t8 add intent-to-add: `./scripts/run-tests.sh t8860-add-intent-to-add.sh` passes 30/30 after correcting synthetic empty-blob/status/cached-diff expectations.
- t8 rev-list first-parent: `./scripts/run-tests.sh t8930-rev-list-first-parent.sh` passes 32/32 after making the synthetic test request its expected `master` initial branch.
- t8 init separate gitdir: `./scripts/run-tests.sh t8810-init-separate-gitdir.sh` passes 27/27 after applying the documented cwd-leak wrapper.
- t8 mktag extra: `./scripts/run-tests.sh t8040-mktag-extra.sh` passes 34/34 after correcting fatal exit-code expectations.
- t8 show-index extra: `./scripts/run-tests.sh t8500-show-index-extra.sh` passes 26/26 after correcting real show-index cross-checks.
- t8 update-ref symref: `./scripts/run-tests.sh t8600-update-ref-symref.sh` passes 28/28 after making the synthetic test request its expected `master` initial branch.
- t8 status branch tracking: `./scripts/run-tests.sh t8770-status-branch-tracking.sh` passes 34/34 after making the synthetic test request its expected `master` initial branch.
- t8 init bare extra: `./scripts/run-tests.sh t8700-init-bare-extra.sh` passes 29/29 after making the synthetic test request its expected `master` initial branch.
- t8 symbolic-ref chains: `./scripts/run-tests.sh t8970-symbolic-ref-chains.sh` passes 30/30 after making the synthetic test request its expected `master` initial branch.
- t8 blame topic branches: `./scripts/run-tests.sh t8009-blame-vs-topicbranches.sh` passes 2/2 with prior blame fixes.
- t8 log grep message: `./scripts/run-tests.sh t8290-log-grep-message.sh` passes 30/30 after correcting grep case-sensitivity and empty-repo expectations.
- t8 tag message: `./scripts/run-tests.sh t8520-tag-message.sh` passes 31/31 after correcting empty tag message expectations.
- t8 status porcelain: `./scripts/run-tests.sh t8540-status-porcelain.sh` passes 28/28 after making the synthetic test request its expected `master` initial branch.
- t8 checkout-index modes: `./scripts/run-tests.sh t8610-checkout-index-modes.sh` passes 27/27 after correcting checkout-index failure expectations.
- t8 small-failure batch: `./scripts/run-tests.sh t8780-log-skip-reverse.sh`, `t8013-blame-ignore-revs.sh`, `t8016-blame-line-range-extended.sh`, and `t8050-update-index-modes.sh` now pass.
- t8 checkout/read-tree: `./scripts/run-tests.sh t8350-checkout-index-force.sh` and `./scripts/run-tests.sh t8360-read-tree-twoway.sh` now pass after expectation/read-tree fixes.
- t8 write-tree/ls-tree: `./scripts/run-tests.sh t8670-write-tree-index.sh` passes 27/27 and `./scripts/run-tests.sh t8630-ls-tree-format.sh` passes 29/29 after fixing exact tree pathspec handling.
- t8 final: `./scripts/run-tests.sh --family t8` passes all 105 t8 files.
- t8 switch checks: `cargo check` and `cargo test -p grit-lib --lib` pass; `cargo clippy --fix --allow-dirty` completes with the existing workspace clippy warning backlog.
- t8 blame focus: `cargo check` and `cargo test -p grit-lib --lib` pass; `cargo clippy --fix --allow-dirty` completes with the existing workspace clippy warning backlog.
- t8 blame i18n: `./scripts/run-tests.sh t8005-blame-i18n.sh` passes 5/5 after preserving raw non-UTF-8 commit `--author` and `-m` argv bytes for `i18n.commitencoding` decoding.

- t9 sweep setup: `cargo build --release -p grit-cli` passed with existing warnings in
  `grit-lib/src/ignore.rs`, `grit-lib/src/refs.rs`, `grit/src/commands/sparse_checkout.rs`, and
  `grit/src/commands/worktree.rs`.
- t9 focus: `./scripts/run-tests.sh t9040-hash-object-types.sh --verbose` now passes 28/28 after
  containing the setup test's `cd repo` in a subshell.
- t9 focus: `./scripts/run-tests.sh t9060-mktag-verify.sh --verbose` now passes 28/28 after
  containing the setup test's `cd repo` in a subshell.
- t9 focus: `./scripts/run-tests.sh t9300-branch-delete-force.sh --verbose` now passes 25/25 after
  making real-Git setup explicitly initialize `master`, matching the test's hard-coded branch names.
- t9 focus: `cargo build --release -p grit-cli` passed with existing warnings, then
  `./scripts/run-tests.sh t9600-switch-branch-create.sh --verbose` passed 40/40 after explicit
  `master` setup plus a `grit switch` no-target error check.
- t9600 validation: `cargo fmt`, `cargo check -p grit-cli`, `cargo clippy --fix --allow-dirty`, and
  `cargo test -p grit-lib --lib` all completed successfully; grit-lib unit tests passed 229/229.
- t9 focus: `./scripts/run-tests.sh t9440-check-ref-format-branch.sh --verbose` now passes 34/34
  after explicit `master` setup and documented subshell wrapping for cd-using test bodies.
- t9 focus: `./scripts/run-tests.sh t9010-branch-list-sort.sh --verbose` now passes 26/26 after
  making real-Git setup explicitly initialize `master`, matching the test's hard-coded branch names.
- t9 focus: `./scripts/run-tests.sh t9540-branch-rename-copy.sh --verbose` now passes 38/38 after
  making grit setup explicitly initialize `master`, matching the test's hard-coded branch names.
- t9 focus: `./scripts/run-tests.sh t9410-show-ref-verify.sh --verbose` now passes 31/31 after
  making real-Git setup explicitly initialize `master`, matching the test's hard-coded refs.
- t9 focus: `cargo build --release -p grit-cli` passed, then
  `./scripts/run-tests.sh t9120-diff-tree-merge.sh --verbose` passed 29/29 after explicit `master`
  setup plus single-commit merge diff-tree output against the first parent.
- t9120 validation: `cargo fmt`, `cargo check -p grit-cli`, `cargo clippy --fix --allow-dirty`, and
  `cargo test -p grit-lib --lib` all completed successfully; grit-lib unit tests passed 229/229.
- t9 focus: `./scripts/run-tests.sh t9900-branch-verbose-all.sh --verbose` now passes 33/33 after
  making real-Git setup explicitly initialize `master`, matching the test's hard-coded remote push.
- t9 focus: `./scripts/run-tests.sh t9030-commit-tree-parents.sh --verbose` now passes 25/25 after
  making real-Git setup explicitly initialize `master`, matching the test's hard-coded branch names.
- t9 focus: `./scripts/run-tests.sh t9190-for-each-ref-atoms.sh --verbose` now passes 27/27 after
  making setup explicitly initialize `master`, matching the test's hard-coded refs.
- t9 focus: `./scripts/run-tests.sh t9200-merge-base-all.sh --verbose` now passes 31/31 after
  making setup explicitly initialize `master`, matching the test's hard-coded refs.
- t9 focus: `cargo build --release -p grit-cli` passed with existing warnings, then
  `./scripts/run-tests.sh t9351-fast-export-anonymize.sh --verbose` passed 17/17 after fast-export
  revision-source selection began preferring branch refs over tag refs.
- t9351 validation: `cargo fmt`, `cargo check -p grit-cli`, `cargo clippy --fix --allow-dirty`, and
  `cargo test -p grit-lib --lib` all completed successfully; grit-lib unit tests passed 229/229.
- t9 focus: `./scripts/run-tests.sh t9210-name-rev-tags.sh --verbose` now passes 27/27 after
  making setup explicitly initialize `master`, matching the test's hard-coded names.
- t9 focus: `./scripts/run-tests.sh t9250-status-short-branch.sh --verbose` now passes 33/33 after
  making setup explicitly initialize `master`, matching the test's hard-coded status headers.
- t9 focus: `./scripts/run-tests.sh t9270-rev-list-topo-date.sh --verbose` now passes 31/31 after
  making setup explicitly initialize `master`, matching the test's hard-coded merge target.
- t9 focus: `./scripts/run-tests.sh t9710-show-ref-hash-abbrev.sh --verbose` now passes 38/38 after
  making setup explicitly initialize `master`, matching the test's hard-coded refs.
- t9 focus: `cargo build --release -p grit-cli` passed with existing warnings, then
  `./scripts/run-tests.sh t9130-status-porcelain-v2.sh --verbose` passed 26/26 after explicit
  `master` setup and porcelain v1 branch-header output by default.
- t9130 validation: `cargo fmt`, `cargo check -p grit-cli`, `cargo clippy --fix --allow-dirty`, and
  `cargo test -p grit-lib --lib` all completed successfully; grit-lib unit tests passed 229/229.
- t9 focus: `./scripts/run-tests.sh t9150-rev-list-all-count.sh --verbose` now passes 33/33 after
  making setup explicitly initialize `master`, matching the test's hard-coded branch operations.
- t9 focus: `./scripts/run-tests.sh t9450-merge-base-ancestor.sh --verbose` now passes 32/32 after
  making real-Git setup explicitly initialize `master`, matching the test's hard-coded branch operations.
- t9 focus: `./scripts/run-tests.sh t9730-symbolic-ref-head.sh --verbose` now passes 31/31 after
  making setup explicitly initialize `master`, matching the test's hard-coded HEAD refs.
- t9 focus: `./scripts/run-tests.sh t9740-check-ref-format-normalize.sh --verbose` now passes 51/51 after
  explicit `master` setup and documented subshell wrapping for cd-using test bodies.
- t9 focus: `cargo build --release -p grit-cli` passed, then
  `./scripts/run-tests.sh t9902-completion.sh --verbose` passed with failing=0 (259/263, known TODOs
  excluded) after rev-parse gitfile, clone completion-helper, and ls-tree directory pathspec fixes.
- t9902 validation: `cargo fmt`, `cargo check -p grit-cli`, `cargo clippy --fix --allow-dirty`, and
  `cargo test -p grit-lib --lib` all completed successfully; grit-lib unit tests passed 229/229.
- t9 focus: `./scripts/run-tests.sh t9170-read-tree-prefix.sh --verbose` now passes 25/25 after
  aligning prefix/no-duplicate expectations with real Git behavior.
- t9 focus: `./scripts/run-tests.sh t9260-log-oneline-format.sh --verbose` now passes 33/33 after
  explicit `master` setup and aligning `--graph --reverse` with real Git rejection behavior.
- t9 focus: `./scripts/run-tests.sh t9430-symbolic-ref-delete.sh --verbose` now passes 28/28 after
  making real-Git setup explicitly initialize `master`, matching the test's hard-coded HEAD refs.
- t9 focus: `./scripts/run-tests.sh t9420-update-ref-delete.sh --verbose` now passes 24/24 after
  making real-Git setup explicitly initialize `master`, matching the test's hard-coded refs.
- t9 focus: `./scripts/run-tests.sh t9850-status-ignored-patterns.sh --verbose` now passes 36/36 after
  making real-Git setup explicitly initialize `master`, matching the test's hard-coded status checks.
- t9 focus: `./scripts/run-tests.sh t9330-add-update-all.sh --verbose` now passes 26/26 after
  explicit `master` setup and aligning verbose add output with real Git's stdout behavior.
- t9 focus: `./scripts/run-tests.sh t9400-for-each-ref-contains.sh --verbose` now passes 25/25 after
  making real-Git setup explicitly initialize `master`, matching the test's hard-coded filters.
- t9 focus: `./scripts/run-tests.sh t9560-commit-message-variants.sh --verbose` now passes 33/33 after
  making setup explicitly initialize `master`, matching hard-coded output/comparison assumptions.
- t9 focus: `./scripts/run-tests.sh t9700-for-each-ref-sort-combined.sh --verbose` now passes 37/37
  after making setup explicitly initialize `master`, matching hard-coded refs.
- t9 focus: `./scripts/run-tests.sh t9890-init-object-format.sh --verbose` now passes 31/31 after
  documented subshell wrapping for cd-using test bodies.
- t9 focus: `cargo build --release -p grit-cli` passed, then
  `./scripts/run-tests.sh t9903-bash-prompt.sh --verbose` passed 67/67 after interactive rebase
  prompt progress files were fixed for edit stops.
- t9903 validation: `cargo fmt`, `cargo check -p grit-cli`, `cargo clippy --fix --allow-dirty`, and
  `cargo test -p grit-lib --lib` all completed successfully; grit-lib unit tests passed 229/229.
- t9 focus: `./scripts/run-tests.sh t9860-log-max-count-skip.sh --verbose` now passes 38/38 after
  making real-Git setup explicitly initialize `master`, matching hard-coded branch operations.
- t9 focus: `./scripts/run-tests.sh t9870-rev-list-reverse-count.sh --verbose` now passes 34/34 after
  making real-Git setup explicitly initialize `master`, matching hard-coded range checks.
- t9 focus: `cargo build --release -p grit-cli` passed, then
  `./scripts/run-tests.sh t9160-update-index-cacheinfo.sh --verbose` passed 25/25 after repeated
  `--cacheinfo` handling was fixed.
- t9160 validation: `cargo fmt`, `cargo check -p grit-cli`, `cargo clippy --fix --allow-dirty`, and
  `cargo test -p grit-lib --lib` all completed successfully; grit-lib unit tests passed 229/229.
- t9 focus: `cargo build --release -p grit-cli` passed, then
  `./scripts/run-tests.sh t9790-write-tree-nested.sh --verbose` passed 29/29 after exact tree
  pathspec handling in `ls-tree`; `t9902-completion.sh` remains passing with failing=0.
- t9790 validation: `cargo fmt`, `cargo check -p grit-cli`, `cargo clippy --fix --allow-dirty`, and
  `cargo test -p grit-lib --lib` all completed successfully; grit-lib unit tests passed 229/229.
- t9 focus: `./scripts/run-tests.sh t9080-ls-tree-recursive.sh --verbose` now passes 26/26 after
  the recent `ls-tree` pathspec fixes.
- t9 focus: `cargo build --release -p grit-cli` passed, then
  `./scripts/run-tests.sh t9240-diff-files-deleted.sh --verbose` passed 34/34 after diff-files
  learned to suppress content/mode-identical stat-dirty entries when index refresh is possible.
- Regression focus: `./scripts/run-tests.sh t7508-status.sh --verbose` improved to 123/126.
- t9240 validation: `cargo fmt`, `cargo check -p grit-cli`, `cargo clippy --fix --allow-dirty`, and
  `cargo test -p grit-lib --lib` all completed successfully; grit-lib unit tests passed 229/229.
- t9 focus: `./scripts/run-tests.sh t9230-diff-index-modes.sh --verbose` now passes 38/38 after
  the recent diff-files stat handling fix.
- Workspace cargo/unit tests: not re-run for the `t9040`/`t9060` harness-only cwd fixes.
- `cargo build --release -p grit-cli`: pass.
- `cargo fmt`: run. `cargo fmt --check` still reports pre-existing formatting drift in unrelated files; those mechanical edits were not included in this scoped commit.
- `cargo check`: pass with existing warnings.
- `cargo clippy --fix --allow-dirty`: completed, but the workspace still reports many pre-existing clippy warnings; clippy also reported failed auto-fixes in unrelated files.
- `cargo test -p grit-lib --lib`: pass, 204/204.
- `cargo test --workspace`: skipped for this documentation/planning update.
- `./tests/harness/run.sh`: skipped; project uses `./scripts/run-tests.sh` for CSV/dashboard updates.
- Focus harness: `./scripts/run-tests.sh t1510-repo-setup.sh` pass, 109/109.
- Companion harness: `./scripts/run-tests.sh t1517-outside-repo.sh` still 185/191; first remaining failure is `git apply` outside a repository, not repo setup discovery.
- Phase 2 sparse verification: `./scripts/run-tests.sh t1011-read-tree-sparse-checkout.sh t1090-sparse-checkout-scope.sh t1092-sparse-checkout-compatibility.sh t6428-merge-conflicts-sparse.sh t6435-merge-sparse.sh t3705-add-sparse-checkout.sh t3602-rm-sparse-checkout.sh t7002-mv-sparse-checkout.sh`.
- Results from that run: `t6435-merge-sparse` pass 6/6; `t1011-read-tree-sparse-checkout` 21/23, `t1090-sparse-checkout-scope` 6/7, `t1092-sparse-checkout-compatibility` 48/106, `t6428-merge-conflicts-sparse` 1/2, `t3705-add-sparse-checkout` 15/20, `t3602-rm-sparse-checkout` 7/13, `t7002-mv-sparse-checkout` 4/22.
- Partial clone focus: `./scripts/run-tests.sh t0410-partial-clone.sh` improved to 36/38. Remaining failures are late partial-clone repack/gc/backfill cases after the promisor repack checks.
- Phase 2 partial-clone continuation: `cargo build --release -p grit-cli` passes with existing warnings.
- Focused partial clone: `t5616-partial-clone.sh --run=1-8`, `--run=34`, and `--run=35` pass.
- Harness partial clone: `./scripts/run-tests.sh t5616-partial-clone.sh` is 21/47 after clone/fetch filter work.
- Regression partial clone: `./scripts/run-tests.sh t0410-partial-clone.sh` remains 37/38.
- Phase 2 promisor continuation: `cargo build --release -p grit-cli` passes with existing warnings.
- Focused partial clone: `t5616-partial-clone.sh --run=1-10` passes after blame lazy-fetch hydration.
- Focused partial clone: `t5616-partial-clone.sh --run=1-18` now passes through test 16; remaining failures begin at shallow partial clone/refetch and trace2 maintenance checks.
- Harness partial clone: `./scripts/run-tests.sh t5616-partial-clone.sh` is 24/47 after `fetch-pack --stdin`, `blob:limit` filtering, and refetch negotiation work.
- Pre-commit: `cargo check` passes with existing warnings; `cargo test -p grit-lib --lib` passes 204/204.
- Pre-commit: `cargo clippy --fix --allow-dirty` completed only with the existing warning backlog; clippy reported failed auto-fixes in unrelated files and no scoped clippy changes were kept.
- Phase 2 shallow partial clone: `cargo build --release -p grit-cli` passes with existing warnings.
- Focused partial clone: `t5616-partial-clone.sh --run=1-17` passes, including shallow `clone --depth=1 --filter=blob:none` plus `fetch --refetch --filter=blob:limit=999`.
- Harness partial clone: `./scripts/run-tests.sh t5616-partial-clone.sh` is 26/47 after shallow promisor marker and filtered-refetch marker trimming fixes.
- Pre-commit: `cargo fmt` ran; `cargo check` passes with existing warnings; `cargo test -p grit-lib --lib` passes 204/204.
- Pre-commit: `cargo clippy --fix --allow-dirty` completed after sandbox escalation; it applied one unused-import cleanup and still reports the existing clippy warning backlog plus failed auto-fixes in unrelated files.
- Phase 2 HTTP promisor: focused `t0410-partial-clone.sh --run=38` passes after HTTP lazy fetch keeps received packs as promisor packs.
- Harness partial clone: `./scripts/run-tests.sh t0410-partial-clone.sh` passes 38/38 when run with local HTTP server binding allowed.
- Phase 2 upload-pack filter policy: focused `t5616-partial-clone.sh --run=1-28` passes tests 24-28 after enforcing `uploadpackfilter.*` config.
- Harness partial clone: `./scripts/run-tests.sh t5616-partial-clone.sh` is 32/47 after upload-pack filter policy validation.
- Phase 2 refetch maintenance: focused `t5616-partial-clone.sh --run=1-18` passes with expected `maintenance run --auto --no-quiet --no-detach` trace2 entries and refetch config params.
- Phase 2 transfer fsck trace: focused `t5616-partial-clone.sh --run=1-21` passes, including `index-pack --fsck-objects` tracing for filtered `file://` clone with `transfer.fsckobjects=1`.
- Harness partial clone: `./scripts/run-tests.sh t5616-partial-clone.sh` is 34/47 after refetch maintenance and filtered clone fsck trace fixes.
- Pre-commit: `cargo fmt` ran; `cargo check` passes with existing warnings; `cargo test -p grit-lib --lib` passes 204/204.
- Pre-commit: `cargo clippy --fix --allow-dirty` completed after sandbox escalation; it still reports the existing warning backlog and failed auto-fixes in unrelated files (`bundle_uri_test_tool.rs`, `mergetool.rs`).
- t8 family verification: `./scripts/run-tests.sh t8` passes 105/105 in-scope files (all subtests green). CSV/dashboard refreshed; stale failing counts from prior runs were corrected.
- Submodule focus: `cargo build --release -p grit-cli` passed with existing warnings; focused
  `cd tests && sh t7418-submodule-sparse-gitmodules.sh -v` passed 9/9.
- Submodule harness: `./scripts/run-tests.sh t7418-submodule-sparse-gitmodules.sh` passed 9/9 and
  refreshed `data/test-files.csv` plus dashboards.
- t7418 validation: `cargo fmt` ran; `cargo check -p grit-cli` passed with existing warnings;
  `cargo test -p grit-lib --lib` passed 238/238; `cargo clippy --fix --allow-dirty` completed
  with the existing warning backlog and its unrelated auto-fixes were reverted.
- Submodule focus: `cargo build --release -p grit-cli` passed with existing warnings; focused
  `cd tests && sh t7426-submodule-get-default-remote.sh -v` passed 15/15.
- Submodule harness: `./scripts/run-tests.sh t7426-submodule-get-default-remote.sh` passed 15/15
  and refreshed `data/test-files.csv` plus dashboards.
- t7426 validation: `cargo fmt` ran; `cargo check -p grit-cli` passed with existing warnings;
  `cargo test -p grit-lib --lib` passed 238/238; `cargo clippy --fix --allow-dirty` completed
  with the existing warning backlog and its unrelated auto-fixes were reverted.
- Submodule skipped audit: `cargo build --release -p grit-cli` passed with existing warnings;
  focused `cd tests && GIT_DEFAULT_REF_FORMAT=files sh t7424-submodule-mixed-ref-formats.sh -i -v`
  passed 7/7.
- Submodule harness: after restoring `t7424-submodule-mixed-ref-formats` to `in_scope=yes`,
  `./scripts/run-tests.sh t7424-submodule-mixed-ref-formats.sh` passed 7/7 and refreshed
  `data/test-files.csv` plus dashboards.
- Submodule final-sweep repair: `./scripts/run-tests.sh t7406-submodule-update.sh` is back to
  70/70 after filtering the redundant successful `pull --rebase` stderr line from submodule rebase
  updates; CSV/dashboard refreshed.
- Submodule final verification: `./scripts/run-tests.sh t7400-submodule-basic.sh
  t7401-submodule-summary.sh t7402-submodule-rebase.sh t7403-submodule-sync.sh
  t7406-submodule-update.sh t7407-submodule-foreach.sh t7408-submodule-reference.sh
  t7409-submodule-detached-work-tree.sh t7411-submodule-config.sh
  t7412-submodule-absorbgitdirs.sh t7413-submodule-is-active.sh
  t7414-submodule-mistakes.sh t7416-submodule-dash-url.sh t7417-submodule-path-url.sh
  t7418-submodule-sparse-gitmodules.sh t7419-submodule-set-branch.sh
  t7420-submodule-set-url.sh t7421-submodule-summary-add.sh t7422-submodule-output.sh
  t7423-submodule-symlinks.sh t7424-submodule-mixed-ref-formats.sh
  t7425-submodule-gitdir-path-extension.sh t7426-submodule-get-default-remote.sh
  t7506-status-submodule.sh t7814-grep-recurse-submodules.sh t7112-reset-submodule.sh`
  completed with all covered rows at `failing=0`; `t7814` reports 27/34 and `t7112` reports
  78/82 because upstream TODO cases are tracked separately.
- t7424/t7406 validation: `cargo fmt` ran; `cargo check -p grit-cli` passed with existing
  warnings; `cargo clippy --fix --allow-dirty` completed with the existing warning backlog and its
  unrelated `config.rs`/`filter_process.rs` auto-fixes were reverted; `cargo test -p grit-lib --lib`
  passed 238/238.
