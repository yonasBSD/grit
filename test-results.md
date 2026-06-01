# Test Results

Updated: 2026-06-01

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
