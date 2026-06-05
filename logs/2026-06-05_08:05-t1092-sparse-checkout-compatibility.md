# t1092 sparse-checkout compatibility

## 2026-06-05

- Claimed ticket `0e0b0d`.
- Starting from a clean GitButler workspace on `grit-t1-progress`.
- Refreshing `t1092-sparse-checkout-compatibility.sh` before inspecting failures.
- Baseline remained `63/106`.
- Moved sparse checkout warning detection before worktree updates in `checkout`; harness improved to
  `67/106`, clearing false warnings for files checkout created inside the sparse cone.
- Taught checkout to recognize `--patch` after the tree-ish, accepted hyphen-leading commit
  messages (`commit -m "-a"`), skipped absent skip-worktree entries during `commit -a`, allowed
  `add --refresh` to refresh sparse entries, and honored `add --sparse .` for out-of-cone paths.
- Latest harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `70/106`.
  Ticket remains open; next direct failure is still within `status/add: outside sparse cone`.
- After commit `c8a1bc3`, direct verbose execution showed subtest 15 now passes; first direct
  failure moved to subtest 18 (`diff with renames and conflicts`).
- Found that `checkout <current-branch>` rebuilt the index whenever staged changes made it differ
  from HEAD. That rejected staged D/F changes in the full checkout while sparse checkouts skipped
  the path. Adjusted the already-on-branch path to preserve staged work unless forced, sparse
  reapply is needed, or the index is empty.
- A follow-up direct run showed sparse checkout still failed the same loop because current-branch
  checkout re-applied sparse rules even with staged D/F changes. Narrowed current-branch checkout
  further: only force or an empty index rebuilds; ordinary `checkout <current-branch>` preserves
  staged work.
- Direct execution then passed subtest 18 and failed subtest 19. The remaining mismatch was a
  tracked D/F descendant (`folder2/0/1/1`) still marked skip-worktree in sparse repos after
  restoring `folder2/0/1` as a file from another tree. Added a path-checkout helper that clears
  skip-worktree on tracked descendants that can no longer exist on disk.
- Direct execution then passed subtest 19 and failed subtest 22. `blame` was allowing a missing
  working-copy path to proceed when the index knew the path; Git's no-revision working-copy blame
  lstat check fails immediately for missing sparse paths. Tightened that guard.
- Direct execution then passed subtest 22 and failed subtest 26. `reset base -- nonexistent-file`
  should be a no-op for an explicit non-HEAD tree-ish, while `reset HEAD -- nonexistent` remains an
  error. Narrowed the unmatched pathspec behavior accordingly.
- Direct execution now passes through subtest 35 and stops at read-tree subtest 36. Canonical
  harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `74/106`.
- The read-tree failure was `read-tree -m -u base HEAD update-folder2` rejecting sparse checkouts
  because `require_uptodate` treated missing skip-worktree entries as local changes. Missing
  skip-worktree paths are intentionally up to date in sparse checkouts, so `read-tree` now accepts
  them while still checking present files.
- Direct execution now passes through subtest 41 and stops at subtest 42
  (`merge, cherry-pick, and rebase`). Canonical harness:
  `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `78/106`.
- The subtest 42 merge failure was caused by sparse-index placeholders being collapsed before the
  merge commit tree was written; expanding the index for tree writing preserves out-of-cone files.
- The next subtest 42 failure was sparse-index cherry-pick. Cherry-pick applied sparse rules to all
  out-of-cone paths and then tried to write sparse-directory placeholders during checkout. Changed
  cherry-pick to clear skip-worktree for paths changed by the replay, expand placeholders before
  commit-tree writing, and skip sparse stage-0 entries/placeholders during worktree checkout.
- Focused sparse-index cherry-pick of `update-folder1` now succeeds and materializes `folder1/a`
  while keeping unchanged out-of-cone entries sparse.
- Canonical harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `80/106`.
  Direct execution now passes subtest 42 and exposes later conflict-resolution failures.
- Subtest 43 (`merge with conflict outside cone`) left `folder2/a` unmerged after
  `mv folder2/a folder2/z && git add --sparse folder2`; directory adds now remove absent
  unmerged entries under the added directory, resolving the renamed conflict path.
- `merge --continue` also needed sparse placeholder expansion for the commit tree after its index
  write collapsed sparse directories. Focused subtest 43 reproduction now matches full, sparse, and
  sparse-index repositories through status and tree checks.
- Canonical harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `82/106`.
- Subtest 47 (`stash`) created sparse stash commits that omitted out-of-cone tracked paths because
  sparse-directory placeholders were written directly to stash trees. Stash snapshots now expand
  sparse placeholders before tree writing, and working-tree stash snapshots preserve absent
  skip-worktree entries instead of treating them as deletions.
- `stash apply --index` also left restored out-of-cone entries marked skip-worktree, hiding the
  expected worktree deletions from sparse status. Clearing skip-worktree for stash-touched paths
  makes the status match full checkout.
- Canonical harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `83/106`.
- Subtests 48 and 49 (`checkout-index`) exposed that `checkout-index -- <path>` printed
  "already exists, no checkout" for a modified existing file but still exited successfully. Git
  treats that as a failed checkout unless `-f` is provided, so the command now returns an error in
  that path while preserving the force behavior.
- Canonical harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `85/106`.
- Subtest 52 (`clean`) removed the empty sparse-present `folder1` directory after deleting its
  ignored file. Empty-parent pruning now stops at directories that are tracked prefixes in the
  index, including skip-worktree sparse entries.
- Canonical harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `86/106`.
- Subtest 56 (`git apply functionality`) had correct behavior but mismatched sparse stderr because
  the missing outside-cone worktree file error included absolute paths with the repo directory
  name. The apply preimage stat check now reports the adjusted repo-relative path.
- Canonical harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `87/106`.
- The first trace2 sparse-index check regressed badly with broad repository-level index trace
  hooks, so that approach was backed out. Narrow command-level trace regions now report the
  expansion/conversion case for `reset -- folder1/a`, plain `ls-files` expansion, and
  `status -c index.sparse=false`/conversion writes without tripping later `ensure_not_expanded`
  checks.
- Canonical harness: `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `88/106`.
- Follow-up direct run showed subtest 58 still missed the first `ensure_full_index` region when the
  sparse index had placeholder entries but the in-memory `sparse_directories` flag was false. Status
  now detects sparse-index-on-disk from actual placeholder entries too. Direct `--run=1,57,58,59`
  passes subtests 57 and 58, then fails at the pre-existing subtest 59 block; full harness remains
  `88/106`.
- Subtest 59 then failed at `restore -s rename-out-to-out -- deep/deeper1` because restore treated a
  literal source-tree directory as a single blob path. Tree-source restore now expands literal
  directory pathspecs to contained file paths. The reset trace detector also no longer reports
  expansion for trailing-slash directory pathspecs like `folder1/`.
- Direct `--run=1,59` passes, and canonical harness:
  `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `89/106`.
- Subtest 15 (`status/add: outside sparse cone`) mismatched the long sparse-checkout banner after
  materializing `folder1/a` outside the cone: a full sparse-checkout index reported a tracked-file
  percentage, while sparse-index always used the short sparse banner. Status now remembers the raw
  sparse-directory prefixes before expanding them and only keeps the short sparse-index banner when
  no expanded sparse path became present/non-skip-worktree.
- Direct `--run=1,15` passes, and canonical harness:
  `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `90/106`.
- Subtest 71 (`ls-files`) failed for `ls-files --sparse --modified` after materializing and editing
  `folder1/a` outside the sparse cone. Sparse ls-files now expands placeholders only for the
  working-tree comparison modes (`--modified`/`--deleted`) and clears skip-worktree for present
  files before comparing stats, while plain `ls-files --sparse` still preserves placeholders.
- Direct `--run=1,71` passes, and canonical harness:
  `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `91/106`.
- Subtests 75 and 76 (`checkout behaves oddly with df-conflict-*`) documented Git's unusual
  branch-checkout stdout for staged directory/file conflicts. Checkout now records staged paths
  dropped because a target file replaces their parent directory and prints the matching `D`/`A`
  path updates for non-sparse-index cases, while preserving sparse-index's quiet output.
- Direct `--run=1,75,76` passes, and canonical harness:
  `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `93/106`.
- Subtest 79 (`rm pathspec outside sparse definition`) exposed that `git rm --sparse folder1/*`
  on a sparse index expanded the `folder1/` placeholder and printed each child removal. `rm` now
  preserves raw sparse-directory placeholders as the porcelain removal unit when pathspecs select
  them, while still removing expanded children from the index.
- Direct `--run=1,79` passes, and canonical harness:
  `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `94/106`.
- Subtests 80 and 81 check sparse-index expansion trace behavior for `rm`. `rm` now mirrors Git's
  pathspec expansion decision for trace2: in-cone literal/wildcard removals stay quiet, while
  pathspecs that may need to inspect partial contents of sparse-directory placeholders emit
  `index/ensure_full_index`.
- Direct `--run=1,80,81` passes, and canonical harness:
  `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `95/106`.
- Subtest 84 (`grep within submodules is not expanded`) failed because recursive cached grep did
  not descend into submodules for a superproject wildcard pathspec like `*/folder1/*`. The
  submodule descendant probe now synthesizes a candidate from the pathspec tail, allowing the grep
  to recurse and find sparse-directory matches without emitting an index expansion trace.
- Direct `--run=1,84` and `--run=1,85` pass, and canonical harness:
  `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh` -> `96/106`.
- 2026-06-05 15:01 CEST: Subtests 93, 97, 99, and 100 were advanced. Worktree add/remove now
  reports user-supplied relative paths, cached diff checks load missing sparse `.gitattributes`
  rules from the index, apply skips worktree preimage reads for missing outside-cone files while
  still emitting sparse-index expansion traces, reset preserves a partially expanded sparse-index
  shape across `reset --hard`, and interactive add emits expansion traces for outside-cone patch
  selections.
- Direct `--run=1,99` and `--run=1,100` pass. Direct `--run=1,101,102,103,104,105,106` now
  reaches subtest 101 and fails at `git add .` sparse-path advice before the checkout/reset patch
  trace assertions. Canonical harness:
  `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh --verbose --timeout 180` ->
  `99/106`.
- 2026-06-05 15:28 CEST: Subtest 101 now passes. `git add .` no longer emits sparse-path advice
  when it staged an in-cone match and only skipped outside-cone materialized paths. `reset --patch`
  and `checkout --patch` now emit `index/ensure_full_index` only for sparse patch candidates that
  require index expansion, including partially expanded outside-cone paths.
- Direct `--run=1,101` passes. Canonical harness:
  `./scripts/run-tests.sh t1092-sparse-checkout-compatibility.sh --verbose --timeout 180` ->
  `101/106`.
