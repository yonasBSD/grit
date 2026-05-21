# Phase 2 filter clone/fetch

Started from `t5616-partial-clone.sh` at 15/47 via `./scripts/run-tests.sh`.

Initial direct verbose run showed the first failures are filter accounting rather than a transport
crash:

- `rev-list --quiet --objects --missing=print` printed ordinary reachable objects as well as
  missing objects, inflating the expected missing-blob counts.
- `fetch --no-filter` is not accepted yet.
- repeated `clone --filter` flags are rejected instead of being combined.
- later failures cover upload-pack filter policy, sparse filters, and submodule filtering.

Implemented this pass:

- `rev-list --quiet --objects --missing=print` now stays quiet for present objects and prints only
  missing objects.
- `fetch` accepts `--no-filter`, inherits `remote.<name>.partialclonefilter` for promisor remotes,
  and accepts `--keep` for tests that need pack retention.
- `clone` accepts repeated `--filter` flags and normalizes them to `combine:<filters>`.
- `clone --filter=tree:0` now records partial-clone config and a missing tree/blob marker for
  local/file clone paths.
- `fetch` blob:none accounting now excludes blobs already reachable from the matching local branch,
  so inherited filtered fetch reports only newly omitted blobs.

Verification:

- `cargo build --release -p grit-cli`: pass with existing warnings.
- Focused `t5616-partial-clone.sh --run=1-8`: pass.
- Focused `t5616-partial-clone.sh --run=34`: pass.
- Focused `t5616-partial-clone.sh --run=35`: pass.
- `./scripts/run-tests.sh t5616-partial-clone.sh`: 21/47, up from 15/47 at the start of this log
  and 18/47 at the start of today’s continuation.
- `./scripts/run-tests.sh t0410-partial-clone.sh`: 37/38, unchanged from the latest baseline.
