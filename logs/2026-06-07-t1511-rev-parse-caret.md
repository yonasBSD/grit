# t1511-rev-parse-caret

Ticket: 9d8486

## Problem

grit's rev-parse `^{/<regex>}` commit-message search did not implement the
special leading-`!` semantics from git's `get_oid_oneline` (object-name.c):

- `!` followed by anything other than `!` or `-` is RESERVED -> must fail.
- `!!<pat>` escapes to a literal `!<pat>` regex search.
- `!-<pat>` is a NEGATED search (find first commit NOT matching `<pat>`).

The `^{/...}` peel path went through `resolve_commit_message_search_from`,
which treated the pattern as an ordinary regex with no `!` handling, so
reserved forms resolved instead of failing and negated forms were unhandled.

(The sibling `:/...` form in `resolve_commit_message_search` already had
partial `!`/`!!` handling but not `!-` or the reserved-form failure; the
ticket scope is the `^{/...}` path, which I fixed.)

## Fix

grit-lib/src/rev_parse.rs:

- Added `parse_oneline_pattern(&str) -> Option<(bool /*negate*/, &str)>`
  mirroring git's `get_oid_oneline` prefix logic:
  - no leading `!` -> `(false, pattern)`
  - `!-<pat>` -> `(true, <pat>)`
  - `!!<pat>` -> `(false, !<pat>)` (keep one `!` so regex matches literal)
  - otherwise (reserved) -> `None`
- `resolve_commit_message_search_from` now calls it; `None` => ObjectNotFound
  (fatal), and matching uses `negate ^ base_match`.

Search starts from the given OID and walks ancestors. `CommitData.message`
is already the body after the header `\n\n`, matching git's `p + 2` search
start. The t1511 history is linear, so ancestor-walk order == date order
(git uses a commit-date prio queue).

## Result

./scripts/run-tests.sh t1511-rev-parse-caret.sh => 17/17 (was 11/17).
cargo test -p grit-lib --lib: only the 2 known pre-existing
ignore::gitignore_glob_tests failures (unrelated).
