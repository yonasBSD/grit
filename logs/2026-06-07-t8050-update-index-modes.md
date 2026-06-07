# t8050-update-index-modes — MOP-UP ROUND 2 (ticket 62e3f9)

Date: 2026-06-07T09:34Z

## Result
Still 29/31. No grit code change warranted. Re-verified the prior diagnosis
end-to-end against real git AND the grit binary AND the canonical C source.

## Failing subtests
- #5 `update-index --remove removes a file from index`
- #11 `ls-files --stage shows all entries with modes` (pure cascade of #5)

## Re-verification (fresh, post other-agents' fixes)
1. Built `target/release/grit` (-j4) and ran `./scripts/run-tests.sh
   t8050-update-index-modes.sh` -> 29/31. Direct verbose run confirms ONLY #5
   and #11 fail; all 29 others pass.

2. Real git 2.39.5 (Apple Git-154):
   `git update-index --remove world.txt` on a PRESENT, unchanged file keeps it
   in the index (exit 0, world.txt stays). Reproduced in /tmp.

3. grit binary: identical behavior — present unchanged file under plain
   `--remove` stays in the index (exit 0).

4. Canonical source `git/builtin/update-index.c`:
   - `process_path` (L380): when `lstat` succeeds (file present) and the path
     is a regular file, it falls through to `add_one_path(ce, ...)` (L413),
     which UPDATES/keeps the entry. It does NOT call `remove_one_path`.
   - `remove_one_path` (the actual index drop) is only reached when `lstat`
     FAILS / the worktree file is MISSING (process_lstat_error -> remove_one_path),
     or via `--force-remove` (`remove_path` flag).
   - Therefore plain `--remove` only drops an entry whose worktree file is gone.

5. grit `grit/src/commands/update_index.rs` `PathMode::Remove` branch
   (~L844-865) implements exactly this: file missing on disk -> remove entry;
   file still present -> fall through to re-stat/update. CORRECT.

## Why this can't be "fixed"
- #5 asserts removal of a file that still EXISTS on disk and is unchanged —
  wrong against real git, so grit is correct.
- #11 expects 4 index entries (hello.txt, cached-file.txt, exec-file.sh,
  link-file); because world.txt correctly stays there are 5. Pure cascade.
- Current correct semantics were set by commit 5a17e9cc9 to make
  t4007/t4009/t4002 green; reverting to always-remove re-breaks those.
- This is a grit-custom test (no upstream `git/t/t8050` exists) codifying
  pre-5a17e9cc9 behavior. The only fixes are test-body edits (delete world.txt
  before `--remove`, or relax the line count) or `expect_failure`, both
  forbidden by harness rules (only `expect_failure -> expect_success` allowed).

## Lib tests
`cargo test -p grit-lib --lib`: only the 2 known
`ignore::gitignore_glob_tests` failures (unrelated to t8050) — treated as passing.

## Conclusion
Leaving ticket OPEN as a known-buggy-test record. No grit change is appropriate.
