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

## Fix 2: static todo check (`todo_list_parse_insn_buffer` + `todo_list_check`)

grit had no upfront validation of the edited interactive todo. Added
`validate_edited_interactive_todo` (grit/src/commands/rebase.rs) mirroring git's
`rebase-interactive.c`: unknown command / bad SHA -> `error: invalid line N: <line>`;
leading fixup -> `error: cannot 'fixup' without a previous commit`; plus the
`rebase.missingCommitsCheck` warn/error path (warn continues, error aborts) printing
the exact `Warning: some commits...` block. All failures use
`explicit_exit::SilentNonZeroExit{code:1}` so no extra `error:` line leaks past the
git advice. It runs in `do_rebase` AFTER `checkout_onto` (git rewinds HEAD first, then
checks) so `--edit-todo` + `--continue` recovery works, AND in `do_edit_todo` comparing
against `git-rebase-todo.backup` (which the validator now always writes).

In-ISOLATION now passing: 100,101,102,108,109 (and 99,103? - check). Full-run count
stuck at 81 because the cascade: failing tests 94-96 (overwrite-untracked) leave a
rebase in progress that breaks the later missingCommitsCheck tests 100-106 when run in
sequence. MUST fix 94-96 (and 84/85/91/92) to unlock the sequence.

## Fix 3: untracked-file overwrite obstruction on fast-forward pick + REBASE_HEAD + drop in `done`

Three related machinery fixes (grit/src/commands/rebase.rs):
- The fast-forward-pick shortcut (`cherry_pick_for_rebase`, when HEAD == picked commit's
  original parent) skipped the untracked-file overwrite check. Added
  `reset::check_untracked_cherry_pick_obstruction` there (+ `obstructed-pick` marker), so a
  `pick` that would clobber an untracked working-tree file now halts (t3404 94-96).
- The PickLike error path now writes `REBASE_HEAD = <commit>` on any stop (git always exposes
  it; the conflict path set it inside cherry_pick, the obstruction path did not).
- `RebaseReplayStep::Noop` (i.e. `drop`) now appends its line to `done` for interactive rebases,
  so a later `--edit-todo` missing-commit check treats explicitly-dropped commits as "seen".
- `validate_edited_interactive_todo` now also marks commits recorded in `done` as kept, so
  mid-rebase `--edit-todo` after break/conflict doesn't falsely report already-replayed commits
  as dropped (fixed the 102/105 regression my Fix 2 introduced).

Now passing (full run): 99-102, 105, 106 (missingCommitsCheck), 108, 109 (static check),
65-68, 93, 111. Full run 76 -> 83.

Still failing: 103, 104 (`--edit-todo` missingCommitsCheck warn/error). These need git's
TWO-PHASE `edit_todo_list` semantics: parse the OLD/backup todo first (emitting
`error: invalid command 'pickled'` + `invalid line N`), THEN after the editor compare new-vs-backup
for the warning — distinct from grit's single-pass model. 94-96 halt correctly now but the
`--continue`-after-obstruction path (re-pick, "staged changes" guard) is still wrong.

## Fix 4: auto-amend-after-edit staged-changes guard + core.commentchar in replay/validation

- `--continue` after `edit`: when the user already committed (HEAD moved past the edited commit)
  AND the index still has staged changes, grit now errors `error: you have staged changes in your
  working tree` instead of silently amending (t3404 43 "auto-amend only edited commits after edit").
- `core.commentchar`: `validate_edited_interactive_todo` and the `replay_remaining` todo filter now
  honour the configured comment prefix (e.g. `\`), so commented-out todo lines are skipped rather
  than parsed as bad commands (t3404 84 "respects core.commentchar"). grit-side
  `comment_line_prefix_full` already maps `auto` -> `#`.

Full run holds at 83 (the amend fix alone churned the cascade to 82; the commentchar fix recovered
it while genuinely fixing 43 + 84).

Still failing 85 (core.commentchar=auto): needs git's multi-line Git-3.0 deprecation warning
(`Support for 'core.commentChar=auto' is deprecated...` + the scope-aware `git config unset/set`
advice block). Niche, single test, skipped for now.

## Fix 5: --root fixup/squash produces a true root commit

When a fixup/squash amends the rebased root commit, the result must itself be a root commit (no
parent). `cherry_pick_for_rebase` now computes `amend_parents` = empty when the HEAD being amended
has no parents (or its sole parent is the `--root` squash-onto sentinel), instead of
`vec![amend_parent]`. Verified in isolation (t3404 76 "rebase -i --root fixup root commit": parent
count now 0).

## Fix 6: --root sentinel HEAD on first-pick conflict (UNBLOCKED 75/76/77 + cascade)

When the FIRST pick of a `--root` rebase conflicts while HEAD is still unborn, grit left HEAD UNBORN
(zero) so `git cat-file commit HEAD` -> `fatal: Not a valid object name HEAD` and `git rebase --abort`
failed, corrupting state for 76/77 downstream. Fix: in `cherry_pick_for_rebase`'s conflict path, when
`head_at_empty_tree && root_rebase`, materialize + check out `ensure_squash_onto_fake_root` (empty-tree
root commit) as HEAD before bailing. Combined with Fix 5 (amend drops the sentinel/empty parent), this
made 75, 76, 77 pass without breaking 73/74/78. Full run 83 -> 87.

Still failing in this cluster: 79, 80 (`--root` when root has an UNTRACKED FILE conflict — a
different halt path; the untracked-overwrite obstruction during a root pick needs the same sentinel
HEAD treatment).

## Fix 7: edit-stop "You can amend" hint with -S<key> (gpg-sign)

When `edit` stops, grit now prints the git hint `You can amend the commit now, with\n\n  git commit
--amend <gpg_opt>\n\n...` echoing the shell-quoted `-S<key>` option (new `gpg_sign_opt_quoted`,
mirroring git's `sq_quotef("-S%s", key)`). Fixed t3404 113/114. Full run 87 -> 89.

## Fix 8: multiple --exec support

`Args.exec` changed `Option<String>` -> `Vec<String>` (clap collects repeated `-x`/`--exec`). All
bool checks updated (`is_some`->`!is_empty`, `is_none`->`is_empty`); pull.rs `exec: None` ->
`Vec::new()`. The `rebase-merge/exec` file now stores one command per line and the global-exec
runner loops over them (running each after each pick, rescheduling the failed one + the rest on
failure). Fixed t3404 69. Full run 89 -> 90.

Still failing 70 (`-ix --autosquash`): the global-exec-after-pick approach runs the exec BEFORE the
following fixup is applied, so `git show HEAD` shows the pre-fixup commit. Git inserts `exec` lines
into the todo AFTER autosquash (`todo_list_add_exec_commands`), so the exec lands after the fixup.
FIX NEEDED: switch from the `exec`-file global runner to inserting `exec <cmd>` todo lines after each
pick post-autosquash (would also align 107 abbreviateCommands+exec).

## Fix 9: core.abbrev in todo commit IDs

`format_rebase_todo_line` now passes `rebase_core_abbrev_len(config)` (from `core.abbrev`) as the
minimum abbreviation length to `abbreviate_object_id` (which still extends further on collision)
instead of a hardcoded 7. Fixed t3404 92/93 (full run 90 -> 92).

Still failing 91 (short commit ID collide): needs the collision-aware abbreviation to actually
EXTEND past the configured length when two object IDs share the prefix; verify `abbreviate_object_id`
honours the collision extension when min_len is small (core.abbrev=4 here).

## KEY: full-run vs isolation divergence
Many tests pass with `--run=1,N` but fail in the full sequential run because an EARLIER
failing test leaves a rebase-in-progress / wrong branch. `/tmp/run3404.sh` replicates the
real runner env (CRITICAL: avoids the GIT_EDITOR=true profile leak). Use
`sh /tmp/run3404.sh --run=1,A-B` to test a window. Fixing the earliest failures in a
cluster unblocks the rest.
