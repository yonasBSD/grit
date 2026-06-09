# t3436-rebase-more-options.sh — rebase --ignore-whitespace / date options

Ticket: 161432 (thread A, rebase-core / sequencer machinery)

## Result
Went from 2/19 → 19/19 (fully passing).

## Root causes & fixes (grit/src/commands/rebase.rs)

### 1. `--ignore-whitespace` / `--ignore-space-change` ignored during the content merge
The rebase replay (`cherry_pick_for_rebase`) selected the per-file merge engine
based only on `ws_fix_rule`. For the common case (commit with parents, no
`--whitespace=fix`), it used `merge_trees_for_single_cherry_pick`, which does
**not** thread `ignore_space_change` into the 3-way merge. So `--ignore-whitespace`
(apply backend = `git am --ignore-whitespace`; merge backend = `--ignore-space-change`
to the strategy) had no effect and a whitespace-only divergence spuriously
conflicted (tests 2,3,4).

Fix: when `replay_opts.ignore_space_change` is set, route through
`three_way_merge_with_content` (which already passes `ignore_space_change` into
`merge()`), and skip the `overlapping_content_changes` re-conflict pass that would
otherwise re-introduce the whitespace conflict.

### 2. `--reset-author-date` / `--committer-date-is-author-date` not applied when committing a conflict resolution via `--continue`
The conflict-resolution branch of `do_continue` read the author line straight from
`rebase-merge/author-script` (the picked commit's original identity + date) and
used it verbatim, bypassing the date-rewriting helper. So a commit finished after
manual conflict resolution kept its original author date/timezone instead of being
reset (tests 13, 15).

Fix: capture `raw_author_for_replay` (author-script when present, else the picked
commit's author) and route it through `rebase_replayed_author_line` /
`rebase_replayed_committer_line` (same helpers the clean picks use). This is a
no-op when no date options are set (`rebase_replayed_author_line` returns the raw
author unchanged), so non-date rebases are unaffected.

## Notes for next agent
- rebase.rs is a SHARED file in this thread; while I worked another agent had
  uncommitted hunks in it (GRIT_DEBUG_OBSTRUCT removal + `fixup_was_obstructed`
  logic for t5407). Those are NOT mine.
- t3404-rebase-interactive fluctuated (HEAD TOML=77, mid-run=75, my run=74) due to
  that other agent's obstruction work, NOT my changes: my edits only activate with
  `--ignore-whitespace` or date options, neither of which t3404 exercises.
- No regression from my changes: t3406 (32/32), t3407 (17/17), t3403 (20/20),
  t3418 (29/30, unchanged) all stable.
