# t0450-txt-doc-vs-help

Ticket: 77d28b

## Summary

Verified `tests/t0450-txt-doc-vs-help.sh` is fully passing: **542/542**.

The ticket reported 7 failing subtests (188 shortlog, 315/317/320 whatchanged,
405/407/410 index-pack) because the `-h` short-help output of those three
commands did not match their documented SYNOPSIS.

On a fresh build of the current tree those three commands already emit proper
`-h` usage strings (exit code 129), so all 7 subtests now pass. The underlying
fixes were implemented by prior work in `grit/src/commands/`:

- `grit shortlog -h` =>
  ```
  usage: git shortlog [<options>] [<revision-range>] [[--] <path>...]
     or: git log --pretty=short | git shortlog [<options>]
  ```
- `grit index-pack -h` =>
  ```
  usage: git index-pack [-v] [-o <index-file>] [--[no-]rev-index] <pack-file>
     or: git index-pack --stdin [--fix-thin] [--keep] [-v] [-o <index-file>]
                 [--[no-]rev-index] [<pack-file>]
  ```
- `grit whatchanged -h` => `usage: git whatchanged <option>...`

These match the synopses in `git/Documentation/git-{shortlog,index-pack,whatchanged}.adoc`
after the test's tab->space / spacing normalization, and use no tabs with
consistent leading-space alignment.

## Result

- `./scripts/run-tests.sh t0450-txt-doc-vs-help.sh` => 542/542, fully passing.
- The on-disk TOML was stale (535/7); this run updated it to 542/0.
- `cargo test -p grit-lib --lib`: only the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures (not in scope for this ticket).

No Rust changes were required — only verification and the test-status TOML refresh.
