# Libify: commit (date-to-git-timestamp engine)

## Target
`grit/src/commands/commit.rs` → new `grit_lib::commit`.

## What moved
Extracted the pure-domain date normalisation used to fill the author/committer
timestamp out of the CLI into a new library module `grit-lib/src/commit.rs`:

- `parse_date_to_git_timestamp(&str) -> Option<String>` — turns a user
  `--date` / `GIT_AUTHOR_DATE` / `GIT_COMMITTER_DATE` string into Git's stored
  `<epoch> <offset>` form (RFC 3339, `YYYY-MM-DD HH:MM:SS <tz>`, `@<epoch> <tz>`,
  and approxidate forms via `crate::git_date::parse::parse_date`).
- `format_git_timestamp(OffsetDateTime) -> String` — its helper (now `pub`
  because four surviving CLI sites in `commit.rs::resolve_author` still call it).

Both functions are pure: no clap, no `println!`/`eprintln!`, no `std::env`, no
tty/pager, no `crate::` references. The closure is just these two plus the
already-libified `parse_date`.

This is high-leverage because the function had **10 call sites across 8 command
files** (`commit`, `stash`, `notes`, `rebase`, `cherry_pick`, `checkout`,
`revert`, `tag`, `format_patch`), all now pointing at `grit_lib::commit::`.

## What was DEFERRED (and why)
The instructed primary target — the commit-object **assembly** core (tree from
index → parents → author/committer → message → new commit oid) — was **not**
extracted. In `commit.rs::run` that assembly is a single ~1450-line monolith
that interleaves `std::process::exit`, `eprintln!` UTF-8/encoding warnings,
editor launch, `prepare-commit-msg`/`pre-commit`/`commit-msg`/`post-commit`
hook dispatch, and HEAD/reflog updates, all mutating shared locals
(`message`, `raw_message`, `author_raw`, `committer_raw`). There is no function
boundary around the pure assembly, so lifting it out byte-exact would require a
large, risky refactor. Per the recipe ("DEFER if too entangled"), the assembly
core is left in the CLI; only the clean, widely-reused date engine was moved.

## Verification (byte-exact gate — all green, no regressions)
Baselines recorded from data/tests TOMLs, then re-run after the change:

| harness | before | after |
| --- | --- | --- |
| t7501-commit-basic-functionality | 77/77 | 77/77 |
| t7502-commit-porcelain | 82/82 | 82/82 |
| t7508-status | 126/126 | 126/126 |
| t7509-commit-authorship | 12/12 | 12/12 |

All four `fully_passing = true`, unchanged. `cargo build --release -p grit-cli`
clean (no warnings in touched files). `cargo test -p grit-lib --lib`: 289
passed, 2 failed (only the known `ignore::gitignore_glob` failures).
