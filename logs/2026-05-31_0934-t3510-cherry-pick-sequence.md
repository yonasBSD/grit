# t3510-cherry-pick-sequence: 34/55 -> 55/55

Branch: wf/p6/t3510-cherry-pick-sequence
Base: 4f49b581273d204c74cdb7efa02702fdd31c8b07

## Result
- t3510-cherry-pick-sequence: 34/55 -> 55/55 (fully green)
- Side improvements (no regressions anywhere):
  - t3502-cherry-pick-merge: 3/12 -> 12/12
  - t4013-diff-various: 51/230 -> 62/230
- grit-lib --lib unit tests: 228 passed, 0 failed.

## Root causes fixed (5 clusters)

### 1. checkout `<tree> -- <path>` left conflict stages (tests 3, 41, 42)
`grit checkout HEAD -- foo` after a conflict called `index.add_or_replace` with a
stage-0 entry but did not remove the leftover stage 1/2/3 unmerged entries, so the
path still looked unmerged. Fix in `grit/src/commands/checkout.rs` (the
`Some(source_spec)` branch, glob/dir/single-file cases): call
`index.remove_path_all_stages(path)` before `add_or_replace`, collapsing all stages
to a single stage-0 entry (matches git).

### 2. diff-tree `-s` not honored in single-commit path (test 38)
`git diff-tree -s --pretty=tformat:%s HEAD` printed the subject AND the raw diff line.
Fix in `grit/src/commands/diff_tree.rs`: in `run_one_commit`, skip `print_diff` when
`opts.suppress_diff` is set (root + single-parent paths), and in `write_commit_header`
omit the trailing blank line after the `tformat:%s` subject when the diff is suppressed.

### 3. Sequencer todo dropped the conflicting commit + auto-resume on plain commit
(tests 16, 40, 43, 46, 49, 50, 53, 54, 55, and prerequisites of 5/6/7/15/17/19/26/39)
Two coupled bugs, fixed to match git's sequencer model:
- `run_commit_sequence` saved `remaining = &oids[i+1..]` on a stop, dropping the
  conflicting commit from `sequencer/todo`. Changed to `&oids[i..]` so the conflicting
  pick stays at the head of the todo (git keeps `pick <conflicting>` first).
- A plain `git commit` resolving a conflict auto-resumed the rest of the sequence via
  `try_resume_pick_sequence_after_commit` (commit.rs). git does NOT do this; only
  `--continue` advances. Removed the auto-resume call (functions kept, dead-code-marked).
- `do_continue` (post-commit branch) now verifies the index matches HEAD, then strips
  the already-committed head pick from the todo (git's `todo_list.current++`) before
  replaying the rest. `reset` no longer deletes the `sequencer/` dir while picks remain
  (mirrors git's `have_finished_the_last_pick`: dir torn down only when todo <= 1 line).
- A nested standalone `cherry-pick <single commit>` mid-sequence no longer touches the
  sequencer state (git's `single_pick` fast path); the old todo-line-stripping there
  corrupted the sequence (tests 3, 15).

### 4. --skip / --continue replayed the just-handled pick (tests 5, 6, 7, 15, 19, 39)
`do_skip` and the post-resolution `do_continue` loaded `remaining` BEFORE stripping the
head todo line, so the replay re-ran the committed/skipped commit (showing up as an
empty pick). New helper `skip_current_pick_and_continue` strips the head line first,
then loads `remaining`, preserving the stored pre-sequence HEAD. The continuation replay
(`orig_head_override` set) now always cleans up `sequencer/` on completion, even with a
single remaining pick (test 45).

### 5. option-compat check + signoff reaffirmation (tests 5/6/7, 45, 47, 46, 48)
- `verify_pick_flags_not_with_operation` now runs on the raw argv flags BEFORE
  `merge_sequencer_opts`, matching git's `verify_opt_compatible` (which runs before
  `read_populate_opts`). Persisted flags carried forward no longer trip the
  `--empty/-x/--signoff cannot be used with --skip/--continue` check.
- The conflict-resolution commit no longer auto-applies persisted `--signoff`. Signoff
  is baked neither into MERGE_MSG at conflict time nor in the `--continue` resolution
  commit unless `-s` is re-affirmed on the `--continue` command line. `-x` is still
  carried (test 45). Fresh picks replayed by the continued sequence still get the
  persisted `-s` (test 46). Tests 46/47/48 were `test_expect_failure` upstream and are
  `test_expect_success` in the harness; this implements the desired behavior.

## Files changed
- grit/src/commands/checkout.rs
- grit/src/commands/cherry_pick.rs
- grit/src/commands/commit.rs
- grit/src/commands/diff_tree.rs
- grit/src/commands/reset.rs
- grit/src/commands/revert.rs

## Regression check (all vs baseline, no regressions)
- t3501-revert-cherry-pick 20/21 (unchanged, pre-existing 1 fail)
- t3502-cherry-pick-merge 3/12 -> 12/12
- t3511-cherry-pick-x 22/22 (unchanged)
- t7110-reset-merge 21/21, t7110-reset-modes 2/20 (unchanged, pre-existing)
- t2008-checkout-subdir 9/9, t2070-restore 15/15, t7201-co 28/46 (unchanged)
- t4013-diff-various 51/230 -> 62/230
- t7102-reset 36/38 (unchanged)
- t5813-proto-disable-ssh 81/81, t5563-simple-http-auth 17/17,
  t5547-push-quarantine 6/6 (all unchanged)
