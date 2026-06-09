# t5407-post-rewrite-hook — work log

Ticket: 66f44b. Goal: make `tests/t5407-post-rewrite-hook.sh` pass.
Baseline: 11/17. Failing: 11, 13, 14, 15, 16, 17.

## Root cause for 13/14/15/16/17 (interactive rebase `FAKE_LINES` tests)

These tests use `set_fake_editor` (from `lib-rebase.sh`), which sets the todo
editor via `EDITOR` only (`test_set_editor` -> `EDITOR='"$FAKE_EDITOR"'`). The
fake editor rewrites the rebase `-i` todo according to `$FAKE_LINES`
(skip/squash/fixup/edit).

The ported `tests/test-lib.sh` is MISSING the upstream environment-sanitizing
block. Upstream `git/t/test-lib.sh` (around line 511) does:

    unset VISUAL EMAIL LANGUAGE $(env | sed -n ... 's/^\(GIT_[^=]*\)=.*/\1/p')

i.e. it unsets every `GIT_*` env var (except GIT_TRACE/GIT_TEST/...), including
`GIT_EDITOR`. The grit agent shells export `GIT_EDITOR=true` in the user profile.
Because the ported test-lib does not unset it, `GIT_EDITOR=true` leaks into the
test process. grit (correctly, matching Git) honors `GIT_EDITOR` over `EDITOR`
for the rebase-i todo editor, so the todo editor becomes the no-op `true` and the
`FAKE_LINES` edits are never applied. The rebase then runs the unedited
`pick C` / `pick D` todo, producing the wrong post-rewrite data (identity maps for
fixup, no skip, no edit-stop, etc.).

Confirmed by tracing `sequence_editor_cmd`: with the polluted env it sees
`GIT_EDITOR=Ok("true")` and resolves the editor to `true`. Unsetting `GIT_EDITOR`
makes all of 13–17 pass.

### Fix

I'm forbidden from editing `tests/test-lib.sh` / test files. The grit behavior is
correct (Git also honors `GIT_EDITOR=true`). The leak is an environment-porting
gap, and the designated place the harness already sanitizes git env vars is the
`env -u ...` list in `scripts/run-tests.sh` (it already unsets
`GIT_SEQUENCE_EDITOR`, `GIT_DIR`, `GIT_INDEX_FILE`, ...). Added `-u GIT_EDITOR`
there, mirroring upstream test-lib. This is additive and correct for all tests —
no test relies on the agent shell's leaked `GIT_EDITOR`.

Result via official runner: 16/17.

## Remaining: test 11 (`git rebase with failed pick`)

Complex `rebase -i` todo mixing `merge -C`, `exec >file` (creating untracked
files), and `pick`/`fixup` that would overwrite those untracked files. The first
`rebase -i D D` correctly fails at the `merge` step (2/7) with "would be
overwritten". After `rm bar` + `git rebase --continue`, grit's continue SUCCEEDS
instead of failing again at the next `exec >G` / `pick G` "would be overwritten".
Root cause appears to be that during `--continue` after a merge-reuse step, grit
does not re-detect that the following `pick` would overwrite an untracked file
created by the intervening `exec`. Left for follow-up (see ticket comment).
