# t7424-submodule-mixed-ref-formats

## 2026-06-02 21:52

- Claimed skipped `t7424-submodule-mixed-ref-formats.sh` for audit after all in-scope
  submodule-plan rows reached passing state.
- Starting baseline from `data/test-files.csv`: `in_scope=skip`, 3/14 passing, 11 failing.

## 2026-06-02 22:04

- Direct audit first exposed that the local harness lacked upstream's default
  `GIT_DEFAULT_REF_FORMAT=files`; added that default in `scripts/run-tests.sh` without modifying
  `tests/test-lib.sh`.
- Fixed `submodule add`'s existing-repo check to resolve symbolic and reftable-backed `HEAD`
  through `grit_lib::diff::read_submodule_head_oid`.
- Added `--ref-format` parsing and forwarding for `submodule add` and
  `submodule update --init`.
- Fixed recursive clone with `--ref-format=reftable` by propagating the destination ref storage
  backend to submodule clone jobs and resolving the destination `HEAD` through the shared ref
  resolver when collecting gitlink paths.
- Added `clone --no-recurse-submodules` parsing so explicit non-recursive clones remain accepted.
- Verified `cd tests && GIT_DEFAULT_REF_FORMAT=files sh t7424-submodule-mixed-ref-formats.sh -i -v`
  passes 7/7.
- Restored `t7424-submodule-mixed-ref-formats` to `in_scope=yes` and refreshed harness data with
  `./scripts/run-tests.sh t7424-submodule-mixed-ref-formats.sh`, passing 7/7.
