# t1090-sparse-checkout-scope — 9c3604

## Result
7/7 passing (was 6/7).

## Failing subtest
`not ok 7 - in partial clone, sparse checkout only fetches needed blobs`

## Root cause (NOT what the ticket guessed)
The ticket hypothesized a fetch `--filter=blob:none` / promisor / `rev-list --missing=print`
gap. That machinery already works: a manual repro of the fetch + checkout + rev-list flow
produced the correct `?<oid>` output for `server/b` and `server/c/c`.

The real failure was earlier in the `&&` chain. The test does:

    git clone --template= "file://$(pwd)/server" client &&
    ...
    mkdir client/.git/info &&

Upstream `git clone --template=` (explicit empty template) treats the template as an empty
directory, so `hooks/`, `info/`, `description`, and `branches/` are NOT created (they are
template content — git/setup.c only writes them via copy_templates). grit's clone instead
created `.git/info/`, so the subsequent `mkdir client/.git/info` failed with
`File exists`, breaking the `&&` chain and failing the whole subtest (verbose run showed
`mkdir: client/.git/info: File exists`).

## Why grit created info/
`grit/src/commands/clone.rs::effective_template_dir` collapsed `--template=` (empty string)
to `None`. The `skip_hooks_info` guard in `init_repository` /
`init_repository_separate_git_dir` is
`!bare && template_dir.is_some_and(|p| p.as_os_str().is_empty())`, which can never fire when
`template_dir` is `None` — so hooks/ and info/ were always created.

## Fix
`effective_template_dir` now returns `Some(PathBuf::new())` (empty path) for explicit
`--template=` instead of `None`. The empty path activates the existing `skip_hooks_info`
guard; an empty path is never `is_dir()`, so no template copy is attempted. Bare clone path
(`init_bare_clone_minimal`, gated on `args.bare && --template= empty`) is unchanged.

## Verification
- t1090: 7/7.
- Regression check: t5601-clone 112/115 and t5516-fetch-push 123/124 — identical to
  committed baseline (no regression). Only the explicit-empty-template case changes behavior.
- grit-lib --lib: only the 2 known pre-existing ignore::gitignore_glob_tests failures.

## Files
- grit/src/commands/clone.rs
