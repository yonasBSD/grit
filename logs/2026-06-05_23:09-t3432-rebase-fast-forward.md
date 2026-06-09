# t3432-rebase-fast-forward

Ticket: 97dfee
Claimed: 2026-06-05 23:09 Europe/Berlin

Goal: make `tests/t3432-rebase-fast-forward.sh` fully pass by matching Git's fast-forward/noop behavior across rebase backends and force-replay options.

Initial ticket state: 201/225 passing; remaining failures are clustered in the final matrix for `our and their changes` with `--keep-base`, `--fork-point`, `--onto main... main`, and `--no-ff`.

Update 2026-06-05 23:24: Persisted upstream-tip in rebase state so forced one-commit no-op preservation only applies when fork-point did not shorten the replay range.

Update 2026-06-05 23:32: Release build passed with known warnings. t3431 guard passed 26/26. t3432 improved to 219/225; remaining failures are in the final forced fork-point keep-base variants.

Update 2026-06-05 23:36: Direct verbose run completed through test 225. The file now has no unexpected failures: 219 passing assertions plus the pre-existing 6 TODO known breakages. Closing the ticket.

Validation 2026-06-05 23:45:
- cargo build --release -p grit-cli: pass with known warnings.
- ./scripts/run-tests.sh t3432-rebase-fast-forward.sh --timeout 240: fully_passing=true, 219 passing assertions with 6 existing TODO known breakages.
- ./scripts/run-tests.sh t3431-rebase-fork-point.sh --timeout 180 --data-dir /tmp/grit-t3431-check: 26/26.
- Direct verbose t3432 run completed through 225 with no unexpected failures.
- cargo fmt: pass.
- cargo check: pass with known warnings.
- cargo clippy --fix --allow-dirty: exits 0 with pre-existing warning backlog; restored unrelated promisor_remote.rs autofix.
- cargo test -p grit-lib --lib: baseline 252 passed, 2 unrelated ignore glob failures.
