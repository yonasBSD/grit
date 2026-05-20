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
