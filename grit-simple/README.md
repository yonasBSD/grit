# grit-simple

`grit-simple` provides `gi`, a small opinionated command line interface backed by [`grit-lib`](https://crates.io/crates/grit-lib).

It is not intended to be a drop-in replacement for Git. For Git-compatible command behavior, use the `grit` binary from the `grit-cli` crate. `gi` is a simpler interface for workflows built on Grit's Rust implementation.

## Install

```sh
cargo install grit-simple
```

This installs the `gi` executable.

## Commands

### `gi shortlog`

Show the current branch, the target branch, and commits that are reachable from `HEAD` but not from the target branch.

```sh
gi shortlog
# alias:
gi sl
```

Target branch lookup uses the first available value from:

1. `target.branch` in Git config
2. `origin/master`
3. `origin/main`
4. `master`
5. `main`

Example:

```text
On feature/example
Ahead of origin/main by 2 commits
abc1234 Add example
fed9876 Refine output
```
