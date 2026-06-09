# t3507-cherry-pick-conflict.sh

Ticket: 432dcd (test, t3)
Agent: schacon+opus48@gmail.com

## Starting state
Fresh run after other agents' fixes cascaded: 42/44 (was 40/44 at last scan).

Failing subtests:
- 33 "successful final commit clears revert state"
- 34 "reset after final pick clears revert state"

## Diagnosis
Both tests run `git revert picked-signed base` (a two-commit revert sequence).
The first revert commits cleanly; the second (`base`) conflicts, leaving REVERT_HEAD
and a `.git/sequencer/` dir whose `todo` has at most one remaining line. The test
resolves the conflict, then finishes with either `git commit -a` (33) or
`git reset` (34) and asserts `.git/sequencer` is gone.

`git reset` (reset.rs:1996-2006) already handled both CHERRY_PICK_HEAD and
REVERT_HEAD, so test 34 passed in isolation but the harness recorded it failing
because test 33's stale `.git/sequencer` leaked into it (pristine_detach does not
clean `.git`).

Root cause in grit/src/commands/commit.rs: the post-commit sequencer cleanup gate
```
if resume_pick_after_cp && sequencer_finished_last_pick(...) { remove sequencer }
```
only considered the cherry-pick head. The revert equivalent
(`_resume_revert_after_rv`) was computed but unused (underscore-prefixed), so a
conflicted revert finished by a plain `git commit` never removed `.git/sequencer`.

Upstream `sequencer.c:sequencer_post_commit_cleanup` sets `need_cleanup = 1` for
either CHERRY_PICK_HEAD or REVERT_HEAD, then removes the state when
`have_finished_the_last_pick()` is true.

## Fix
grit/src/commands/commit.rs:
- Renamed `_resume_revert_after_rv` -> `resume_revert_after_rv` (now used).
- Gate now `(resume_pick_after_cp || resume_revert_after_rv) && sequencer_finished_last_pick(...)`.

## Result
44/44 passing. grit-lib unit tests: only the 2 known pre-existing
ignore::gitignore_glob_tests failures (unrelated). No new clippy warnings in
commit.rs.
