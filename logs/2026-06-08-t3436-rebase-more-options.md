# t3436-rebase-more-options — date options must force-rewrite root picks in `rebase -r`

Ticket: 809c7b (regression vs closed 161432)

## Starting state
- 16/19 passing. Failing:
  - not ok 8  — `--committer-date-is-author-date` works with `rebase -r`
  - not ok 9  — `--committer-date-is-author-date` works when forking merge
  - not ok 14 — `--reset-author-date` works with `rebase -r`

## Root cause
The ticket suspected the merge directive path. After reproducing subtest 8
manually, the recreated **merge** commit's dates were actually correct
(`rewrite_merge_head_for_replay_opts` handles `committer_date_is_author_date` /
`ignore_date`). The real culprit was the **root pick** in the `rebase -r` script.

The generated todo for `rebase -r --root` is:
```
reset [new root]
pick <root> # add file
...
```
`cherry_pick_for_rebase` fast-forwards a root pick that sits on the `[new root]`
squash-onto sentinel when `!force_rewrite_commits` (rebase.rs ~8684), reusing the
ORIGINAL root commit object — so its original committer date (2005) survived,
breaking `test_ctime_is_atime` (and `test_atime_is_ignored` for `--reset-author-date`).

`force_rewrite_commits` comes from `rebase_force_rewrite_requested(args)`, which
only checked `--no-ff` / `--signoff` / `--trailer`. Git's `builtin/rebase.c`
additionally sets `REBASE_FORCE` (which clears `replay.allow_ff`) when
`--committer-date-is-author-date` or `--reset-author-date`/`--ignore-date` is set
(rebase.c:1478-1479, 188). With `allow_ff` cleared, even a tree-identical root
pick is rewritten with the new timestamp.

## Fix
`grit/src/commands/rebase.rs` — `rebase_force_rewrite_requested` now also returns
true for `args.committer_date_is_author_date || args.reset_author_date`. This
propagates as `force_rewrite_commits` into `replay_remaining` → the per-pick
fast-forward guard, so root picks (and any tree-identical picks) are rewritten and
pick up the corrected committer/author dates. Mirrors Git's REBASE_FORCE behavior.

All three failing subtests shared this single root cause (subtest 9's forking
merge also relies on the rebuilt first-parent chain dates).

## Result
- t3436-rebase-more-options: 19/19 fully passing.
- cargo test -p grit-lib --lib: 276 pass, only the 2 known pre-existing
  `ignore::gitignore_glob_tests` failures (unrelated to this ticket).
