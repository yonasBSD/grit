# PLAN.md — Current execution queue

## Active task — t2 family 100% pass

- [~] Make all `t2` family tests fully pass. Work one file at a time, always choosing the
  non-green in-scope `t2` row with the largest `failing` count in `data/test-files.csv`, then
  re-running that file until it has `failing=0` before moving on. After all current in-scope rows
  pass, audit skipped t240x worktree rows so literal t2 completion is not hidden behind skips.
  - Current first target: `t2050-checkout.sh` (8/80, failing=72 at claim time).
  - Execution log: `logs/2026-06-01_2000-t2-family.md`.

---

## Active task — t9 family 100% pass

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

The t0 family has **85 files: 47 fully green, 25 in-scope-not-full (~247 failing subtests),
13 skipped**. This plan splits the 25 remaining in-scope files into **work lanes grouped by the
source modules they touch**, so the lanes can run **in parallel** (one agent per lane, each in its
own git worktree) with minimal cross-lane merge conflict. Within a lane the files share code, so a
single agent should own the whole lane.

> Each lane lists the test files (with current `pass/total`) and the **primary modules it owns**.
> The disjointness of "owned modules" is what makes parallel execution safe.

---

## Lane 1 — Conversion: CRLF / clean-smudge filters / working-tree-encoding
**Owns:** `grit-lib/src/crlf.rs`, `grit-lib/src/filter_process.rs`, `grit-lib/src/attributes.rs`, `grit-lib/src/ws.rs`
- [~] `t0021-conversion` 28/42 — clean/smudge filter + `filter.<driver>.process` protocol
- `t0028-working-tree-encoding` 8/22 — `working-tree-encoding` attr (iconv reencode on checkout/checkin)
- [x] `t0020-crlf` 36/36, [x] `t0023-crlf-am` 2/2 — autocrlf / eol normalization
- `t0027-auto-crlf` 0/0 — **runs 0 tests; investigate** (errors out or all-prereq-skip before summary)
**Subtotal: ~31 failing.**

## Lane 2 — Filesystem: case-insensitivity / precompose / symlinks
**Owns:** `grit-lib/src/precompose_config.rs`, `grit-lib/src/unicode_normalization.rs`
- `t0050-filesystem` 8/13 — `core.ignorecase`, NFC/NFD precompose, beyond-symlink behavior
**Subtotal: ~5 failing.** *(May lightly touch index/dir case-handling — see shared-file note.)*

## Lane 3 — Refs: files backend (loose + packed)
**Owns:** `grit-lib/src/refs.rs`, `grit-lib/src/reflog.rs`, `grit/src/commands/pack_refs.rs`
- `t0600-reffiles-backend` 15/33 — files ref-store semantics, symref/loose/packed transitions
- `t0601-reffiles-pack-refs` 45/47 — `pack-refs` edge cases
**Subtotal: ~20 failing.**

## Lane 4 — Refs: reftable backend
**Owns:** `grit-lib/src/reftable.rs`
- `t0613-reftable-write-options` 4/11 — block size / restart / compaction write options
- `t0610-reftable-basics` 89/91 — 2 remaining basics
- `t0611-reftable-httpd` 0/1 — single test; likely httpd-env, confirm not-grit before chasing
**Subtotal: ~10 failing.**

## Lane 5 — Repository setup: init / discovery / env / safe-directory / gitfile / var (HEAVIEST)
**Owns:** `grit-lib/src/repo.rs`, `grit/src/commands/init.rs`, `grit-lib/src/dotfile.rs`, `grit/src/commands/var.rs`, and the `safe.directory`/`GIT_*`-env read paths in `grit-lib/src/config.rs`
- `t0110-environment` 3/31 — `GIT_*` env var precedence/handling (big gap)
- `t0001-init` 74/102 — `git init` (`--bare`, `--separate-git-dir`, templates, reinit, `--shared`)
- `t0120-dot-git-dir` 8/32 — `.git` dir/file discovery edge cases
- `t0033-safe-directory` 20/22, `t0034-root-safe-directory` 0/0 (**sudo-gated**: runs only with `GIT_TEST_ALLOW_SUDO`)
- `t0002-gitfile` 12/14 — `.git` gitfile indirection
- `t0007-git-var` 26/27 — `git var` (1 failing)
**Subtotal: ~85 failing.** Heaviest lane; do NOT split (everything here converges on `repo.rs`).
Consider giving this lane a longer iteration budget.

## Lane 6 — Objects / tree-hash / cache-tree / oid-validation / pack (HEAVY)
**Owns:** `grit-lib/src/objects.rs`, `grit-lib/src/odb.rs`, `grit-lib/src/write_tree.rs`, `grit-lib/src/index.rs` (cache-tree extension), `grit-lib/src/pack.rs`, `grit/src/commands/{mktree,hash_object}.rs`
- `t0130-sha1-validation` 1/30 — object-id parse/validate, `GIT_TEST_BUILTIN_HASH`, fsck-ish id checks (big gap)
- `t0080-tree-hash` 3/30 — `mktree` / tree object hashing (big gap)
- [ ] `t0090-cache-tree` 16/22 — cache-tree index extension build/invalidate/write; remaining failures are partial/interactive commit patch semantics and checkout cache-tree shape edge cases
- [x] `t0081-find-pack` 4/4 — `test-tool find-pack` path display
**Subtotal: ~77 failing.** Grouped because they all touch `objects.rs`/`write_tree.rs`/`index.rs`.

## Lane 7 — Path utilities
**Owns:** `grit-lib/src/git_path.rs` (+ path normalization helpers)
- `t0060-path-utils` 206/219 — `test-tool path-utils` (normalize, relative, dirname, real_path, etc.)
**Subtotal: ~13 failing.**

## Lane 8 — Docs vs help-synopsis consistency
**Owns:** the `-h` synopsis strings / `grit/src/commands/upstream_synopsis_help.rs` and `git/Documentation/*.txt` alignment (text, not engine logic)
- `t0450-txt-doc-vs-help` 537/542 — 5 commands whose `-h` synopsis doesn't match their doc
**Subtotal: ~5 failing.** No engine overlap with any other lane.

## Lane 9 — Basics (single failure)
**Owns:** TBD — the failing subtest decides
- [x] `t0000-basic` 92/92 — fixed `update-index --refresh` to refresh complete stat tuples.
**Subtotal: 1 failing.**

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
These 13 t0 files are `in_scope=skip` and excluded from the aggregate — deliberate v1 non-goals.
Only unskip if pursuing literal 100%:
- i18n: `t0200`–`t0204` (gettext)
- tracing: `t0210`–`t0213` (trace2)
- other: `t0013-sha1dc`, `t0029-core-unsetenvvars`, `t0051-windows-named-pipe`, `t0612-reftable-jgit-compatibility`
