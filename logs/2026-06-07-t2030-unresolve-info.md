# t2030-unresolve-info.sh — unmerge after committing

Ticket: 86b1a7

## Symptom
Subtest 7 "unmerge can be done even after committing" failed (13/14). After
`git commit` recorded the resolved merge, `git update-index --unresolve fi/le`
printed:

    error: corrupted cache-tree has entries not present in index

and exited non-zero, so the subsequent `git ls-files -u` did not show the
expected 3 unmerged stages.

## Root cause
`Index::unmerge_path_from_resolve_undo` (grit-lib/src/index.rs) reinstalls the
unmerged stages 1/2/3 from the REUC record via
`install_unmerged_from_resolve_undo`, which removes the resolved stage-0 entry
and adds the conflicted stages — but it never invalidated the cache-tree.

After committing, the index carries a valid TREE extension whose stage-0 entry
for the path no longer matches the (now unmerged) index. The next write-tree /
commit path runs `verify_cache_tree`, which reports the "corrupted cache-tree"
error.

Git's `unmerge_index_entry` (git/resolve-undo.c) performs the swap with
`remove_index_entry_at` + `add_index_entry`, both of which internally call
`cache_tree_invalidate_path`. The pre-commit subtests (5, 6) passed only because
no valid cache-tree was present at that point.

## Fix
In `unmerge_path_from_resolve_undo`, after installing the unmerged stages, call
`invalidate_untracked_cache_for_path` and `invalidate_cache_tree_for_path` for
the path — mirroring the existing `Index::remove` path.

## Result
t2030-unresolve-info: 14/14 passing. `cargo test -p grit-lib --lib` clean modulo
the 2 known pre-existing ignore::gitignore_glob_tests failures.
