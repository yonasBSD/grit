# t5334-incremental-multi-pack-index — MOP-UP ROUND 1

## Starting state
- No ticket existed for t5334 (`ti list --tag t5 | grep 5334` empty). Nothing to claim.
- TOML: 15/16 passing, 1 failing.
- Re-ran fresh: still 15/16. Failing subtest #3 "convert incremental to non-incremental".

## Diagnosis
Test #3 sequence:
```
test_commit squash
git repack -d
git multi-pack-index write          # non-incremental
test_path_is_file $packdir/multi-pack-index    # passed
test_dir_is_empty $midxdir                      # FAILED
```
`test_dir_is_empty` requires the directory to EXIST (`test_path_is_dir`) and be empty.

Grit's non-incremental write path in `grit-lib/src/midx.rs` did
`fs::remove_dir_all(&midx_d)`, deleting the whole `multi-pack-index.d/`
directory. The old comment claimed "Git leaves no `multi-pack-index.d/`
directory behind" — that is wrong.

Ground truth in `git/midx-write.c` `clear_midx_files` →
`clear_incremental_midx_files_ext` (`git/midx.c:805`): git iterates
`multi-pack-index.d/` via `for_each_file_in_pack_subdir`
(`git/packfile.c:942`) and `unlink`s the individual
`multi-pack-index-<hash>.{midx,bitmap,rev}` files, then unlinks the chain
file. It never `rmdir`s the directory, so a non-incremental write leaves an
EMPTY `multi-pack-index.d/` behind.

## Fix (grit-lib/src/midx.rs)
- Added `clear_incremental_midx_files(pack_dir)`: unlinks the chain file and
  removes `multi-pack-index-*.{midx,bitmap,rev}` files inside
  `multi-pack-index.d/`, leaving the directory in place.
- Replaced the `remove_dir_all(&midx_d)` + chain-unlink block in the
  non-incremental write branch with a call to the new helper.
- Tightened the byte-identical short-circuit (mtime-retention) check to key off
  `chain_file_path(...).exists()` instead of `midx_d_dir(...).exists()`, so an
  empty leftover `multi-pack-index.d/` from a prior conversion does not defeat
  the t5319 retention optimization.

## Result
- t5334: 16/16 (was 15/16). FULLY PASSING.
- Regression checks (shared code is the midx write path):
  - t5319-multi-pack-index: 89/98 (unchanged).
  - t5310-pack-bitmaps: 200/236 (unchanged).
  - t5326 is in_scope=skip.
- `cargo test -p grit-lib --lib`: 269 pass; 2 pre-existing failures in
  `ignore::gitignore_glob_tests` (gitignore glob matching, unrelated to midx;
  `ignore.rs` is unmodified in the working tree). Not introduced by this change.
- rustfmt + clippy: no new warnings on midx.rs / the new function.
