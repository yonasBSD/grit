# t3437-rebase-fixup-options.sh — regression fix (ticket 5a0724)

## Summary
Restored t3437 to 13/13 (was 11/13, a regression from the de1f52 13/13 pass).

Two failing subtests:
- 8: sequence of fixup, fixup -C & squash --signoff works
- 12: fixup -C works upon --autosquash with amend!

Both failed on `test_cmp "$TEST_DIRECTORY/t3437/expected-squash-message" actual-squash-message`
(the FAKE_MESSAGE_COPY of the combined squash editor buffer).

## Root cause
Commit `c4f32f568` (`fix: plain fixup keeps pick-target message in squash chain`, for t3404 #35)
changed the `count == 0` branch of `update_squash_message_file` so a plain `fixup` keeps the 1st
commit message uncommented (`# This is the 1st commit message:`) instead of skipping it. That fix is
correct for the standalone plain-fixup case.

But t3437 #8/#12 use a chain like `pick, fixup, fixup -C, fixup -C, squash, fixup -C`. When a later
`fixup -C` (a message-replacing fixup, `is_fixup_flag && !seen_squash`) runs, Git retroactively
comments out EVERY section accumulated so far — including the now-uncommented plain-fixup section —
via `update_squash_message_for_fixup` (sequencer.c). Grit's old code only marked a single tracked
section (`message-fixup-active-section`), which plain fixup never wrote, so the earlier sections were
left uncommented and the buffer diverged from `expected-squash-message`.

## Fix
Ported Git's `update_squash_message_for_fixup` faithfully into `grit/src/commands/rebase.rs`:
- New `update_squash_message_for_fixup(buf)` walks the whole squash buffer, flipping every
  "This is the (Nth/1st) commit message:" header to "...will be skipped:" and commenting out its
  body (empty body lines become `#`, matching `strbuf_add_commented_lines`), while leaving
  already-skipped sections untouched. Blank separators between sections are preserved via the same
  `off` blank-line bookkeeping Git uses.
- New `copy_section(out, text, comment_mode)` helper mirrors Git's `copy_lines`/`add_commented_lines`
  switch (verbatim vs commented copy; never double-comments `#` lines).
- Rewrote the `count > 0` branch to match Git's order in `update_squash_messages`:
  splice the combination-count header, then (when `is_fixup_flag && !seen_squash`) run
  `update_squash_message_for_fixup`, then write FIXUP_MSG, then append the new section.
- Removed the now-obsolete single-section tracking: `mark_squash_section_skipped`,
  `comment_block`, `comment_block_preserving_comments`, and the `message-fixup-active-section`
  state file writes. (`clear_squash_ctx` still removes the file for backward cleanup.)

The `count == 0` branch (c4f32f568's plain-fixup behavior) is unchanged, so t3404 #35 still passes.

## Verification (all via run-tests harness, baseline binary vs new binary, isolated data dirs)
- t3437: 12/13 (baseline direct) -> 13/13 (new, harness). Stable across 3 re-runs.
- t3404-rebase-interactive: 106/132 == 106/132 (no change)
- t3415-rebase-autosquash: 28/28 == 28/28
- t3434-rebase-i18n: 6/6, t3440-rebase-trailer: 10/10, t3428-rebase-signoff: 7/7
- t3436-rebase-more-options: 16/19 == 16/19, t3421-rebase-topology-linear: 63/64 == 63/64
- t3430-rebase-merges: 34/34, t3412-rebase-root: 25/25, t3424-rebase-empty: 19/20 (all == baseline)
- cargo test -p grit-lib --lib: pass (modulo the 2 known ignore::gitignore_glob_tests failures)
- clippy: 0 warnings reference my new code.

No regressions.
