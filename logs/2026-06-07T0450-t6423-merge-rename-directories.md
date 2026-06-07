# t6423-merge-rename-directories — rename/rename(2to1) transitive conflict labels

Date: 2026-06-07T04:50 (UTC)
Branch: grit-t5-progress

## Ticket / starting state

No open or closed ticket existed for this file; created a fresh one (tags: test, t6).
Re-ran the file fresh: 80/82 markers passing (the 2 non-passing are `# TODO known
breakage` / `test_expect_failure`), with one real failure:

- **test 23 — "7b: rename/rename(2to1), but only due to transitive rename"**

## Diagnosis

Testcase 7b is a rename/rename(2to1) where the two colliding files arrive at the
shared destination `y/d` differently:

- HEAD: `git mv w/d y/`  → `w/d` renamed directly to `y/d`.
- B:     `git mv x/d z/`  → `x/d` renamed to `z/d`, then a *directory* rename
  (HEAD renamed `z/` → `y/`) transitively moves it to `y/d`.

The final two-way collision merge in the working-tree file `y/d` had its conflict
marker labels wrong:

```
got:       <<<<<<< HEAD        ...  >>>>>>> B^0
expected:  <<<<<<< HEAD:y/d    ...  >>>>>>> B^0:z/d
```

The grit 2to1 pre-pass (`merge_trees` in `grit/src/commands/merge.rs`, the
"different sources renamed to the same destination" block) performed the final
empty-base two-way merge of the two staged blobs using the bare branch labels
`ours_label` / `their_name`, never the per-side `:path` suffix.

Ground truth: `git/merge-ort.c` `merge_3way()` builds the labels as
`branch:pathname` whenever the three `pathnames[]` of the colliding entry are not
all identical, and bare `branch` when they are. Each side's pathname is its rename
target *before* any directory rename was applied (`apply_directory_rename` copies
the pre-dir pathname via `new_ci->pathnames[index] = ci->pathnames[index]`).

For 7b the pathnames are (w/d, y/d, z/d) → not all equal → suffixed labels
`HEAD:y/d` and `B^0:z/d`.

## Fix

In the 2to1 pre-pass, before the final two-way `try_content_merge`:

- Added a long-lived `ours_renames_pre_dir_for_labels` map (mirror of the existing
  `theirs_renames_pre_dir_for_labels`), populated with the pre-directory-rename
  rename targets.
- Computed `ours_marker_path` / `theirs_marker_path` = each side's pre-dir target
  (falls back to the destination for a plain rename with no directory rename).
- Suffix the labels with `:path` only when the pathnames are NOT all equal to the
  destination (`pathnames_all_equal`); otherwise keep the bare branch labels.

The conditional is essential: `t6416` "check nested conflicts" (test 38) renames
both sides directly to `m` (no directory rename), so all pathnames equal `m` and
the outer collision markers must stay bare `HEAD` / `R2^0`. An earlier unconditional
suffix broke that case; the conditional restores it.

## Verification

- `t6423-merge-rename-directories.sh`: **passed all remaining 80 test(s)** →
  `fully_passing = true`, `failing = 0` in the TOML.
- `t6416-recursive-corner-cases.sh`: passes (test 20 confirmed restored by the
  conditional; test 38 unaffected by this change).
- `t6422-merge-rename-corner-cases.sh`: unchanged from its committed baseline
  (1 pre-existing failure — not this ticket).
- `cargo test -p grit-lib --lib`: only the 2 known `ignore::gitignore_glob_tests`
  failures (pre-existing, unrelated).
- No new clippy warnings in the changed lines.

Note: while diagnosing, an unconditional-suffix interim version was bisected against
a committed-state binary to prove the t6416 test-38 wobble was pre-existing
(introduced by the partial-clone prefetch commit `19177c1dc`), not by this change.

## Files changed

- `grit/src/commands/merge.rs` — 2to1 collision label fix (described above).
- `data/tests/t6/t6423-merge-rename-directories.toml` — refreshed run results.
- `logs/2026-06-07T0450-t6423-merge-rename-directories.md` — this log.
