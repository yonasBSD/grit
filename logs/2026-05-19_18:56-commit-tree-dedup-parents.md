# commit-tree duplicate parents

Branch: `fix-commit-tree-dedup-parents`

Focused on `t0000-basic` without touching the files owned by the existing review branches.

## Findings

- `git commit-tree <tree> -p P -p P` should write one `parent` header.
- Grit was preserving duplicate `-p` arguments, so the raw commit contained duplicate parent lines.

## Changes

- Deduplicate resolved parent object IDs while preserving first-seen parent order.

## Validation

- `cargo fmt`
- `cargo fmt --check`
- `cargo check -p grit-cli`
- `cargo build --release -p grit-cli`
- Manual duplicate-parent reproduction produced one `parent` header.
- `./scripts/run-tests.sh --output-csv /tmp/grit-t0000.csv --no-catalog t0000-basic.sh`
  moved `t0000-basic` from 91/92 to 92/92.
