# Phase 2 HTTP promisor lazy fetch

Task: finish the remaining `t0410-partial-clone.sh` failure.

Changes made:

- Reproduced the lone failure in test 38, `fetching of missing objects from an HTTP server`.
- Found HTTP lazy fetch successfully fetched the missing object but unpacked it as a normal pack, leaving no `pack-*.promisor` sidecar.
- Changed HTTP promisor lazy fetch to keep fetched packs as promisor packs even when no object filter is sent.

Validation:

- `cargo build --release -p grit-cli` passes with existing warnings.
- Focused `t0410-partial-clone.sh --run=38` passes when the local HTTP server is allowed to bind.
- `./scripts/run-tests.sh t0410-partial-clone.sh` reports 38/38 when the local HTTP server is allowed to bind.
