# t4205-log-pretty-formats — MOP-UP work log (ticket f1cb34)

## Context
File had previously been driven fully passing (closed ticket a9cb4f) but regressed to 119/125
(4 real `test_expect_success` failures). 16 & 125 remain `test_expect_failure` / TODO.

Fresh run 2026-06-07 failures:
- 18: NUL termination with --reflog --pretty=medium
- 19: NUL termination with --reflog --pretty=full
- 20: NUL termination with --reflog --pretty=fuller
- 22: NUL termination with --reflog --pretty=raw

(17 short, 21 email, 23 oneline still passed — they use the decoded `info.message`.)

## Diagnosis
The 4 failing tests compare `git show -s "$r" --pretty=$p` (expect) against
`git log -z --reflog --pretty=$p` (actual). The repo's initial commit is stored under
`i18n.commitEncoding=ISO8859-1` (message "initial. anfänglich"), so the commit object carries
an `encoding ISO-8859-1` header and the body byte for ä is 0xe4 (Latin-1).

`git show -s` re-encodes the body to the log output encoding (UTF-8 default) → `ä` = 0xc3 0xa4.
`git log -z --reflog --pretty=medium|full|fuller|raw` emitted the raw 0xe4 byte — no re-encode.

Root cause: commit `a35710718` ("fix: make t3900-i18n-commit fully pass") changed
`grit-lib/src/objects.rs` to populate `CommitData::raw_message` for **every** non-UTF-8 commit
encoding (previously only for genuinely undecodable bodies). `grit/src/commands/log.rs`'s
`write_commit_body_colored` preferred `info.raw_message` whenever present, bypassing the
log-output-encoding re-encode. Real git's `pretty_print_commit` calls `repo_logmsg_reencode`
for *all* builtin formats (medium/full/fuller/raw/short), converting from the commit's `encoding`
header to `get_log_output_encoding()` before formatting (git/pretty.c:2301-2303). So even
`--pretty=raw` re-encodes.

`objects.rs` is correct & needed for t3900 (which only exercises `git show`); the fix belongs in
log.rs.

## Fix (grit/src/commands/log.rs only)
1. Added `encoding: Option<String>` to the `CommitInfo` struct (the commit's `encoding` header),
   wired through all 5 construction sites from `CommitData::encoding`.
2. Added `commit_body_output_bytes(info, output_encoding)` — a faithful port of git's
   `logmsg_reencode` decision for the body:
   - output UTF-8 (None): emit decoded `info.message` when grit can faithfully decode the commit
     encoding (UTF-8 / ISO-8859-1 / any `is_known_encoding`); else keep verbatim `raw_message`
     (git keeps the raw buffer when `reencode_string` returns NULL).
   - output non-UTF-8 label: re-encode decoded `info.message` into that label via
     `encode_header_text`; fall back to `raw_message` if the target is unknown.
3. `write_commit_body` / `write_commit_body_colored` now take `output_encoding: Option<&str>` and
   run the indent/colorize loop over the resolved body bytes. All 4 builtin arms
   (medium/full/fuller/raw) pass `args.log_output_encoding.as_deref()`.

## Results
- t4205: 123/125 → `failing = 0`, `fully_passing = true` (16 & 125 are test_expect_failure TODO).
- No regressions: t3900 38/38, t4201 32/32, t4202 143/149 (unchanged 6 pre-existing fails),
  t8005 5/5, t4203-mailmap 74/74, t6006-rev-list-format 80/80.
- `cargo test -p grit-lib --lib`: 276 pass, only the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures (unrelated).
- No clippy warnings in commands/log.rs.
