# rev-parse git-path formatting

Branch: `fix-rev-parse-git-path-relative`

Focused on `rev-parse --git-path` without touching the dashboard/script files from the repo-setup
PR or the diff command from the outside-repo PR.

## Findings

- `git rev-parse --git-path objects` printed an absolute path by default.
- Relative `core.hooksPath` values were treated as raw relative paths during `--git-path hooks/...`
  formatting, which produced paths relative to the process cwd machinery rather than the work tree.
- Fixing the default `--git-path objects` behavior advances specific `t1500` cases, but that file
  still has a separate shallow-clone failure.

## Changes

- Make default `--git-path` output relative to the current directory when no explicit
  `--path-format` is supplied.
- Resolve relative `core.hooksPath` against the work tree before formatting `--git-path hooks/...`.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `cargo check -p grit-cli`
- `cargo build --release -p grit-cli`
- `cargo clippy --fix --allow-dirty` completed with pre-existing warnings and no unrelated edits.
- `./scripts/run-tests.sh --output-csv /tmp/grit-t1350.csv --no-catalog t1350-config-hooks-path.sh`
  moved `t1350-config-hooks-path` from 3/4 to 4/4.
- `./scripts/run-tests.sh --output-csv /tmp/grit-t1500.csv --no-catalog t1500-rev-parse.sh`
  remains 80/81 due to the separate shallow-clone failure.
