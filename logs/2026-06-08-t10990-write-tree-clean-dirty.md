# t10990-write-tree-clean-dirty — 2026-06-08 (schacon+opus-t5)

Ticket: 1b50d6. Status going in: blocked, 36/37, with two prior agents (c065e8236,
b2cb8500b) both concluding subtest 21 is a TEST BUG, not a grit bug.

## Task
File: tests/t10990-write-tree-clean-dirty.sh (1 failing).
Failing subtest 21: "write-tree after update-index remove" — asserts
`! grep "ui.txt" actual` after `grit update-index --remove ui.txt` while ui.txt is
present on disk AND tracked.

## Independent re-verification (third pass)

1. Fresh run: `./scripts/run-tests.sh t10990-write-tree-clean-dirty.sh` => 36/37,
   subtest 21 the only failure (matches recorded state).

2. C source ground truth — git/builtin/update-index.c:
   - `update_one` (462): not force_remove => lstat(path). File present => stat_errno=0,
     calls `process_path(path,&st,0)`.
   - `process_path` (380): not skip-worktree, stat_errno==0, not a dir =>
     `return add_one_path(ce,path,len,st)` (413).
   - `add_one_path` (282): entry up-to-date (`!ie_match_stat`) => return 0 — KEEPS entry.
   - `remove_one_path` (259) is reached ONLY from `process_lstat_error` (275), i.e. when
     the file is MISSING on disk. `--force-remove` (495) is the unconditional remover.
   => Plain `--remove` does NOT drop a present, tracked path. Ticket premise is false.

3. Direct A/B with real git 2.52.0 (`/tmp/t10990ab`): built the exact subtest-21 state
   (ui.txt added via `git add`, present on disk + tracked) in two repos, ran
   `update-index --remove ui.txt` with real git and with grit:
   - real git: exit 0, ui.txt STAYS in ls-files (1) and in write-tree (1).
   - grit:     exit 0, ui.txt STAYS in ls-files (1) and in write-tree (1).
   Byte-identical. grit matches git. The subtest's `! grep ui.txt` would FAIL against
   real git too.

4. grit code review — grit/src/commands/update_index.rs:844-865: plain Remove path mode
   removes the entry only when `symlink_metadata` errors (file gone); otherwise falls
   through to the normal stat/update path, exactly mirroring git's process_path/add_one_path.
   Code is correct.

5. Test provenance: no git/t/t10990* upstream; the file mixes `$REAL_GIT` and `grit`.
   It is grit-authored. Subtest is already `test_expect_success`, so the only permitted
   edit (expect_failure->expect_success) does not apply, and per the rules I cannot
   change the assertion itself.

## Verdict
No grit change warranted. Subtest 21 encodes an incorrect expectation that contradicts
real git. Staying blocked. 36/37 is the correct outcome for grit's behavior.
