# 2026-05-20 partial clone promisor

Task: Phase 2.1, promisor ODB and packs.

- Moved sparse checkout from Phase 2 to Phase 5, after hooks, and moved partial clone/promisor work into Phase 2.
- Claimed the promisor marker/config/pack sidecar task in `plan.md`.
- Baseline `./scripts/run-tests.sh t0410-partial-clone.sh`: 19/38 before this branch work.
- Fixed optional ref lookups/listing so file-prefix paths that return `ENOTDIR` are treated like missing candidates. This unblocked `branch -f my_branch my_commit` in `t0410`.
- Taught `fsck` to ignore refs-fsck `missingObject` diagnostics when the missing ref target is in the promisor object closure.
- Made local lazy fetches materialize fetched objects into a `.promisor` pack sidecar so `cat-file` lazy fetch matches Git's pack shape.
- Result: `t0410-partial-clone` is now 27/38. First remaining failure is the unreliable promisor source checkout case after `clone --filter=blob:none --no-checkout`.

Validation:

- `cargo fmt`
- `cargo check` passed with existing warnings.
- `cargo clippy --fix --allow-dirty` completed after sandbox escalation; workspace still reports pre-existing clippy warnings.
- `cargo test -p grit-lib --lib` passed, 204/204.
- `cargo build --release -p grit-cli` passed.
- `./scripts/run-tests.sh t0410-partial-clone.sh` passed 27/38.

Continuation:

- Fixed `checkout HEAD` in a partial clone with an empty/mismatched index so it hydrates the HEAD
  tree instead of returning as a no-op. The unreliable promisor checkout case now fails with
  `could not fetch ... from promisor remote` when the source no longer has the promised blob.
- Added `uploadpack.allowrefinwant` to protocol-v2 capability advertisements so packet traces show
  `fetch=... ref-in-want`.
- Added `rev-list --exclude-promisor-objects` traversal support: commit walks stop at the expanded
  promisor closure, and `--objects` output filters promised tree/blob IDs.
- Added `rev-list --objects-edge-aggressive` parsing and `--ignore-missing` handling for missing
  command-line objects.
- Result: `t0410-partial-clone` is now 35/38. First remaining harness failure is the next
  post-`--ignore-missing` partial-clone case after test 24.
- Wired `pack-objects --exclude-promisor-objects` into the full-repack rev-list enumeration so
  `repack -a/-A/-l` does not traverse through promised commits into deleted ancestors.
- Result: `t0410-partial-clone` is now 36/38. First remaining harness failure is currently the
  next repack/gc partial-clone case after test 28.
