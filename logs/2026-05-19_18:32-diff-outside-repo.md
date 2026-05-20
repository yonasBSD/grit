# diff outside repo

Branch: `fix-apply-outside-repo`

Focused on `t1517-outside-repo.sh` without touching dashboard, plan, or progress files from the
parallel repo-setup PR.

## Findings

- `git diff` saw index/worktree changes but emitted no unified patch by default.
- That left `sample.patch` empty in `t1517`, which made the outside-repo `git apply` case fail.
- Outside a repository, `git diff one two` classified both filesystem paths as revisions and
  errored instead of falling back to no-index diff mode.

## Changes

- Emit unified patch output for plain `git diff` when no other output format is selected.
- Keep format-only behavior for `--stat`, `--raw`, `--name-only`, etc. unless `-p` is explicitly
  requested.
- Treat two existing filesystem paths outside a repository as implicit `--no-index`.

## Validation

- `cargo fmt --check`
- `cargo check -p grit-cli`
- `cargo build --release -p grit-cli`
- `./scripts/run-tests.sh --output-csv /tmp/grit-t1517.csv --no-catalog t1517-outside-repo.sh`
  moved `t1517-outside-repo` from 185/191 to 187/191.
