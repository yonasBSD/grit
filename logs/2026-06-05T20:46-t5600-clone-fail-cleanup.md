# t5600-clone-fail-cleanup — failed-clone cleanup

Ticket: 3b6684. Started 8/14, finished **14/14** fully passing.

## Failures and root causes

The 6 failing subtests (8–13) all exercised cleanup after a failed clone. The upstream
contract (builtin/clone.c `remove_junk`): on failure, remove the git dir + work tree we
created. If a directory already existed (empty) before the clone, only its *contents* are
removed — the pre-existing directory is kept (`REMOVE_DIR_KEEP_TOPLEVEL`).

Grit had scattered, ad-hoc `fs::remove_dir_all(&target_path)` calls in a few error branches
of the local-clone path and **no cleanup at all** when checkout/HEAD failed, and never honored
keep-toplevel — so it deleted pre-existing dirs entirely (or left partial state behind).

### Fix 1 — `JunkGuard` (RAII cleanup), in `grit/src/commands/clone.rs` (`run`)
Mirrors upstream `atexit(remove_junk)` + `junk_mode = JUNK_LEAVE_ALL`-on-success:
- Armed right after the destination dirs are created.
- On Drop (any `?`/early return) it removes git dir then work tree, honoring keep-toplevel.
- `disarm()` called just before the final `Ok(())` so a successful clone keeps its output.
- Keep-toplevel flags: work tree keeps top level when the target pre-existed empty
  (`empty_dir_ok`); separate git dir keeps top level when it pre-existed (`real_dest_exists`).
- Removed the two now-redundant manual `fs::remove_dir_all(&target_path)` calls in the
  upload-pack error branches (the guard now handles them with correct keep-toplevel semantics).
Helpers added: `remove_dir_contents_keep_toplevel`, `remove_junk_path`, `JunkGuard`.

This fixed subtests 8, 9, 11, 12, 13.

### Fix 2 — detached-HEAD object verification for bare clones (subtest 10)
`clone --bare foo empty` with a corrupted source still *succeeded* because a bare clone has no
checkout to fail on. Upstream writes the (detached) HEAD via
`refs_update_ref(..., UPDATE_REFS_DIE_ON_ERR)` which verifies the target object exists; the
missing commit makes it die. Branch (symref) HEADs are written via an INITIAL transaction
upstream, which skips object verification, so only the detached case is checked.
Added `verify_detached_head_object_present(dest, oid)` (allows promisor objects) and call it in
the bare local-clone path before writing a raw-OID HEAD. On failure → `JunkGuard` cleanup.

## Notes / non-regressions
- t5601/t5602/t5603/t5604/t5606/t5607 unchanged vs committed baseline (no regressions).
- t5611 dropped 13→11 (tests 6,8: `clone -c remote.origin.fetch=<refspec>`), but that is
  another agent's in-flight `ls_remote.rs`/`remote.rs`/`main.rs` work in the shared checkout,
  NOT this change — my clone.rs edits never touch refspec/fetch handling (verified by diff).
- Pre-existing, unrelated: 2 `grit-lib` unit-test failures in `ignore::gitignore_glob_tests`
  (ignore.rs unchanged from HEAD); 4 clippy errors in hash_object.rs/pull.rs/rebase.rs (not
  my files). No clippy warnings in clone.rs.
