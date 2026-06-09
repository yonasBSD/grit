# t7401-submodule-summary

Ticket: 4b0b6f

## Starting state
24/25 passing. Failing subtest:
- 25 - should not fail in an empty repo

## Root cause
In a fresh, empty repo (no commits), `git submodule summary` with no
arguments resolves the base tree from `HEAD` via `resolve_revision`. The
DWIM ref resolver `resolve_ref_dwim_for_rev_parse` in
`grit-lib/src/rev_parse.rs` found that the symref `HEAD` pointed at an
unborn branch (`refs/heads/master`, which does not exist yet) and printed
`warning: ignoring dangling symref HEAD` to stderr. The test redirects
stderr into `output` and asserts `test_must_be_empty output`, so the
warning caused the failure.

Upstream `expand_ref` (git/refs.c:829) only emits this warning when the
dangling symref is NOT literally `HEAD`:

```c
} else if ((flag & REF_ISSYMREF) && strcmp(fullref.buf, "HEAD")) {
    warning(_("ignoring dangling symref %s"), fullref.buf);
}
```

i.e. an unborn `HEAD` is silently ignored; only other dangling symrefs
get the warning.

## Fix
`grit-lib/src/rev_parse.rs` — in `resolve_ref_dwim_for_rev_parse`, guard
the `eprintln!("warning: ignoring dangling symref ...")` with
`if candidate != "HEAD"`, matching upstream. The `continue` (skip the
candidate) is preserved either way, so resolution behavior is unchanged;
only the spurious warning for an unborn HEAD is suppressed.

## Result
25/25 passing. `data/tests/t7/t7401-submodule-summary.toml` updated to
fully_passing = true. No new clippy warnings in the edited file.
grit-lib unit tests pass modulo the 2 known pre-existing
`ignore::gitignore_glob_tests` failures (unrelated).
