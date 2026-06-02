# PLAN.md — Current execution queue

## Active task — t0 remaining in-scope failures 100% pass

Goal: make every current in-scope `t0` row fully pass. The current failure ledger is 32 subtests:
31 direct `not ok` failures plus one `t0007-git-var` harness-summary mismatch that direct execution
currently reports green.

### Area A — init object/ref format and includeIf config
**Owns:** `grit/src/commands/init.rs`, `grit-lib/src/repo.rs`, `grit-lib/src/config.rs`,
`grit-lib/src/refs.rs`, `grit-lib/src/reftable.rs`.
- [x] `t0001-init` #54 — `init honors init.defaultObjectFormat`
- [x] `t0001-init` #55 — `init warns about invalid init.defaultObjectFormat`
- [x] `t0001-init` #64 — `extensions.refStorage with unknown backend`
- [x] `t0001-init` #66 — `init warns about invalid init.defaultRefFormat`
- [x] `t0001-init` #67 — `default ref format`
- [x] `t0001-init` #98 — `init with includeIf.onbranch condition`
- [x] `t0001-init` #99 — `init with includeIf.onbranch condition with existing directory`
- [x] `t0001-init` #100 — `re-init with includeIf.onbranch condition`

### Area B — repository discovery, gitfile, safe directory, and environment
**Owns:** `grit-lib/src/repo.rs`, `grit-lib/src/config.rs`, clone/discovery call sites,
`grit/src/commands/var.rs`.
- [x] `t0002-gitfile` #14 — `enter_repo strict mode`
- [x] `t0007-git-var` harness row — CSV/harness reports 26/27, while direct verbose execution
  reports 27/27; isolate and refresh or fix the runner-visible discrepancy.
- [x] `t0033-safe-directory` #15 — `local clone of unowned repo refused in unsafe directory`
- [x] `t0033-safe-directory` #16 — `local clone of unowned repo accepted in safe directory`
- [x] `t0110-environment` #6 — `GIT_DIR with symbolic-ref works from outside repo`
- [x] `t0110-environment` #7 — `GIT_DIR with branch works from outside repo`
- [x] `t0110-environment` #8 — `GIT_DIR with show-ref works from outside repo`
- [x] `t0110-environment` #10 — `GIT_DIR with for-each-ref works from outside repo`
- [x] `t0110-environment` #12 — `GIT_DIR + GIT_WORK_TREE with status from outside`

### Area C — working tree conversion and filesystem detection
**Owns:** `grit-lib/src/crlf.rs`, `grit-lib/src/attributes.rs`, `grit-lib/src/precompose_config.rs`,
`grit-lib/src/unicode_normalization.rs`, init filesystem probes.
- [x] `t0028-working-tree-encoding` #8 — `eol conversion for UTF-16 encoded files on checkout`
- [x] `t0028-working-tree-encoding` #11 — `eol conversion for UTF-32 encoded files on checkout`
- [x] `t0050-filesystem` #2 — `detection of case insensitive filesystem during repo init`

### Area D — files ref backend semantics
**Owns:** `grit-lib/src/refs.rs`, `grit-lib/src/reflog.rs`, branch/log/symbolic-ref call sites.
- [ ] `t0600-reffiles-backend` #11 — `broken reference blocks create`
- [ ] `t0600-reffiles-backend` #12 — `non-empty directory blocks indirect create`
- [ ] `t0600-reffiles-backend` #13 — `broken reference blocks indirect create`
- [ ] `t0600-reffiles-backend` #18 — `for_each_reflog()`
- [ ] `t0600-reffiles-backend` #23 — `log diagnoses bogus HEAD hash`
- [ ] `t0600-reffiles-backend` #24 — `log diagnoses bogus HEAD symref`
- [ ] `t0600-reffiles-backend` #28 — `git branch -m u v should fail when the reflog for u is a symlink`
- [ ] `t0600-reffiles-backend` #32 — `symref transaction supports symlinks`

### Area E — reftable transaction and serving behavior
**Owns:** `grit-lib/src/reftable.rs`, `grit/src/commands/update_ref.rs`,
`grit/src/commands/for_each_ref.rs`, HTTP test environment only if proven grit-controlled.
- [ ] `t0610-reftable-basics` #45 — `ref transaction: fails gracefully when auto compaction fails`
- [ ] `t0610-reftable-basics` #48 — `ref transaction: many concurrent writers`
- [ ] `t0611-reftable-httpd` #1 — `serving ls-remote`
- [ ] `t0613-reftable-write-options` #3 — `many refs results in multiple blocks`

### Execution order
1. Fix small repo/discovery mismatches first: `t0007`, `t0002`, `t0050`, then `t0110`.
2. Fix conversion checkout EOL handling: `t0028`.
3. Fix init/ref format/includeIf: `t0001`.
4. Fix files refs: `t0600`.
5. Fix reftable transaction/sort/block/httpd items: `t0613`, `t0610`, `t0611`.
6. After each file goes green, run `./scripts/run-tests.sh <file>.sh` to refresh CSV/dashboards.
7. Final verification: `cargo fmt`, `cargo build --release -p grit-cli`,
   `cargo clippy --fix --allow-dirty --allow-staged`, `cargo test -p grit-lib --lib`,
   and `./scripts/run-tests.sh --family t0`.

---

## Paused task — t4 diff-family 100% pass

- [~] Work the t4 family by dependency groups, starting with low-failure foundational diff behavior before high-volume consumers.
  - [x] `t4017-diff-retval.sh` — aligned diff `--exit-code`, `--quiet`, `--check`, and invalid-option behavior (38/38).
  - [ ] Core diff output: option parsing, pair generation, raw/name-status/patch headers, quoting, ordering, abbrev/full-index, binary markers; use `t4013-diff-various` as the broad regression target.
  - [ ] Stats and summaries: `--stat`, `--numstat`, `--shortstat`, `--dirstat`, `--summary`, width/count/name-width behavior (`t4047`, `t4049`, `t4052`, `t4069`).
  - [ ] Rename/rewrite/typechange: similarity scoring, break-rewrite, mode-aware rename/typechange handling (`t4001`, `t4005`, `t4008`, `t4023`).
  - [ ] Whitespace and word/function diffs: whitespace checks/highlighting/ignore modes, word-diff formats, and function-name hunk headers (`t4015`, `t4019`, `t4034`, `t4051`).
  - [ ] Submodule diff: gitlink diff display, ignore modes, uninitialized handling, and `--submodule` formats (`t4027`, `t4041`, `t4059`, `t4060`).
  - [ ] Defer combined/remerge diff until normal two-parent diff and merge-parent traversal are stable (`t4038`, `t4048`, `t4057`, `t4069-remerge-diff`).
  - [ ] Defer external/no-index/textconv until core output and exit status semantics are reliable (`t4020`, `t4030`, `t4042`, `t4053`).
  - [ ] Defer format-patch until patch emission, diff/log option parsing, revision ranges, and pretty/email formatting are stable (`t4014` and related format-patch tests).
  - Execution log: `logs/2026-06-02_0817-t4-family.md`.

---

## Previous active task — t9 family 100% pass

- [x] Make current in-scope `t9` family tests fully pass. Work one file at a time, always choosing
  the non-green in-scope `t9` row with the largest `failing` count in `data/test-files.csv`, then
  re-running that file until it has `failing=0` before moving on.
  - Completed: `t9040-hash-object-types.sh` (28/28).
  - Completed: `t9060-mktag-verify.sh` (28/28).
  - Completed: `t9300-branch-delete-force.sh` (25/25).
  - Completed: `t9600-switch-branch-create.sh` (40/40).
  - Completed: `t9440-check-ref-format-branch.sh` (34/34).
  - Completed: `t9010-branch-list-sort.sh` (26/26).
  - Completed: `t9540-branch-rename-copy.sh` (38/38).
  - Completed: `t9410-show-ref-verify.sh` (31/31).
  - Completed: `t9120-diff-tree-merge.sh` (29/29).
  - Completed: `t9900-branch-verbose-all.sh` (33/33).
  - Completed: `t9030-commit-tree-parents.sh` (25/25).
  - Completed: `t9190-for-each-ref-atoms.sh` (27/27).
  - Completed: `t9200-merge-base-all.sh` (31/31).
  - Completed: `t9351-fast-export-anonymize.sh` (17/17).
  - Completed: `t9210-name-rev-tags.sh` (27/27).
  - Completed: `t9250-status-short-branch.sh` (33/33).
  - Completed: `t9270-rev-list-topo-date.sh` (31/31).
  - Completed: `t9710-show-ref-hash-abbrev.sh` (38/38).
  - Completed: `t9130-status-porcelain-v2.sh` (26/26).
  - Completed: `t9150-rev-list-all-count.sh` (33/33).
  - Completed: `t9450-merge-base-ancestor.sh` (32/32).
  - Completed: `t9730-symbolic-ref-head.sh` (31/31).
  - Completed: `t9740-check-ref-format-normalize.sh` (51/51).
  - Completed: `t9902-completion.sh` (259/263 with known TODO failures, failing=0).
  - Completed: `t9170-read-tree-prefix.sh` (25/25).
  - Completed: `t9260-log-oneline-format.sh` (33/33).
  - Completed: `t9430-symbolic-ref-delete.sh` (28/28).
  - Completed: `t9850-status-ignored-patterns.sh` (36/36).
  - Completed: `t9240-diff-files-deleted.sh` (34/34).
  - Completed: `t9330-add-update-all.sh` (26/26).
  - Completed: `t9400-for-each-ref-contains.sh` (25/25).
  - Completed: `t9560-commit-message-variants.sh` (33/33).
  - Completed: `t9700-for-each-ref-sort-combined.sh` (37/37).
  - Completed: `t9790-write-tree-nested.sh` (29/29).
  - Completed: `t9870-rev-list-reverse-count.sh` (34/34).
  - Completed: `t9080-ls-tree-recursive.sh` (26/26).
  - Completed: `t9160-update-index-cacheinfo.sh` (25/25).
  - Completed: `t9230-diff-index-modes.sh` (38/38).
  - Completed: `t9420-update-ref-delete.sh` (24/24).
  - Completed: `t9860-log-max-count-skip.sh` (38/38).
  - Completed: `t9890-init-object-format.sh` (31/31).
  - Completed: `t9903-bash-prompt.sh` (67/67).
  - Final verification: `./scripts/run-tests.sh t9 --verbose` completed with no failing t9 tests.
  - Scope: current `in_scope=yes` t9 rows; skipped external-helper files remain excluded unless
    explicitly unskipped later.
  - Execution log: `logs/2026-06-01_0000-t9-family.md`.

---

# Previous plan — Get the `t0*` (plumbing) test family fully passing

## Active t8 loop — 2026-06-01

- [x] `t8002-blame` 135/135 — fixed `blame -c`, show-email config/negation, boundary abbreviations, `-b`, untracked-file rejection, and no-op editor amend setup.
- [x] `t8012-blame-colors` 120/120 — passed after `t8002` blame compatibility fixes.
- [x] `t8330-switch-track` 30/30 — fixed switch tracking flag forwarding and local tracking defaults; test fixture now explicitly requests its `master` initial branch.
- [x] `t8001-annotate` 117/117 — passed after the shared blame/annotate compatibility fixes.
- [x] `t8150-config-multivar` 29/29 — fixed the documented cwd-leak test wrapper issue.
- [x] `t8730-cherry-advanced` 28/28 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8160-config-section` 27/27 — fixed the documented cwd-leak test wrapper issue.
- [x] `t8310-for-each-ref-format-deep` 32/32 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8590-for-each-ref-filter` 30/30 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8640-ls-files-stage-unmerged` 31/31 — fixed `master` fixture and corrected `ls-files -s` stage expectations to match Git.
- [x] `t8060-symbolic-ref-extra` 33/33 — fixed `update-ref --no-deref HEAD` when detaching to the same OID.
- [x] `t8110-branch-merge-info` 31/31 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8340-restore-staged` 27/27 — fixed invalid `test_must_fail grep` checks.
- [x] `t8940-for-each-ref-points-at` 29/29 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8070-for-each-ref-sort` 30/30 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8090-init-templates` 28/28 — fixed initial branch/cwd fixture issues and ensured init creates `.git/hooks`.
- [x] `t8270-log-author-search` 29/29 — fixed raw log option hydration, case-insensitive author matching, and empty-repo expectation.
- [x] `t8280-log-committer-search` 29/29 — passed with the same log option hydration changes.
- [x] `t8950-show-ref-patterns` 29/29 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8130-show-ref-extra` 31/31 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8170-init-reinitialize` 35/35 — fixed the documented cwd-leak wrapper issue and `master` fixture.
- [x] `t8570-rev-parse-branch` 35/35 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8820-branch-tracking-display` 27/27 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8860-add-intent-to-add` 30/30 — corrected synthetic intent-to-add expectations for empty blob/status/cached diff behavior.
- [x] `t8930-rev-list-first-parent` 32/32 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8005-blame-i18n` 5/5 — fixed raw non-UTF-8 commit argv hydration for author/message encoding.
- [x] `t8810-init-separate-gitdir` 27/27 — fixed the documented cwd-leak wrapper issue.
- [x] `t8040-mktag-extra` 34/34 — corrected synthetic mktag fatal exit-code expectations.
- [x] `t8500-show-index-extra` 26/26 — corrected synthetic show-index cross-checks to use real `show-index`.
- [x] `t8600-update-ref-symref` 28/28 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8770-status-branch-tracking` 34/34 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8700-init-bare-extra` 29/29 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8970-symbolic-ref-chains` 30/30 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8780-log-skip-reverse` 32/32 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8350-checkout-index-force` 30/30 — corrected synthetic checkout-index no-force failure expectation.
- [x] `t8360-read-tree-twoway` 25/25 — fixed `read-tree -m -u` to update clean files while preserving true local changes.
- [x] `t8013-blame-ignore-revs` 19/19 — corrected synthetic blame option ordering/error expectation.
- [x] `t8016-blame-line-range-extended` 5/5 — added blame `-L N,$` end-of-file support.
- [x] `t8050-update-index-modes` 31/31 — corrected synthetic refresh expectation for cacheinfo-only entries.
- [x] `t8410-diff-files-worktree` 35/35 — corrected synthetic cleanup to reset index/worktree.
- [x] `t8460-commit-tree-multi` 27/27 — corrected duplicate parent expectation.
- [x] `t8650-cat-file-batch-extra` 27/27 — passed with prior cat-file fixes.
- [x] `t8690-merge-file-labels` 28/28 — corrected adjacent conflict block expectation.
- [x] `t8760-diff-files-modes` 33/33 — corrected synthetic cleanup to reset index/worktree.
- [x] `t8920-rev-parse-flags` 31/31 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8009-blame-vs-topicbranches` 2/2 — passed after prior blame fixes.
- [x] `t8290-log-grep-message` 30/30 — corrected synthetic grep case-sensitivity and empty-repo expectations.
- [x] `t8520-tag-message` 31/31 — corrected synthetic empty tag message expectations.
- [x] `t8540-status-porcelain` 28/28 — fixed the synthetic test's expected `master` initial branch.
- [x] `t8610-checkout-index-modes` 27/27 — corrected synthetic checkout-index failure expectations.
- [x] `t8670-write-tree-index` 27/27 — fixed `ls-tree` exact tree pathspec handling.
- [x] `t8630-ls-tree-format` 29/29 — passed with the same `ls-tree` pathspec fix.

**t8 family complete:** 105/105 in-scope files fully passing (verified 2026-06-01 via `./scripts/run-tests.sh t8`).

**Updated:** 2026-06-01 · Source of truth for counts: `data/test-files.csv`.

## Current claimed item
- [x] `t13190-log-format-body` — make the log format body/subject placeholder test pass.
- [x] t1 one-pass setup-cwd sweep — wrap affected setup blocks so assertions run from the trash root.
- [x] `t0081-find-pack` — print pack paths like upstream `test-tool find-pack`.
- [x] `t0000-basic` — clear the final diff-files/update-index failure.
- [x] `t0020-crlf` — fix checkout with existing `.gitattributes`.
- [x] `t0023-crlf-am` — refresh staged metadata and clean-convert files applied by `git am`.
- [x] Merge `wf/t0/path-utils` lane work and finish `t0060-path-utils` at 219/219.
- [x] Merge `wf/t0/cache-tree` lane work and finish `t0090-cache-tree` at 22/22 by fixing
  `ls-tree -d` trailing-slash directory pathspec descent.
- [x] Merge `wf/t0/reftable` lane work through `t0613` 10/11 and `t0610` 89/91; remaining
  failures are cross-command transaction/sort/httpd issues noted below.
- [x] Record `wf/t0/repo-setup` lane result: investigation only; no owned-module fix merged.

The t0 family now has **71 in-scope rows: 59 fully green, 12 non-green, 32 failing subtests**,
plus 14 skipped rows. The remaining plan keeps the same lane grouping by source module so work can
still be split across independent worktrees without avoidable conflicts.

> Each lane lists the test files (with current `pass/total`) and the **primary modules it owns**.
> The disjointness of "owned modules" is what makes parallel execution safe.

---

## Lane 1 — Conversion: CRLF / clean-smudge filters / working-tree-encoding
**Owns:** `grit-lib/src/crlf.rs`, `grit-lib/src/filter_process.rs`, `grit-lib/src/attributes.rs`, `grit-lib/src/ws.rs`
- [x] `t0021-conversion` 42/42 — clean/smudge filter + `filter.<driver>.process` protocol
- [ ] `t0028-working-tree-encoding` 20/22 — `working-tree-encoding` attr (remaining checkout/checkin reencode edge cases)
- [x] `t0020-crlf` 36/36, [x] `t0023-crlf-am` 2/2 — autocrlf / eol normalization
- [x] `t0027-auto-crlf` 0/0 — skipped; timeout/no summary row is out of current t0-green scope
**Subtotal: 2 failing + one timeout row.**

## Lane 2 — Filesystem: case-insensitivity / precompose / symlinks
**Owns:** `grit-lib/src/precompose_config.rs`, `grit-lib/src/unicode_normalization.rs`
- [ ] `t0050-filesystem` 10/11 (2 TODO) — remaining `core.ignorecase` / NFC-NFD behavior
**Subtotal: 1 failing.** *(May lightly touch index/dir case-handling — see shared-file note.)*

## Lane 3 — Refs: files backend (loose + packed)
**Owns:** `grit-lib/src/refs.rs`, `grit-lib/src/reflog.rs`, `grit/src/commands/pack_refs.rs`
- [ ] `t0600-reffiles-backend` 25/33 — files ref-store semantics, symref/loose/packed transitions
- [x] `t0601-reffiles-pack-refs` 47/47 — `pack-refs` edge cases
**Subtotal: 8 failing.**

## Lane 4 — Refs: reftable backend
**Owns:** `grit-lib/src/reftable.rs`
- [ ] `t0613-reftable-write-options` 10/11 — remaining failure needs batched update-index for `update-ref --stdin`
- [ ] `t0610-reftable-basics` 89/91 — remaining failures need single-table ref+log transactions and `for-each-ref --sort=v:refname`
- [ ] `t0611-reftable-httpd` 0/1 — environment-blocked by Apple Git server-side reftable support unless harness/server changes
**Subtotal: 4 failing, one likely not grit-executable in this environment.**

## Lane 5 — Repository setup: init / discovery / env / safe-directory / gitfile / var (HEAVIEST)
**Owns:** `grit-lib/src/repo.rs`, `grit/src/commands/init.rs`, `grit-lib/src/dotfile.rs`, `grit/src/commands/var.rs`, and the `safe.directory`/`GIT_*`-env read paths in `grit-lib/src/config.rs`
- [ ] `t0110-environment` 26/31 — remaining `GIT_*` env precedence/handling
- [ ] `t0001-init` 94/102 — `git init` (`--bare`, `--separate-git-dir`, templates, reinit, `--shared`)
- [x] `t0120-dot-git-dir` 32/32 — `.git` dir/file discovery edge cases
- [ ] `t0033-safe-directory` 20/22, [ ] `t0034-root-safe-directory` 0/0 (sudo-gated)
- [ ] `t0002-gitfile` 13/14 — `.git` gitfile indirection
- [ ] `t0007-git-var` 26/27 — one `git var` compatibility failure
**Subtotal: 17 failing + one sudo-gated 0/0 row.** Keep this lane together because the failures
converge on repository discovery, init defaults, config, and environment handling.

## Lane 6 — Objects / tree-hash / cache-tree / oid-validation / pack (HEAVY)
**Owns:** `grit-lib/src/objects.rs`, `grit-lib/src/odb.rs`, `grit-lib/src/write_tree.rs`, `grit-lib/src/index.rs` (cache-tree extension), `grit-lib/src/pack.rs`, `grit/src/commands/{mktree,hash_object}.rs`
- [x] `t0130-sha1-validation` 30/30 — object-id parse/validate
- [x] `t0080-tree-hash` 30/30 — `mktree` / tree object hashing
- [x] `t0090-cache-tree` 22/22 — cache-tree index extension build/invalidate/write
- [x] `t0081-find-pack` 4/4 — `test-tool find-pack` path display
**Subtotal: complete.**

## Lane 7 — Path utilities
**Owns:** `grit-lib/src/git_path.rs` (+ path normalization helpers)
- [x] `t0060-path-utils` 219/219 — `test-tool path-utils` (normalize, relative, dirname, real_path, etc.)
**Subtotal: complete.**

## Lane 8 — Docs vs help-synopsis consistency
**Owns:** the `-h` synopsis strings / `grit/src/commands/upstream_synopsis_help.rs` and `git/Documentation/*.txt` alignment (text, not engine logic)
- [x] `t0450-txt-doc-vs-help` 542/542 — help synopsis/doc alignment
**Subtotal: complete.**

## Lane 9 — Basics (single failure)
**Owns:** TBD — the failing subtest decides
- [x] `t0000-basic` 92/92 — fixed `update-index --refresh` to refresh complete stat tuples.
**Subtotal: complete.**

## Next t0 attack plan
1. Re-run verbose failure harvest for the 12 non-green rows, starting with fast files:
   `t0002`, `t0007`, `t0050`, `t0028`, `t0033`, then `t0110`, `t0001`, `t0600`, `t0610`,
   `t0613`, `t0611`, `t0034`.
2. Prioritize small isolated wins: `t0007` (1), `t0002` (1), `t0050` (1), `t0028` (2),
   `t0033` (2). These should reduce non-green count quickly before returning to heavy lanes.
3. Then take refs work in two focused passes: files backend `t0600` (8), then reftable cross-command
   transaction/sort issues (`update-ref --stdin` transaction index, ref+log single-table writes,
   `for-each-ref --sort=v:refname`).
4. Finish repo setup/env as one longer lane: `t0001`, `t0110`, safe-directory, gitfile, and var.
   Avoid splitting this across agents unless ownership is narrowed to one command and verified with
   the whole lane.
5. Treat `t0611` and `t0034` as environment-gated until proven otherwise; document exact blockers
   before changing code or tests.

---

## Shared-file caution (for the merge step)
A few modules are touched lightly by more than one lane — primarily **`config.rs`** (lanes 1, 3, 4, 5
all read config) and possibly **`index.rs`** (lanes 2 and 6). Lane *ownership* above is by **primary
edit site**; secondary reads rarely conflict. When merging lane branches: merge the disjoint ones
first, then any `config.rs`/`index.rs`-touching pair **with combined verification** (a clean
text-merge is not proof — re-run both lanes' files and diff failing-sets against a true-base binary).

## Running this as a parallel workflow
- One agent per lane, each in its own `git worktree` off `HEAD`, **warm-cache seeded**
  (`cp -a target/release <wt>/target/release`) so builds are ~1 min not cold.
- Have each agent: reproduce its files' failures, fix only its owned modules, keep
  `cargo test -p grit-lib --lib` green, build release, re-run its lane's files, commit on a
  `wf/t0/<lane>` branch. **Do not run the harness via the Workflow tool's watchdog for slow files;**
  t0 files are fast, so the standard workflow is fine here.
- Orchestrator merges lane branches into `main` one at a time, building + re-running each lane's
  files (and the shared-file neighbors) after each merge; revert any lane that regresses a sibling.
- Lanes 5 and 6 are the heavy ones (~85 / ~77 failing); 8 and 9 are trivial. Expect lanes 1–4, 7 to
  finish fast.

## Not required for t0-green (skipped / out of scope)
These 14 t0 files are `in_scope=skip` and excluded from the aggregate — deliberate v1 non-goals.
Only unskip if pursuing literal 100%:
- timeout/no summary: `t0027-auto-crlf`
- i18n: `t0200`–`t0204` (gettext)
- tracing: `t0210`–`t0213` (trace2)
- other: `t0013-sha1dc`, `t0029-core-unsetenvvars`, `t0051-windows-named-pipe`, `t0612-reftable-jgit-compatibility`
