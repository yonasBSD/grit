# t3404-rebase-interactive (ticket 9e2eff)

Subsystem: rebase-core (interactive rebase / sequencer machinery).

## Key environment gotcha (do NOT trust direct `sh ./t3404...` runs)

Running `sh ./t3404-rebase-interactive.sh` directly leaks the agent shell's
profile env (notably `GIT_EDITOR=true`), which makes grit resolve the sequence
editor to `true` and silently skip all fake-editor edits — producing a flood of
*false* failures (tests 3,4,5,...). The official `scripts/run-tests.sh` (and
`/tmp/run3404.sh`, which replicates its `env -u GIT_EDITOR ...` invocation) gives
the true result. Always reproduce via `/tmp/run3404.sh`.

## Fix 1: interactive todo missing the help/instruction comment block

`run_interactive_rebase` (plain `rebase -i` path, grit/src/commands/rebase.rs)
wrote the todo as bare pick lines with no trailing blank line + `# Rebase ... onto
... (N commands)` help block. Real git appends that block (preceded by a blank
line). Several `--exec` tests (65-72) calibrate `sed 1,Nd` against that layout
(the blank line survives the fake editor's `grep -v '^#'`), so grit's output was
off by one line.

Fix: `run_interactive_rebase` now takes a `revs_onto` arg and calls
`append_rebase_todo_help` (same helper the `--rebase-merges` path already used),
computing `revs = <short-upstream>..<short-orig-head>` and `onto = <short-onto>`.

Result: 76 -> 81 / 132.

## Remaining failures (51) — clusters to investigate
- 122-129: --update-refs (label/update-ref command generation + application)
- 100-111: rebase.missingCommitsCheck warn/error + static checks of bad command/SHA
- 75-80: rebase -i --root (sentinel/fixup/reword)
- 84,85,92,107: core.commentchar / core.abbrev / abbreviateCommands
- 94-96: commits that overwrite untracked files
- 113,114: --gpg-sign
- 117-120: post-commit hook / empty pick errors / onto hash
- 18,35,43,45,46,47,48,50,54,57,69,70,72,81,91,108,109,130,131 misc
