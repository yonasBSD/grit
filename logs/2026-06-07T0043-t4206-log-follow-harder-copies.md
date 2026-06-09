# t4206-log-follow-harder-copies — 2026-06-07

Ticket: 238f15 — tests/t4206-log-follow-harder-copies.sh
Group: line-log (thread C). Was 6/7; subtest 5 ("validate the output") failing.

## Diagnosis

`git log --follow --name-status --pretty="format:%s" path1` produced wrong output
in three ways:

1. The copy commit showed `A\tpath1` instead of `C100\tpath0\tpath1`. `--follow`
   ran copy/rename detection in `follow_filter` to decide which commits to keep
   and how to retarget the path, but threw away that result — the displayed diff
   was recomputed plainly via `compute_commit_diff` and filtered to the original
   pathspec, so neither the copy detection nor the retargeting reached the output.

2. The log name-status renderer in `write_commit_diff_body` only printed
   `<letter>\t<path>`; it never emitted `R100/C100 <old> <new>` for renames/copies
   (a latent bug, exposed once copy detection reached the renderer).

3. Separator newline between the commit message and the `--name-status` /
   `--name-only` block was wrong: no leading newline (so the subject and the
   first status line were glued together under `format:`), plus a spurious
   trailing blank (double blanks under builtin formats).

## Fix (grit/src/commands/log.rs only)

- `follow_filter` now also returns a per-commit `FollowDisplay { display_path,
  display_entry }` map: the path the followed file had in that commit (the copy/
  rename destination) and the already copy/rename-detected diff entry.
- `write_commit_diff` gained a `follow_override: Option<&FollowDisplay>` arg.
  When set it restricts the displayed diff to `display_path` (retargeting older
  commits to the source name) and substitutes the detected `C100/R100` entry for
  the plain add/modify entry, so name-status/raw/patch all show the copy.
  Passed through from the non-streaming display loop; `None` at the other 5 call
  sites.
- Log name-status renderer now prints `<letter><score>\t<old>\t<new>` for
  Renamed/Copied, matching diff-tree.
- New helper `log_name_list_needs_separator` (= `!oneline`): git always prints
  exactly one newline before the name-only/name-status block except for
  `oneline`. With `format:` that newline terminates the unterminated subject
  (no visible blank); for tformat/%s/medium/etc it shows as a blank. Both the
  name-only and name-status blocks now use it, and the name-status block no
  longer prints a trailing blank (the inter-commit blank comes from the caller).
  This also fixed the pre-existing `--oneline --name-only` extra-blank bug.

## Result

- t4206: 7/7 (was 6/7).
- t4205-log-pretty-formats: 123/125 (= baseline, 2 expect_failure).
- t4202-log: 129/149 (baseline 128 — +1 improvement, no regression).
- t4013-diff-various 230/230, t4216-log-bloom 167/167, t4001-diff-rename 23/23,
  t4023-diff-rename-typechange 4/4 all still full.
- grit-lib --lib: 272 pass, only the 2 known ignore::gitignore_glob failures.

## Out of scope / not fixed

- `git log -p --name-status`: grit prints the patch where git suppresses it when
  name-status is also requested. Pre-existing, orthogonal to t4206, untouched.
