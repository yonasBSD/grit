# t5526-fetch-submodules — mop-up round 1 (ticket 1d762d)

## Status this session: 41/56 passing (was 39 recorded; +2 from cascaded fixes by other agents).
## Build was blocked early (resolved by another agent ~07:00); then ROOT-CAUSED all 15 remaining failures.

## DEFINITIVE ROOT CAUSE (update after build unblocked): test-file PORTING bug, not grit.
The bulk of the 15 failures are caused by the cd-subshell wrapping (`scripts/_wrap_cd_subshell.py`)
having wrapped test bodies that contain `test_when_finished`. `test_when_finished` sets the shell
var `test_cleanup` in the CURRENT shell; when the body is wrapped in `( ... )`, the registration
happens INSIDE the subshell and is LOST when the subshell exits, so the cleanup NEVER runs. Upstream
`git/t/t5526-fetch-submodules.sh` does NOT wrap these bodies. Confirmed by diffing grit test 31
(line 594, wrapped in `( )`) vs upstream test 31 (line 558, unwrapped).

Concrete cascades:
- **Test 31** (`...changed submodules and the index`) has `test_when_finished "rm expect.err.sub2"`.
  Trapped in the subshell -> `expect.err.sub2` is NEVER removed. Proven: after `--run=1-31`,
  `trash.t5526.../expect.err.sub2` still exists (183 bytes). Every later `verify_fetch_result`
  appends the stale sub2 block, so **tests 32, 33, 34, 35, 36 all fail** with the SAME spurious
  trailing diff:
    `-Fetching submodule submodule2 at commit 7d0b177`
    `-From .../submodule2`
    `-   OLD_HEAD..1cc1a79  sub2       -> origin/sub2`
  grit's ACTUAL fetch in test 32 is CORRECT — verified against real git 2.52.0 (manual repro,
  /tmp/repro5526b.sh): real git's test-32 fetch output is ONLY `From .../.  super -> origin/super`,
  it does NOT fetch submodule2, and origin/sub2 does not move. So grit is right; the comparison
  baseline is poisoned by the un-removed expect.err.sub2.
- **Test 54** (`fetch --all with --recurse-submodules`, PASSES) has
  `test_when_finished "rm -fr src_clone"` trapped in its subshell -> `src_clone` is never removed.
  **Tests 55 and 56** then do `git clone --recurse-submodules src src_clone` which fails with
  `destination path 'src_clone' already exists and is not an empty directory`. Confirmed in run.

=> Tests **32,33,34,35,36,55,56 (7 tests)** are NOT grit bugs. They need the test file to un-trap
`test_when_finished` (move it out of the wrapping subshell / unwrap those bodies, matching upstream).
I am restricted to only flipping test_expect_failure->success, and the auto-mode classifier blocks
both editing and running an edited test file, so I could not apply or even measure this fix. The
maintainer (or whoever owns the cd-subshell porting) should unwrap the bodies of tests with a
top-level `test_when_finished` in t5526 (at least tests 31 and 54; audit others).

### Remaining genuine grit gaps (the other ~8: 38,39,40-45) — many likely cascade from the above
- **38 (broken repository)**: old-fashioned submodule layout (`git -C dst clone ../src/sub sub`,
  nested independent clone, no gitlink-with-modules). Real git: after `rm -r dst/sub/.git/objects`,
  `git -C dst fetch --recurse-submodules` must FAIL/terminate. grit exits 0 and does not even
  recurse into the old-layout nested clone (`status`/`diff`/`fetch` all see it as clean). Genuine
  grit gap: recurse into old-style nested-clone submodules and propagate the broken-repo error.
  Verified via /tmp/t38.sh (real git: final fetch "fail(good)"; grit: "ok(BAD)"). Larger feature,
  not attempted (high regression risk, entangled).
- **39 (renamed submodule)**: needs `git mv submodule submodule_renamed` + push + on-demand fetch to
  map origin/rename_sub through the rename. Not isolated-reproduced; may be grit gap or cascade.
- **40-45 (on-demand outside standard refspec / FETCH_HEAD / custom remote / intermittent)**: operate
  on `downstream`, whose state is corrupted by the 32-36 cascade (those tests DID run real fetches
  that mutated downstream even though the text comparison failed). So 40-45 failures may be cascade
  artifacts; cannot cleanly separate until 32-36 pass (i.e. until the test-file porting bug is fixed)
  and the file is re-run.

### Net for this session
No grit Rust change was justified/safe: the dominant root cause is a test-file porting bug outside
my allowed edits, and the residual genuine gaps (38 old-layout recursion, 39 rename) are large,
entangled, and would risk regressing the 41 passing tests. Recorded accurate 41/56 and this
root-cause so the next agent can (a) get the test file un-trapped for test_when_finished, re-run,
then (b) tackle whatever genuinely remains (likely just 38/39 plus a clean look at 40-45).

---
## Earlier-session notes (pre build-unblock)

### Build blocker (could not compile, so could not test any Rust fix)
The GitButler workspace HEAD (`8bf29dbb2 GitButler Workspace Commit`) materializes a
**conflicted commit owned by another agent**:
`21f947bfe [conflict] fix: honor custom --receive-pack for atomic local push exit code (t5543-atomic-push)`.
That conflicted commit leaves raw `<<<<<<< / ======= / >>>>>>>` markers in 4 tracked source files:
- `grit-lib/src/unpack_objects.rs`
- `grit/src/receive_ingest.rs`
- `grit/src/commands/upload_pack.rs`
- `grit/src/commands/unpack_objects.rs`

`cargo build --release -p grit-cli -j 4` fails with `error: encountered diff marker`.
The conflict is a 3-way merge between two OTHER agents' features — promisor missing-references
(`allowed_missing`, `allow_promisor_missing_references`) vs shallow boundaries
(`shallow_boundaries`). The integrated union already exists cleanly on the real `main` branch
(`7f82886a9`): `UnpackOptions` there has all three fields. So the resolution is "union of both
sides" (matches `main`), but **resolving/overwriting these files belongs to the agent who owns
`21f947bfe`** — I did not touch them (auto-mode also blocked overwriting them, correctly).
Next agent: once that conflicted commit is resolved and the build is green, the t5526 fixes below
become testable.

### Remaining 15 failures: 32-36, 38-45, 55, 56
All chain from ONE root cause discovered via verbose run (`-v -i`): **on-demand / changed-submodule
recursion does not pick up `submodule2`**, the second submodule added on branch `super-sub2-only`
in setup subtest 30.

Subtest 32 diff (expected minus actual), grit is MISSING:
```
-Fetching submodule submodule2 at commit 7d0b177
-From .../submodule2
-   OLD_HEAD..1cc1a79  sub2       -> origin/sub2
```
grit fetched only `super` and stopped. Expected: detect that newly-fetched super commits
(across `origin/super` AND `origin/super-sub2-only`) change `submodule2`'s gitlink, then recurse
on-demand into `submodule2` with the `at commit <super_oid>` annotation.

Likely fix location: the changed-submodule detection in grit's fetch recursion (the by-OID /
"changed in newly fetched commits" pass). It currently appears to scan only the checked-out
branch's new commits or only index/work-tree submodules, so a submodule that is referenced solely
by a *different* fetched branch (`super-sub2-only`) and whose work tree was init'd in subtest 30 is
not considered "changed". Cross-reference C: `builtin/fetch.c` `add_oid_to_grow_refs` /
`find_non_local_tags` is unrelated; the relevant C is `submodule.c`
`submodule_touches_in_range` + `collect_changed_submodules` which walks ALL newly-fetched ref tips
(new..old over every updated ref), not just HEAD. grit probably restricts the rev walk to the
super branch only.

- 32: "stops when no new submodule commits are found" — actually fails because it must FIRST
  recurse into submodule2 (the `7d0b177` super-sub2-only commit changed it), then stop at deep.
- 33/34: fetch.recurseSubmodules=on-demand / submodule.<sub>.fetchRecurseSubmodules=on-demand
  config override — same submodule2 detection gap, plus per-submodule on-demand config.
- 35: "don't fetch submodule when newly recorded commits are already present" — needs the
  already-present short-circuit on the changed path.
- 36: on-demand "works also without .gitmodules entry" — changed detection must not depend on a
  .gitmodules entry (use the index/config submodule name fallback).
- 38: fetching into a broken repository — error-handling path.
- 39: submodule got renamed — name<->path mapping across the rename.
- 40-45: on-demand outside standard refspec / in FETCH_HEAD / without .gitmodules / intermittently
  referenced / custom remote name / FETCH_HEAD from custom remote — the by-OID follow-up exists
  (prior agent) but FETCH_HEAD recording and custom-remote-name plumbing for these on-demand cases
  need verifying once submodule2 detection works.
- 55: `fetch --all --recurse-submodules with multiple` — needs 2 "Fetching submodule sub" lines
  (once per remote via fetch --all); prior agent flagged clone --recurse-submodules double-clone
  leaving src_clone uncleaned.
- 56: `fetch --all --no-recurse-submodules only fetches superproject` — `--no-recurse-submodules`
  must override `submodule.recurse=true` config under `--all`.

### Recommended next step
1. Wait for / coordinate resolution of conflicted commit `21f947bfe` (not mine to resolve), rebuild.
2. Make `collect_changed_submodules` equivalent in grit walk EVERY updated ref's `old..new` range
   (not just the super branch) so `submodule2` referenced by `origin/super-sub2-only` is detected.
   This single fix should unblock 32-36 and most of 40-45.
3. Then tackle 55/56 (fetch --all clone cleanup + --no-recurse override) and 38/39 separately.

No Rust changed this session (build was unbuildable). Only the test TOML (now 41/56) and this log.
