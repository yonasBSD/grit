# t6132-pathspec-exclude — MOP-UP round 1

Ticket: 2a22a9
Date: 2026-06-07T03:54Z

## Starting state
Ticket scan said 23/31 (8 failing). Fresh re-run after other agents' cascading
fixes: 30/31. Single remaining failure: `not ok 28 - grep with all negative`.

## Diagnosis
Repro from `repo/sub`:
    grit -C sub grep -h needle -- ":!sub/"
Expected: `needle sub/file`. grit printed nothing (rc=1).

Debugged the resolved pathspec list inside grep: it was
`["sub/", ":!sub/"]` — the implicit positive `sub/` (cwd) plus the
user-supplied exclude `:!sub/`, which still matched `sub/file` and excluded it.

Root cause: `grep::pathspecs_relative_to_cwd` rebases plain paths against the
current prefix but returned anything starting with `:` unchanged. So a
cwd-relative magic exclude `:!sub/` was never rebased to `:!sub/sub/`. For
user-supplied pathspecs grep then calls `resolve_pathspec` with `prefix=None`
(to avoid double-prefixing the already-rebased plain paths), so the magic spec
never picked up the prefix anywhere. ls-files/add/clean/etc. pass because they
use the typed `Pathspec` that prefixes magic specs.

## Fix
`grit/src/commands/grep.rs` `pathspecs_relative_to_cwd`: route magic pathspecs
(those starting with `:`, non-absolute) through `resolve_pathspec(s, wt,
Some(prefix))`. That helper already turns `:!sub/` (prefix `sub`) into
`:!sub/sub/` and leaves `:(top)` / `:/` rooted specs intact. The later
`resolve_pathspec(..., None)` pass leaves magic specs untouched, so no double
prefixing.

## Results
- t6132-pathspec-exclude: 31/31, fully passing (was 30/31 fresh, 23/31 at scan).
- Regression checks: t7810-grep 263/263, t7811-grep-open 10/10 (unchanged).
- Unit tests: pass except the 2 known pre-existing ignore::gitignore_glob_tests
  failures (unrelated to this change).
- rustfmt + clippy clean on grep.rs.

## Files changed
- grit/src/commands/grep.rs (pathspecs_relative_to_cwd magic-spec rebasing)
- data/tests/t6/t6132-pathspec-exclude.toml (now fully_passing)
- logs/2026-06-07T03:54-t6132-pathspec-exclude.md
