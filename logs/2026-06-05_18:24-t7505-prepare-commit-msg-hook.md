# t7505 prepare-commit-msg hook

Ticket: 5911b7

## Reproduction

- Built `grit-cli` release binary because `target/release/grit` was missing.
- Ran `./scripts/run-tests.sh t7505-prepare-commit-msg-hook.sh`: 22/23, failing subtest 16 (`with hook (rebase -i)`).
- Ran the test directly with `-v -i` to preserve `tests/trash.t7505-prepare-commit-msg-hook`.

## Finding

After an interactive `edit` stop at `rebase-10`, Grit wrote the remaining todo but kept
`rebase-merge/msgnum` at the global completed command count. `rebase --continue` then started
past the end of the shortened todo and called `finish_rebase` before replaying the remaining
`squash`, `squash`, and `edit` commands.

## Fix

When an interactive `edit` command stops successfully, reset the shortened todo state to
`msgnum=1` and `end=<remaining todo length>`, matching the normal post-step todo rewrite path.

Follow-up: after the next edit stop (`rebase-13`), `rebase --continue` with staged changes was
treated as a completed manual commit because HEAD had changed when the edit commit was applied.
Changed that path so a clean HEAD still means "user already committed", but staged changes amend
the edit commit through the same `message` prepare-commit-msg/editor flow Git uses.

Second follow-up: the rebase completed but the reflog subject fixture still differed. Fixed the
remaining sequencer details covered by this test:

- final squash/fixup editor hooks use `message` as the prepare source, not `squash`;
- reword records the initial no-editor pick before the edited replacement;
- staged edit-continue replacements write their own HEAD reflog entry before finish;
- a manual commit made while resolving an interactive rebase conflict is not committed again;
- rebase conflict continues use `message` as the prepare source even when `MERGE_MSG` exists.

Final narrow mismatch was the non-fast-forward reword path: it created the no-editor pick commit
but did not append a HEAD reflog entry for it, leaving the expected `message (no editor)
[reword rebase-6]` subject out of the 18-entry reflog window.
