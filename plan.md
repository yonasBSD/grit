# PLAN.md — Current execution queue

## Active task — t9 family 100% pass

- [~] Make current in-scope `t9` family tests fully pass. Work one file at a time, always choosing
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
  - Next expected target: choose by refreshed CSV after commit.
  - Scope: current `in_scope=yes` t9 rows; skipped external-helper files remain excluded unless
    explicitly unskipped later.
  - Execution log: `logs/2026-06-01_0000-t9-family.md`.

---

# Previous plan — Get the `t0*` (plumbing) test family fully passing

**Updated:** 2026-06-01 · Source of truth for counts: `data/test-files.csv`.

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
- `t0021-conversion` 27/42 — clean/smudge filter + `filter.<driver>.process` protocol
- `t0028-working-tree-encoding` 8/22 — `working-tree-encoding` attr (iconv reencode on checkout/checkin)
- `t0020-crlf` 35/36, `t0023-crlf-am` 1/2 — autocrlf / eol normalization
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
- `t0090-cache-tree` 2/22 — cache-tree index extension build/invalidate/write (big gap)
- `t0081-find-pack` 3/4 — `cat-file --find-pack` (1 failing)
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
- `t0000-basic` 91/92 — one failing subtest; agent first identifies which subsystem it is, then fixes
  it there. Likely object/index/`test_must_fail` plumbing. Keep small.
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
