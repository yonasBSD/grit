# t5526-fetch-submodules — MOP-UP ROUND 2 (session 2026-06-06, agent 3)

Ticket: 1d762d. Re-ran fresh: **41/56** (unchanged from prior recorded 41/56).
Build was current; no other agent's fix cascaded into this file this round.

## Failing subtests (15): 32,33,34,35,36, 38,39,40,41,42,43,44,45, 55,56

## Independent re-verification of prior root-cause (CONFIRMED, no new grit fix safe)

I reproduced and re-confirmed the prior agent's two root causes. All 15 failures fall
into exactly two buckets; 14 of 15 are a **test-file porting bug** I am forbidden to fix,
and the 1 genuine grit bug is a high-risk single-test change deliberately deferred.

### Bucket A — test-porting bug: spurious subshell wrapping breaks `test_when_finished` (14 tests)

`scripts/_wrap_cd_subshell.py` wrapped the bodies of **test 31** (tests/ line 594-630) and
**test 54** (line 1225-) in a `( … )` subshell. Upstream `git/t/t5526-fetch-submodules.sh`
does NOT (test 31 = line 558-592, test 54 = line 1225-). These bodies do not start with a
bare top-level `cd` (their `cd`s are already in their own inner subshells), so the wrapper
should never have touched them — it is an over-wrap porting artifact.

`test_when_finished` registers cleanup in the *current shell*; inside the spurious `( )`
subshell the registration is lost when the subshell exits, so the cleanup never runs:

- **Test 31** body has `test_when_finished "rm expect.err.sub2"` (tests/ line 596). Lost →
  stale `expect.err.sub2` survives. `verify_fetch_result()` (tests/ line 106) concatenates
  `expect.err.sub2` into `expect.err.combined` whenever the file exists, so tests **32-36**
  get 3 spurious trailing `submodule2` lines in the EXPECTED side. Verified the exact diff
  for test 32 in the verbose log:

  ```
  --- expect.err.combined
  +++ actual.err.cmp
  @@ -1,5 +1,2 @@
   From .../trash.../.
      OLD_HEAD..d8b85b0  super      -> origin/super
  -Fetching submodule submodule2 at commit 7d0b177
  -From .../submodule2
  -   OLD_HEAD..1cc1a79  sub2       -> origin/sub2
  ```

  grit's ACTUAL output (2 lines, super only — NO submodule2 fetch) is CORRECT for
  `--recurse-submodules=on-demand` when no new submodule2 commit is referenced. The 3 extra
  lines are purely the stale expected file. NOT a grit bug.

- **Tests 39-45 cascade from test 36.** Test 36 runs `verify_fetch_result` (fails on the
  stale file) BEFORE its `.gitmodules` restore (`git checkout HEAD^ -- .gitmodules && add &&
  commit`, tests/ lines 715-717). When verify fails, the restore never runs, leaving the
  superproject `.gitmodules` empty/broken. Confirmed end-state: `grit show HEAD:.gitmodules`
  in the trash super is EMPTY. Test 39 does `git clone . downstream_rename` then
  `git submodule update --init` → "No url found for submodule path 'submodule' in
  .gitmodules" (the cloned empty .gitmodules). Tests 40-45 then fail downstream of the same
  corrupted super (`.gitmodules` "would be overwritten by merge" etc.). NOT independent grit
  bugs.

- **Test 54** body has `test_when_finished "rm -fr src_clone"` (tests/ line 1226). Lost →
  `src_clone` dir survives. Tests **55/56** then `git clone --recurse-submodules src
  src_clone` → "destination path 'src_clone' already exists and is not an empty directory".
  Test 54 itself PASSES. Verified message in verbose log. NOT a grit bug.
  (Aside: test 54's log shows grit's `clone --recurse-submodules` double-clones the
  submodule — "Cloning into '.../src_clone/sub'" twice — but that does not fail 54.)

**I cannot fix bucket A:** the hard rule allows only `test_expect_failure -> success` flips,
not removing the spurious subshell wrappers. ACTION FOR MAINTAINER: un-wrap the bodies of
t5526 tests 31 and 54 (remove the outer `(`/`)` added by _wrap_cd_subshell.py) so their
top-level `test_when_finished` cleanups run. That single test-file fix should recover
12-14 of the 15 (32-36, 39-45, 55-56).

### Bucket B — genuine grit bug, deferred as high-risk (1 test: 38)

Test 38 "fetching submodule into a broken repository": after `rm -r dst/sub/.git/objects`,
`git -C dst status` / `diff` / `fetch --recurse-submodules` must each FAIL. grit's `status`
returns 0. Reproduced standalone in /tmp.

Root cause: `grit-lib/src/diff.rs::submodule_porcelain_flags` (line 4747). With objects
removed, `read_submodule_head_oid` still returns Some (HEAD ref readable), but the HEAD
*commit object* is unreadable, so `sub_head_tree` is None and staged/unstaged diff default
to false → submodule looks clean → status exits 0. Real git errors because it can't read the
submodule HEAD commit.

`submodule_porcelain_flags` returns flags (not Result) and sits on status's hot path (also
diff/fetch). Making it fatal on this corruption would have to thread an error through
status/diff/fetch — the most-used status path, risking regressions across t2/t7/etc. for a
single test. Prior agent reached the same conclusion. Not changed this session.

## Net
No grit Rust change is safe/justified: 14/15 failures are an out-of-scope test-porting bug;
the lone genuine bug (38) is a high-risk hot-path change for 1 test. Leaving open at 41/56.
