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
