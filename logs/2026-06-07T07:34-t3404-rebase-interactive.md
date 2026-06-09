# t3404-rebase-interactive — MOP-UP ROUND 2 (ticket 9e2eff)

Baseline at session start (fresh run): 101/132 pass, 31 fail.

## Failing set at start
18, 35, 50, 51, 53, 54, 57, 70, 72, 79, 80, 85, 86, 87, 88, 89, 91, 94, 95, 96, 103, 104, 107, 117, 120, 123, 125, 126, 127, 128, 129

## Fixes this session

### Fix 1: rebase (finish) reflog `old` value (test 18, cascade root)
`finish_rebase` (rebase.rs ~9123) read `old_branch_oid` from the branch ref FILE, which has
already been repointed to `new_tip` by replay completion. That recorded a `rebase (finish)` reflog
entry with `old == new`, so `git rev-parse branch@{1}` resolved to the new tip instead of the
pre-rebase tip. Fixed to use `rebase_orig_head_oid(rb_dir)` (the recorded original head) as the
reflog `old` value, falling back to the ref file. Test 18 (reflog shows state before rebase) passes.
102/132.

## Cascade investigation
- 86-89 pass when run as `--run=1-2,79-89` but fail in the full run -> a polluter exists between
  test 19 and 79 leaving bad on-disk state (stray rebase-merge dir or branch).

### Fix 2: plain `fixup` keeps pick-target message in squash chain (test 35)
`update_squash_message_file` (ctx.count==0 branch) treated a plain `fixup` like Git's
`is_fixup_flag` (the `fixup -C`/`-c` amend modes), commenting out the pick target's message. Git
keeps the 1st commit message UNcommented for a plain fixup; only the fixup commit's own message
(#2) is skipped. Gated the skip/comment behavior on `cmd == Fixup && fixup_message_mode.is_some()`.
Editor now sees `# This is the 1st commit message:\nB ... # This is the commit message #3:\nD`,
final message `B\n\nD\n\nONCE`. 103/132.

### Fix 3: value-less `--exec`/`-x` error wording (test 72)
Clap's "a value is required…" lacks "requires a value". Added a check in `preprocess_rebase_argv`:
a trailing bare `--exec` emits `error: option \`exec' requires a value`, `-x` emits
`error: switch \`x' requires a value`, exit 128 (Git's parse-options wording). 104/132.

### Fix 4: weave `--exec` into interactive todo + rebase.abbreviateCommands (tests 70; partial 107)
Git's `complete_action` runs `todo_list_add_exec_commands` after autosquash, interleaving `exec`
lines after each pick chain in the visible todo. grit kept `--exec` in a side file (`rebase-merge/
exec`), so cat-todo tests never saw them. Added `TodoBuildItem::Exec` + `insert_exec_commands`
(mirrors Git: arm after pick, flush before next non-fixup, decorations are inert), threaded
`&args.exec` into `run_interactive_rebase`, and skip the global-exec file in interactive mode
(execution comes from todo lines) to avoid double-run. Also honor `rebase.abbreviateCommands`
(p/r/f/s/x). Test 70 passes; 65-71 unregressed. 105/132.
  - Test 107 still fails: its `expected` is built from grit's `git rev-list --abbrev-commit` which
    does NOT abbreviate (returns full hex) — a rev-list bug, out of rebase scope. grit's todo
    correctly shortens, so expected(full) != actual(short). Needs a rev-list `--abbrev-commit` fix.

## Remaining fails (27) and notes
- 86-89, 120: CASCADE victims (pass in isolation). Origin is test 47 ('file named HEAD'): it builds
  branch3 whose squashed "Add head" commit matches `:/A` (younger than tag A). VERIFIED on an
  identical object DB that real git ALSO resolves `:/A`→"Add head" and ALSO conflicts at test 86 —
  so the divergence is accumulated repo state earlier in the suite, not a test-86 logic bug. Could
  not run the real upstream suite (needs a built git tree / GIT-BUILD-OPTIONS).
- 54: 'avoid unnecessary reset' — no-op `rebase -i` should fast-forward and NOT touch file3 mtime;
  grit replays each pick and rewrites file3. Needs git's post-editor "can fast-forward when todo
  unchanged/identical result" optimization.
- 91: short-commit-ID collide — needs collision-aware abbrev EXTEND in the reloaded todo plus
  git-rebase-todo.tmp/.backup generation during reword reload.
- 103/104: rebase --edit-todo missingCommitsCheck (two-phase parse-old-then-new).
- 123,125-129: --update-refs cluster. 123 needs update-ref insertion + autosquash threading applied
  to the --rebase-merges (label/reset/merge) script path (generate_rebase_merge_script bypasses
  build_autosquash_with_update_refs). 125-129 need edit-todo update-ref re-parse + ref-locking +
  failed-ref report.
- 117: post-commit hook not invoked during rebase replay (many commit sites; count-sensitive).
- 85: core.commentChar=auto deprecation warning (text not present in this checkout's C source).
- 50/51/53: submodule rebase.
- 79/80: --root untracked-file conflict.
