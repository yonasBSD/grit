# t5526-fetch-submodules — mop-up round 1 (ticket 1d762d)

Date: 2026-06-07T06:14Z
Agent: schacon+claude-t5@gmail.com
Branch: grit-t5-progress

## Starting state

Re-ran fresh: 47/56 (matches prior agents' last recorded count). Four prior
sessions had already taken this file from 19 -> 47 and root-caused all 9
remaining failures. I re-verified their analysis and fixed the one genuine
grit bug they had deferred.

## Fix this session: subtest 38 "fetching submodule into a broken repository"

Genuine grit bug (in scope). After `rm -r dst/sub/.git/objects` the submodule's
HEAD ref still resolves to a commit OID, but that commit object is missing from
the submodule's own object store. Upstream git's `is_submodule_modified`
(git/submodule.c:1881) shells `git status --porcelain=2` into the submodule;
that inner status fails to read HEAD's tree, the subprocess exits non-zero, and
`finish_command` makes the surrounding `status`/`diff`/`fetch` die. grit instead
returned the submodule as silently clean, so `git status` / `git diff` exited 0
where the test does `test_must_fail`.

Root cause in grit: `grit-lib/src/diff.rs::submodule_porcelain_flags` computed
`sub_head_tree = None` when the commit object was unreadable and treated that as
clean.

Fix (grit-lib/src/diff.rs):
- Added `submodule_head_object_broken(sub_dir) -> bool`: true when the submodule
  has an embedded git dir and a resolvable HEAD OID, but that commit object is
  missing/unreadable from the submodule's ODB (mirrors git's broken-repo guard).
- In `diff_index_to_worktree_with_options`, the gitlink branch now returns a
  fatal `Error::ConfigError("'git status --porcelain=2' failed in submodule …")`
  when `submodule_head_object_broken` is true (after the `simplify_gitlinks`
  early-out, so tree-only callers are unaffected). This error propagates through
  `?` in `grit/src/commands/status.rs` (the `git status` and `git diff` paths),
  making both exit non-zero. `git fetch --recurse-submodules` already failed.

Verified manually (/tmp/t38repro): healthy old-layout status/diff/fetch still
return 0; after breaking the submodule, status/diff/fetch all return non-zero.

Result: 47/56 -> 48/56. Subtest 38 now passes. No regressions.

## Remaining 8 failures: confirmed TEST-PORTING BUG, not grit (out of my edit scope)

Failures 32,33,34,35,36,39,55,56 are the same `_wrap_cd_subshell.py`
over-wrapping artifact every prior agent found. Tests 31 (tests/ L594) and 54
(tests/ L1287) — plus 55 (L1302) and 56 (L1318) — wrap their ENTIRE body in an
extra outer `( … )` that upstream `git/t/t5526-fetch-submodules.sh` does NOT
have (upstream L558 for 31, L1225/L1238/L1252 for 54/55/56). Their top-level
`test_when_finished` cleanups (`rm expect.err.sub2`, `rm -fr src_clone`) register
inside the subshell and are lost on subshell exit:
- test 31's lost `rm expect.err.sub2` -> stale file poisons `verify_fetch_result`
  for 32-36 (3 spurious submodule2 lines on the EXPECTED side); 36 also fails
  before its `.gitmodules` restore, cascading to 39.
- tests 54/55/56's lost `rm -fr src_clone` -> next `git clone … src_clone` fails
  "destination path already exists" -> 55/56 fail.

PROOF this is a porting bug and there is NO residual grit bug: I produced an
unwrapped copy of the file (removing only the spurious outer `( )` on tests
31/54/55/56, matching upstream byte-for-byte in structure) and ran it through the
real harness with the current grit binary: **passed all 56 test(s)**. So once the
spurious wrappers are removed the whole file is green with my #38 fix in place.

I am restricted to flipping `test_expect_failure -> test_expect_success` only and
must not otherwise edit test files, so I cannot apply the un-wrap. MAINTAINER
ACTION: remove the spurious outer `( )` wrapper on t5526 tests 31, 54, 55, 56 in
tests/t5526-fetch-submodules.sh (restore upstream's top-level
`test_when_finished`). With that + the #38 fix committed here, the file is 56/56
(verified). Leaving ticket OPEN at 48/56.
