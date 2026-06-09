# Libify: tag listing/filtering core -> grit_lib::porcelain::tag

## Target
`grit/src/commands/tag.rs` (951 lines). The plan asked to extract the
annotated-tag object creation core (build + sign + write the tag object) and the
tag listing/filtering core.

## Why the creation core stayed in the CLI
On inspection the tag-object creation path is entangled with output rendering
that the CLI must own byte-for-byte, and — critically — with the CLI's error
*rendering*. `grit`'s `main.rs` treats a `grit_lib::error::Error::Message` in the
error chain specially: `verbatim_lib_error_message` prints it with **no** prefix,
whereas a CLI-side `anyhow::bail!` is rendered as `error: {msg}`. So converting
the creation path's `bail!`s (`"tag '{name}' already exists"`, `"no tag message
provided (use -m or -F)"`, the multi-line `"Tagger identity unknown..."`, the
sign-failure message, etc.) into `Error::Message` as the recipe requires would
flip those user-facing lines from `error: ...` to bare text. The cleanest,
provably byte-exact slice was therefore the **listing/filtering + message-cleanup
core**, which is infallible (predicates/comparators returning bool/Option/i64,
plus a pure `String` transform) and surfaces no errors through that renderer.
`create_annotated_tag` / `create_lightweight_tag` / `delete_tag` / `verify_tag`
(odb/ref writes interleaved with `eprintln!`, `print!`, and the special error
text) and `resolve_tagger` (whose `anyhow` message must render as `error:`) stay
in the CLI.

## What moved (new file `grit-lib/src/porcelain/tag.rs`)
Pure, presentation-free tag-listing/filtering core (no clap, no print, no exit,
no env, no `crate::`-CLI refs, infallible):

- `--contains` / `--no-contains` / `--points-at` predicates: `tag_contains`,
  `tag_points_at`, and the shared `peel_to_commit`.
- `--sort` comparators: `compare_version` / `version_segments` (numeric
  `version:refname`) and `creator_date` / `parse_epoch_from_ident`
  (`creatordate`/`taggerdate` epoch extraction).
- `-l` glob matcher `glob_matches` (+ private `glob_match_bytes`) and `-n<N>`
  annotation extraction `get_tag_annotation`.
- `-m`/`-F` message cleanup `strip_comments`.

All exported `pub` (except the recursive `glob_match_bytes` helper). These were
infallible / pure transforms, so no `bail!`/`context`/`anyhow!` translation was
needed.

## Deduplicated
The CLI's local `format_git_timestamp` was a byte-identical copy of the already
public `grit_lib::commit::format_git_timestamp`. Deleted the local copy;
`resolve_tagger` now calls the lib version.

## CLI changes
- Added `use grit_lib::porcelain::tag::{compare_version, creator_date,
  get_tag_annotation, strip_comments, tag_contains, tag_points_at}`.
- Kept `crate::commands::tag::glob_matches` resolving for its external caller
  `commands/describe.rs` (4 call sites) via `pub use
  grit_lib::porcelain::tag::glob_matches;` — path stability, no edit to
  describe.rs.
- Deleted the moved defs (`glob_matches`/`glob_match_bytes`, `tag_contains`,
  `tag_points_at`, `peel_to_commit`, `creator_date`, `parse_epoch_from_ident`,
  `compare_version`, `version_segments`, `get_tag_annotation`, `strip_comments`)
  and the duplicate `format_git_timestamp`.
- Net: grit/src/commands/tag.rs 7 insertions, 260 deletions.

## What stayed in the CLI (correctly)
- `run` (clap dispatch + sort-key validation that `eprintln!`s and
  `std::process::exit(129)`), `list_tags` (writeln output + calls the moved
  predicates/comparators), `sort_tags` (its unknown-key branch `eprintln!`s and
  `std::process::exit(129)`), `format_tag_line` / `expand_tag_atom`
  (their `bail!`s render as `error:`), `build_tag_message` (stdin/file I/O whose
  io errors must render as the CLI's `error: {anyhow}` text), `resolve_tagger`
  (env-read identity whose `anyhow!` must render as `error:`),
  `create_annotated_tag`/`create_lightweight_tag`/`delete_tag`/`verify_tag`.

## Verification (byte-exact gate)
Both tag harness files are `in_scope = "skip"` in their TOMLs, so the normal
harness never runs them; I temporarily flipped them to `"yes"`, measured the
ORIGINAL tree and the changed tree, then restored the TOMLs byte-for-byte (no
`git diff` under `data/tests/` afterward).

- t7004-tag: 170/231 before == 170/231 after (its non-fully-passing baseline).
- t7030-verify-tag: 16/16 before == 16/16 after, still fully_passing.
- t6300-for-each-ref: 429/429 before == 429/429 after, still fully_passing.

`cargo build --release -p grit-cli -j 4`: clean; the only remaining warnings are
pre-existing and in files I did not touch (`base_layers_declared`, `ext_total`,
two `does not need to be mutable`). `cargo test -p grit-lib --lib`: 289 passed,
only the 2 known `ignore::gitignore_glob` failures.

## Notes for the next agent
A pre-existing rustfmt-only diff in `grit-lib/src/porcelain/status.rs` is sitting
in the working tree (ambient noise, not mine, same one prior logs mention) and
was deliberately excluded from this commit.
