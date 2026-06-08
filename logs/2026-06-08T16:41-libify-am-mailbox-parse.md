# Libify: am mailbox/patch parsing core

Target: `am`. Source `grit/src/commands/am.rs` (4,523 lines).
Destination: new `grit_lib::am`.

## What moved

`grit am` is a large, deeply stateful command entangled with the
`.git/rebase-apply` state machine, patch-to-worktree application, 3-way merge,
commit assembly, hooks, identity resolution, and clap. I extracted the cleanest
self-contained slice: the **mbox/stgit/hg patch parsing layer** that turns patch
text into structured `MboxPatch` values. These functions are pure — string in,
`MboxPatch` out — with no `Repository`, index, filesystem, `std::env`, or clap
dependency.

Moved to `grit-lib/src/am.rs`:

- `MboxPatch` struct (all fields now `pub`) and `QuotedCrAction` enum (`pub`).
- Public entry points: `parse_patches`, `parse_mbox_with_opts`,
  `parse_stgit_patch`, `parse_hg_patch`, `detect_patch_format`,
  `is_stgit_series`, `is_skippable_mail_folder_message`,
  `parse_quoted_cr_action`, `serialize_mbox_patch`, `deserialize_mbox_patch`.
- Private helpers: `unflow_format_flowed`, `split_lines_preserve_cr`,
  `unquote_mboxrd`, `base64_decode`, `decode_transfer_payload`,
  `split_message_body_and_diff`, `parse_format_patch_commit_oid_from_mbox_line`,
  `strip_patch_prefix`, `strip_patch_prefix_keep_non_patch`,
  `apply_scissors_to_message`, `is_scissors_line`, `parse_author_ident`,
  `parse_date_to_epoch`, `parse_rfc2822_date`, `datetime_to_epoch`.

The only lib dependency is `crate::commit_encoding::decode_rfc2047_mailbox_from_line`
(already in lib) and `crate::objects::ObjectId`. `anyhow::anyhow!` was converted
to `crate::error::Error::Message` with byte-identical text.

### Warnings as data (no stderr in the parse core)

Two stderr warnings `git am` prints while parsing — `"warning: quoted CRLF
detected"` and `"warning: Patch sent with format=flowed; space at the end of
lines might be lost."` — would violate the no-output rule in a library. Instead
of `eprintln!`, `parse_patches` / `parse_mbox_with_opts` / `decode_transfer_payload`
now push these strings into a caller-supplied `&mut Vec<String>`. The CLI emits
them verbatim right after each parse call, preserving exact ordering and text
(both happen up-front, before any `Applying:` line).

## What stayed in the CLI (deferred)

Everything stateful or CLI-coupled: the whole `.git/rebase-apply` state machine
(`do_am`, `do_am_stdin`, `apply_remaining`, `do_continue`, `do_skip`,
`do_abort`, `do_retry`, save/load options), patch application
(`apply_patch_to_worktree`, the apply-subset `parse_patch`/`Hunk`/`FilePatch`),
3-way merge, commit assembly (`create_am_commit`), hooks, clap `Args`, and the
two parse helpers that touch config/fs: `malformed_empty_patch` (uses
`ConfigSet` + CLI `crate::ident` identity resolution) and `parse_stgit_series`
(reads files; it calls the now-`pub` `am::parse_stgit_patch`). `parse_quoted_cr_action`
is `pub` in lib and called by the surviving config-aware `resolve_quoted_cr_action`.

## CLI wiring

`grit/src/commands/am.rs`: deleted the moved defs (net -1,108 lines); added
`use grit_lib::am::{…}` so bare-name call sites resolve; threaded the
`&mut Vec<String>` warnings sink through the two `parse_patches` call sites and
emit collected warnings with `eprintln!`.

## Verification (byte-exact)

- `cargo build --release -p grit-cli -j 4`: clean, no warnings in my files (the
  3 pre-existing warnings are in diff.rs / merge.rs / repack.rs).
- `cargo test -p grit-lib --lib`: 289 passed, only the 2 known
  `ignore::gitignore_glob` failures.
- Harness (recorded baseline → result, measured on the current tree):
  - t4150-am: 40/87 → **40/87**.
  - t4151-am-abort: 11/20 → **11/20**.
  - t4152-am-subjects: 10/13 → **10/13**.
  - t4153-am-resume-override-opts: 6/6 → **6/6** (fully_passing).
  - No `fully_passing` flipped true→false; every passed count == baseline.

  (Note: the per-file TOMLs are marked `in_scope = "skip"` with stale
  `passed_last = 0`; baselines above were measured by running the `.sh` files
  directly on the pre-change tree.)

## Files

- new `grit-lib/src/am.rs`
- `grit-lib/src/lib.rs` (+`pub mod am;`, alphabetical before `apply`)
- `grit/src/commands/am.rs` (delete moved defs + import-back + warnings threading)
