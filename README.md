# Grit — Git in Rust

Grit is a **from-scratch reimplementation of Git** in idiomatic Rust. The goal is to match Git's behavior closely enough that the upstream test suite (under `git/t/`) can be ported and run against this tool.

This implementation is being written entirely by AI coding agents. The AGENT.md instructions and a snapshot of the Git source code were provided, and autonomous agents (first Cursor, then OpenClaw orchestrating Claude Code) implement commands, port tests, and validate against the upstream Git test suite.

## Crates

| Crate | Description |
|-------|-------------|
| [`grit-cli`](https://crates.io/crates/grit-cli) | The `grit` binary — a drop-in CLI reimplementation of `git` with 140+ commands |
| [`grit-lib`](https://crates.io/crates/grit-lib) | Core library: object model, diff engine, index, refs, revision walking, merge, config, and more |

Runnable **library examples** (repos, object database, index, `rev-list`, cherry-pick, and more) are documented in [`grit-lib/examples/`](grit-lib/examples/).

## Progress

![Harness test progress](docs/test-progress.svg)

See the **[project progress dashboard](https://schacon.github.io/grit/progress/)** (generated from the per-test status TOMLs in `data/tests/`, which `scripts/run-tests.sh` keeps current). The same numbers drive the static SVG above and the homepage progress card: run `python3 scripts/generate-dashboard-from-test-files.py` (or pass `--dashboard` to a harness run) to refresh [`docs/test-progress.svg`](docs/test-progress.svg) and `docs/index.html`.
