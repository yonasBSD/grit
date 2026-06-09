# Grit — Git in Rust

Grit is a **from-scratch reimplementation of Git** in idiomatic Rust. The goal is to match Git's behavior closely enough that the upstream test suite (under `git/t/`) can be ported and run against this tool.

The Grit project is brought to you by the mad geniuses at [GitButler ⧓](https://gitbutler.com).

## Progress

![Harness test progress](docs/test-progress.svg)

## Approach

This implementation has been written nearly entirely by AI coding agents with the goal of entirely passing the C Git testing suite. For details on how we accomplished this, see our [blog post](https://blog.gitbutler.com/true-grit).

The implementation is entirely in Rust, with most of the generic logic in the [grit-lib](https://crates.io/crates/grit-lib) library crate, and the Git-compatible CLI in the [grit-cli](https://crates.io/crates/grit-cli) crate which uses the library to provide a UI that passes the Git tests.

## Usability

While the `grit` command emulates `git` functionality enough to successfully run over 42k of it's tests, it has been nearly entirely written by agents and has not been used for realsies. It's probably currently unusably slow or completely broken in ways that are not exercised in the test suite.

Our current goal is to get all the tests to pass and then refactor to real usability (speed, API surface, etc) while being able to successfully test for regression easily. Try it out and either send a fix or report an issue for anything you find or ways you want to use it that it doesn't successfully do.

## Installation

To install the `grit` CLI via Bash, you can run our install script:

```sh
$ curl -fsSL https://grit-scm.com/install | sh
```

There are builds for Mac and Linux, (aarch64 and x86_64 for both). Windows is on the list, but there's some work to do there.

## Updating

To update your version of Grit CLI, you can run `grit update` and it will re-run the install script.

## Rust Crates

| Crate                                           | Description                                                                                     |
| ----------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| [`grit-cli`](https://crates.io/crates/grit-cli) | The `grit` binary — a drop-in CLI reimplementation of `git` with 140+ commands                  |
| [`grit-lib`](https://crates.io/crates/grit-lib) | Core library: object model, diff engine, index, refs, revision walking, merge, config, and more |
| grit-examples                                   | Runnable examples of simple lib usage (add, cat-file, write-tree, hash-object, etc              |

## License

All Rust code and crates are MIT.
