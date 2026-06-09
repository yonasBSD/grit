# t3404-rebase-interactive — untracked-overwrite obstruction round (ticket 9e2eff)

Baseline at session start (fresh harness run): 105/132 pass, 27 fail.

## Failing set at start
50, 51, 53, 54, 57, 79, 80, 85, 86, 87, 88, 89, 91, 94, 95, 96, 103, 104, 107,
117, 120, 123, 125, 126, 127, 128, 129
(86–89, 120 are cascade victims — pass in isolation; cascade origin is the real
79/80 failures leaving incomplete `--root` rebase state that pollutes later tests.)

## Fix landed: test 94 (rebase -i commits that overwrite untracked files (pick))

Root causes (3 distinct bugs in `grit/src/commands/rebase.rs`), all matching
Git's `sequencer.c:commit_staged_changes` semantics:

1. **Staged-changes gate placement.** Git refuses `rebase --continue` with
   `error: you have staged changes in your working tree` when the index has
   uncommitted changes and there is no in-progress `rebase-merge/message`
   (the pick was *blocked* by an untracked-file obstruction, never started).
   grit lacked this gate for the pick path. Added it in `do_continue`, and
   crucially placed it BEFORE the `stopped-sha`/`patch` files are consumed —
   the failed continue must not disarm the obstructed-pick re-apply path, or the
   subsequent clean continue (after `git reset --hard`) rewrites the commit
   instead of fast-forwarding it. The bare message is emitted (main.rs adds the
   `error: ` prefix) so the wording matches Git exactly.

2. **`done` double-record.** Git keeps an obstructed pick pending and
   re-consumes it into `rebase-merge/done` on retry, so the obstructed line
   appears in `done` twice. grit popped it from the todo and recorded it once.
   Now the verbatim obstructed todo line is saved to `obstructed-pick-line` at
   halt time and re-appended to `done` in the obstructed-pick re-apply path.

3. (Consequence of #1) the obstructed-pick re-apply now fast-forwards and reuses
   the original commit OIDs, so the final `test_cmp_rev HEAD D` holds.

## Fix landed: edit-continue correctness (helps 95/96 partially; no regression on 41–46)

- **Stale `COMMIT_EDITMSG`.** An `edit` continue reused `.git/COMMIT_EDITMSG` as
  the message source if it existed; a leftover from an unrelated earlier commit
  (here "P") corrupted the amended commit's message. Git amends via
  `git commit --amend -F rebase-merge/message` (the edited commit's own message =
  HEAD's message). Restricted `COMMIT_EDITMSG` to non-edit-continue (reword) flows.
- **Clean edit-continue must not re-commit.** Git's `commit_staged_changes`:
  when the worktree is clean and it is not a final fixup, an `edit` continue does
  NOT re-commit — it removes CHERRY_PICK_HEAD/MERGE_MSG and proceeds, leaving HEAD
  at the edited commit. grit always amended (fresh committer ⇒ new OID), so a
  later squash obstruction left HEAD on the rewritten commit. Added the clean-skip.

## Result
106/132 (was 105). Test 94 now passes; no regressions (41–46 edit tests, 65–78,
99–102, 105–106, 108–109 etc. still green).

## Remaining (notes)
- 95/96 (squash / no-ff overwrite): the squash obstruction halt currently writes
  a `rebase-merge/patch` (test wants it missing) and the squash re-apply does not
  fold the squashed commit's message (final last-line stays "F" not "I"). The
  obstructed-pick re-apply path (`needs_reapply`) only handles Pick|Fixup; Squash
  needs a re-merge + message-fold on continue. Left for a follow-up.
- 79/80 (`--root` untracked conflict): same obstruction family on the `--root`
  pick-onto-empty-tree path; cascade origin for 86–89/120.
- 103/104 (edit-todo missingCommitsCheck two-phase), 107 (exec-after-autosquash
  abbrev), 117 (post-commit hook in replay), 85 (commentChar=auto deprecation),
  50/51/53 (submodule), 54 (no-op fast-forward), 91 (collision abbrev),
  123/125-129 (--update-refs rebase-merges + edit-todo) — unchanged from prior.

## Coexistence note
`grit/src/commands/rebase.rs` is shared with a concurrent agent working on
`--update-refs` (the `rebase_update_refs_todo_lines` / merge-backend update-refs
todo work). Hunk-level staging is stale, so committing the file as a unit carries
their in-flight (compiling, partially test-passing) hunks into my commit. Work is
preserved in history; flagged on the ticket.
