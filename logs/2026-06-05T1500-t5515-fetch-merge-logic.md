# t5515-fetch-merge-logic (ticket 08cdf4)

Upstream test "Merge logic in fetch". Started 1/65 passing.

## Root causes found

1. **FETCH_HEAD URL `.git` stripping**: For configured remotes with URL `../.git/`, FETCH_HEAD
   showed `branch 'main' of ../.git` instead of `of ../`. Git (`builtin/fetch.c` `display_state`
   init) trims trailing slashes then drops a trailing `.git` when there is more than `X.git`
   before it. Fixed `normalize_fetch_url_display` in `grit/src/commands/fetch.rs` to match.

2. **Spurious `refs/remotes/<remote>/HEAD`**: grit's set_head mapped the remote default branch
   through the configured fetch refspec and wrote `refs/remotes/config-explicit/HEAD ->
   refs/remotes/rem/main`. Git's `set_head` always targets `refs/remotes/<remote>/<head_name>`
   (implicit `refs/heads/*` guess) and requires that ref to exist; for refspecs mapping elsewhere
   (rem/*), the target does not exist, so no HEAD is written. Fixed in `fetch.rs` set_head block.

3. **Tag pointing at a tree aborts protocol-v0 negotiation**: with `GIT_TEST_PROTOCOL_VERSION=0`,
   `peel_commit_oid_for_negotiation` errored ("object ... does not name a commit") when an
   advertised/local tag (`tag-one-tree`) peeled to a tree, aborting the whole fetch. Git silently
   skips non-commit refs in negotiation. Changed `peel_commit_oid_for_negotiation`
   (`grit/src/fetch_transport.rs`) to return `Option` via `try_peel_to_commit_for_merge_base` and
   skip `None` at all 7 call sites.

4. **Octopus for-merge marking**: `branch.<name>.merge` with multiple values (octopus) and short
   names (e.g. `two`) were not matched. `fetch_head_is_for_merge_with_branch` now iterates
   `config.get_all("branch.<b>.merge")` and matches via `ref_rev_parse_rules` (`refname_match`).

## Progress
1/65 -> 13/65 after fixes 1-4.

Remaining: remote-explicit/glob, branches-default/one groups, `main`/`br-unconfig` default-fetch
and CLI-url cases. Investigating.

## Round 2 (after PR #800 merged round 1)

5. **Legacy `.git/remotes/<name>` default-remote resolution**: `git fetch` (no args) on a branch
   whose `branch.<b>.remote` names a legacy `.git/remotes/<name>` / `.git/branches/<name>` remote
   fell back to `origin` because `default_fetch_remote_name` only accepted remotes with a
   `remote.<name>.url` config. Now also accepts remotes defined by those files.

6. **Legacy remote tracking refspecs ignored**: `union_refspecs` (used for the actual ref writes)
   fell back to the synthetic `refs/remotes/<name>/*` for remotes with no `remote.<name>.fetch`,
   ignoring the legacy `Pull:`/branches refspecs. Now uses the computed `refspecs` for legacy and
   `.git/branches` remotes.

7. **`.git/branches/<name>` refspec shape**: implemented Git `read_branches_file` semantics —
   fetch `refs/heads/<frag>:refs/heads/<name>` (frag = `#frag` or repo default branch). Added
   `repo_default_branch_name`.

Progress 13/65 -> 39/65. Remaining: branches-one (#frag), branches-default merge/octopus variants,
and the CLI `../.git` url cases (main/br-unconfig + tag args). Investigating.

## Round 3

8. **First-refspec for-merge head**: `fetch_head_is_for_merge_first_refspec_only` used advertised
   `idx==0`, but Git marks the first ref *matched by the first non-pattern refspec* regardless of
   advertised position. Replaced with `fetch_head_first_refspec_merge_ref` (matches via
   `refname_match`), fixing branches-one (`one` for-merge, not idx 0).

9. **`add_merge_config`**: for default fetch, `branch.<b>.merge` entries not already fetched by the
   configured refspec are fetched FETCH_HEAD-only (no tracking ref) and marked for-merge, prepended
   in merge-config order. Fixes branches-default/one merge + octopus (merge refs three / one,two
   absent from the branches refspec).

Progress 43/65 -> 51/65. Remaining 14: all CLI `../.git` url cases (main/br-unconfig with
positional refspec / --tags / tag args). Investigating.

## Round 4 — FULLY PASSING 65/65

10. **Anonymous URL fetch with no refspec**: `git fetch ../.git` synthesized a default tracking
    refspec `refs/remotes/<url>/*` (writing broken `refs/.git/*` refs). Now no opportunistic
    tracking refspec is synthesized for URL fetches; only HEAD lands in FETCH_HEAD (bare-URL line).

11. **Tag auto-follow with dst-less CLI refspecs**: `git fetch ../.git one` auto-followed tags it
    shouldn't. Git only auto-follows tags when a fetched refspec has a destination (`autotags`).
    Added `cli_refspecs_have_dst` gating to `should_fetch_tags`.

12. **`tag <name>` CLI refspecs**: emitted a mislabeled `branch '<name>'` duplicate. Now skip the
    branch-style line for `refs/tags/*` srcs (the dedicated CLI-tag block emits `tag '<name>'`),
    and auto-followed (non-requested) tags are emitted as not-for-merge alongside.

13. **Non-commit for-merge downgrade**: tags pointing at trees/blobs (`tag-one-tree`,
    `tag-three-file`) must be not-for-merge (Git `write_fetch_head` downgrades when the object is
    not a commit). Added `downgrade_non_commit_for_merge_lines`.

14. **Tag FETCH_HEAD ordering**: tags now keep emission order (explicit CLI tags first, then
    auto-followed) instead of being name-sorted, via a stable Equal compare in
    `sort_fetch_head_lines`.

FINAL: 65/65 passing. No regressions in t5510/t5514/t5503/t5582/t5511/t5536 (all still fully pass);
t5505 went 67->68. The only failing grit-lib unit tests (ignore::gitignore_glob_tests x2) are
pre-existing and unrelated to fetch.
