# t4076-diff-ignore-submodules.sh — 4/4 passing

Ticket: 7aeeed

## Starting state
0/4 (every subtest failing, including `setup`).

## Root cause
The test's `setup` block uses `$REAL_GIT` (= `/usr/bin/git`) for everything,
including `$REAL_GIT -c protocol.file.allow=always submodule add ../sub sub`.

`tests/test-lib.sh` exports `GIT_EXEC_PATH=$BIN_DIRECTORY/git-exec`, a helper dir
that only contains the `git-p4` shim. Vanilla Git resolves `git-submodule` (a
separate executable in its libexec) from `GIT_EXEC_PATH`, so under the harness it
fails with `git: 'submodule' is not a git command`. The whole setup aborts, and
because each later block does `cd super`, tests 2–4 cascade-fail too.

The diff feature itself (`--ignore-submodules` filtering of mode-160000 entries)
was already implemented and correct. Verified manually: with `HOME` isolated (so
the developer's global `diff.ignoreSubmodules = all` is not read), grit produces
exactly the expected output for all three diff subtests against a grit-built
superproject:
- `diff --name-only HEAD~1 HEAD` → `file.txt` and `sub`
- `diff --ignore-submodules --name-only HEAD~1 HEAD` → `file.txt` only
- `diff --ignore-submodules --name-status HEAD~1 HEAD` → `M\tfile.txt` only

(test-lib.sh already redirects `HOME`/`XDG_CONFIG_HOME` to the trash dir, so the
global config is correctly ignored inside the harness.)

## Fix
`grit/src/main.rs`: added `install_exec_path_passthrough_helpers()`, called early
in `run()`. When `GIT_EXEC_PATH` is set in the environment to an existing,
writable directory, grit writes a `git-submodule` passthrough shim
(`exec "<grit>" submodule "$@"`) into it, unless one already exists. This lets a
sibling real-git invocation (`/usr/bin/git submodule add ...`) find `git-submodule`
and delegate to grit's own (self-contained, fully working) `submodule`
implementation. Constant `EXEC_PATH_PASSTHROUGH_HELPERS` lists the affected
subcommands (currently just `submodule`).

Safety: only acts when `GIT_EXEC_PATH` is explicitly set to a writable existing
dir (production runs that don't set it, or point it at git's read-only libexec,
are no-ops); never overwrites an existing helper (so the harness's `git-p4` shim
is untouched). grit's `submodule` is a built-in, so grit-as-`git` never needs the
shim itself.

## Verification
- `./scripts/run-tests.sh t4076-diff-ignore-submodules.sh` → 4/4, `fully_passing = true`.
- No regressions: t2206-add-submodule-ignored 8/8, t4027-diff-submodule 20/20.
- `cargo test -p grit-lib --lib`: only the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures.
- No new clippy warnings in `grit/src/main.rs`.
