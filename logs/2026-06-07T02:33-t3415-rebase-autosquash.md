# t3415-rebase-autosquash (2026-06-07)

## Result

- Harness: **28/28** passing (`env -u GIT_EDITOR ./scripts/run-tests.sh t3415-rebase-autosquash.sh`).
- Previously 27/28; remaining failure was **test 28 'pick and fixup respect commit.cleanup'**.

## Diagnosis

The single real failure was test 28. A `prepare-commit-msg` hook appends `# Prepared`
to commit messages; after an autosquash fixup folds `fixup! second commit` into
`second commit`, the expected folded message is `second commit\n\n# Prepared`
(with `commit.cleanup` unset → NONE → comment lines survive), and just
`second commit` when `commit.cleanup=strip`.

grit produced only `second commit` in the unset case — it stripped the hook's
`# Prepared` line.

Root cause in `grit/src/commands/rebase.rs`, final-fixup folding path
(`cherry_pick_for_rebase`, `todo_cmd == Fixup`, no editor): grit ran the
`prepare-commit-msg` hook on the *raw squash template* (which still carried the
`# This is a combination of 2 commits.` / `# The commit message #2 will be skipped:`
headers) and *then* called `cleanup_squash_editor_message`, which strips ALL `#`
lines — including the hook-appended `# Prepared`.

Upstream `sequencer.c` differs: `append_squash_message` writes only the plain
folded message *body* (no squash headers) to `message-fixup`
(`skip_blank_lines(buf->buf + fixup_off)`), and `try_to_commit` runs
`prepare-commit-msg` on that body, then applies only `commit.cleanup`
(`COMMIT_MSG_CLEANUP_NONE` when unset, so no stripspace at all).

## Fix

In the non-editor final-fixup branch, collapse the squash template to its message
body *first* (`cleanup_squash_editor_message(&tmpl, &config)` → `second commit`),
run the `prepare-commit-msg` hook on that body, then apply only
`apply_commit_msg_cleanup(&raw, rebase_commit_msg_cleanup(&config))`. The editor
branch is unchanged (the editor must still see the full template with headers).

This preserves hook-appended comments when `commit.cleanup` is unset and strips
them when `commit.cleanup=strip`, matching Git. No regression in t3437
(13/13), t7505 (unchanged 21/23 — its 2 failures are pre-existing and unrelated).

## Environment note (not a grit bug)

While diagnosing, tests 21/23/26 appeared to fail in an interactive shell where
`GIT_EDITOR=true` leaked from the Claude Code shell snapshot — that env var
overrides the harness's per-test editor (`core.editor`, `EDITOR`), so the
sequence editor resolved to `true` and never ran the test's failing editor. The
official `run-tests.sh` / CI environment has no `GIT_EDITOR`, so those tests
pass there; reproduce with `env -u GIT_EDITOR`. Only test 28 was a genuine bug.
