# t7423-submodule-symlinks

## 2026-06-02 21:30

- Claimed `t7423-submodule-symlinks.sh` after `t7412-submodule-absorbgitdirs.sh` reached 12/12.
- Starting baseline from `data/test-files.csv`: 4/6 passing, 2 failing.
- Direct baseline showed failures in:
  - `git submodule update`, which reattached an existing `.git/modules/a/sm` repository through
    `a -> b` before validating the submodule path.
  - `git checkout -f --recurse-submodules initial`, which removed the symlinked dropped gitlink
    path instead of failing before any migration/removal.
- Moved `validate_submodule_path` earlier in `submodule update` and added recursive-checkout
  validation before dropped gitlink removal.
- `cargo build --release -p grit-cli` completed with the existing warning backlog.
- Direct `sh t7423-submodule-symlinks.sh -v` passed 6/6.
- Harness `./scripts/run-tests.sh t7423-submodule-symlinks.sh` passed 6/6 and refreshed
  `data/test-files.csv` plus dashboards.
- Validation completed: `cargo fmt`, `cargo check -p grit-cli`, `cargo test -p grit-lib --lib`
  (238/238), and `cargo clippy --fix --allow-dirty`; clippy's unrelated edits in
  `grit-lib/src/config.rs` and `grit-lib/src/filter_process.rs` were reverted.
