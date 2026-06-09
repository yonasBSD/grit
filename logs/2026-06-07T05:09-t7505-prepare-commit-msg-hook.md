# t7505-prepare-commit-msg-hook (ticket 5911b7)

Mop-up round. Fresh run after other agents' fixes: 21/23.

Failing subtests:
- 11 "with hook (editor)"
- 16 "with hook (rebase -i)"

## Subtest 11 — FIXED

`GIT_EDITOR=fake-editor git commit` (no -m). Hook replaces line 1 of the message
file with `default` via `sed -e "1s/.*/$source/"`.

Root cause: grit ran `prepare-commit-msg` *after* the editor, on the already
comment-stripped (empty) message buffer. `sed 1s/.*/default/` on an empty file
produces nothing, so the commit aborted "due to empty commit message".

Upstream `builtin/commit.c:prepare_to_commit` runs the hook on the *full template*
buffer (status comments included) BEFORE launching the editor (commit.c:1116 hook,
then 1120 launch_editor).

Fix (grit/src/commands/commit.rs):
- Added `run_prepare_commit_msg_hook_on(repo, args, index_path, use_editor, msg_file)`
  helper (hook on a given path; bails on non-zero exit).
- `prepare_commit_message` now takes a `run_prepare_hook: &dyn Fn(&Path)->Result<()>`
  closure and calls it immediately before each `launch_commit_editor` (6 sites).
- Caller builds the closure and passes it; the post-message hook block now only runs
  for the non-editor path (`if !use_editor_for_message`), since editor commits already
  ran the hook before the editor.

After build: subtest 11 passes. 22/23.

## Subtest 16 — FIXED

`with hook (rebase -i)`: full rebase replay with edit/squash/reword/fixup. Test reads
`git log --pretty=%s -g -n18 HEAD@{1}` and compares to `t7505/expected-rebase-i`.

Three distinct bugs, fixed incrementally:

1. `merge (no editor) [pick rebase-b]` should be `merge [pick rebase-b]`.
   The manual `git commit` while resolving a rebase conflict opens the editor in Git
   (`builtin/commit.c` only clears `use_editor` for `-m`/`-F`/`-c`/`-C`). grit's
   `commit_uses_editor_default` wrongly disabled the editor whenever MERGE_MSG/SQUASH_MSG
   existed, so the prepare-commit-msg hook saw `GIT_EDITOR=:` and tagged "(no editor)".
   Fix: removed the MERGE_MSG/SQUASH_MSG editor-disable branch (and dropped the now-unused
   `git_dir` param from `commit_uses_editor`/`commit_uses_editor_default`). Verified no
   regression in t7600/t3404/t7501/t7502 — tests pass `EDITOR=:` for post-merge commits, and
   grit's `launch_commit_editor` already no-ops on `:`.

2. `HEAD [edit rebase-13]` should be `message [edit rebase-13]`.
   `git rebase --continue` after an `edit` stop runs `git commit --amend -e -F <message>`
   (`commit_staged_changes`); the `-F` makes prepare-commit-msg arg1="message". grit reused
   the `reword` path (`run_commit_editor_for_reword` → arg1="commit", arg2="HEAD"), producing
   "HEAD". Fix: added `run_edit_continue_editor` using arg1="message", None.

3. Missing reflog entry for the `edit` amend (HEAD@{1} showed the un-amended pick).
   The `rebase-amend-continue` block wrote HEAD directly and only appended a HEAD reflog entry
   when `user_made_own_commit`. Git's `commit_staged_changes` amends via `git commit --amend`
   with `GIT_REFLOG_ACTION="<action> (continue)"`, which records its own reflog entry. Fix:
   append the `<action> (continue): <subject>` reflog entry unconditionally after the amend.

Result: t7505 23/23, fully passing.

## Regression verification

The shared release binary also contains other agents' in-flight merge.rs/log.rs edits, which
currently fail many t3404/t7600 subtests. Confirmed those are NOT mine by rebuilding with the
committed versions of commit.rs/rebase.rs: t3404 failure set is byte-identical (zero delta),
t7600 fails exactly {66,70} with and without my changes, t7501/t7502 zero delta.
