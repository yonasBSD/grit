# t8950-show-ref-patterns — MOP-UP ROUND 2 (ticket c3d261)

Date: 2026-06-07T10:16Z
Agent: grit-t5 (Opus 4.8 1M)

## Status
27/29 passing. Failing subtests: 19 (`show-ref --hash shows only SHAs`) and
21 (`show-ref --tags --hash lists only SHAs for tags`). No cascade change from
other agents since prior rounds.

## Independent re-verification of prior diagnosis (CONFIRMED — not a grit defect)

Reproduced subtest 19 logic by hand:
- `grit show-ref --hash refs/heads/master` emits exactly 40 hex chars + one `\n`
  (`0x0a`), verified via `xxd`.
- `diff` of grit output vs `/usr/bin/git show-ref --hash refs/heads/master` =>
  **IDENTICAL** (byte-for-byte).

Root cause is in the SYNTHETIC test body, not grit:
```
sha=$(tr -d "\n" <actual)
len=$(printf "%s" "$sha" | wc -c)   # macOS/BSD wc -c => "      40" (leading ws)
test "$len" = 40                    # QUOTED string compare: "      40" != "40" -> FAIL
```
On GNU/Linux `wc -c` outputs `40` (no padding), so the same test passes there.
Subtest 20 passes because it uses `test_cmp` (no wc). Subtests 4/27 pass because
they use the unquoted `$(... | wc -c)` form which collapses whitespace via word
splitting. The failing tests use a captured var in a quoted compare, which keeps
the leading whitespace.

Real `/usr/bin/git` would fail these same two synthetic subtests on macOS for the
identical reason — the bug is in the test, portable fix is `= 40` -> `-eq 40`
(numeric comparison tolerates leading whitespace).

## Why no grit-side fix
- grit output is already byte-for-byte correct vs git.
- grit cannot alter how the shell's `wc -c` formats its count.
- The only fix is editing the test body (`=` -> `-eq`), which is forbidden by the
  ticket/AGENTS.md "do not modify tests" rule (this is not a
  test_expect_failure->success flip).
- No upstream `git/t/t8950*` exists (fully synthetic file), so there is no
  canonical behavior to align to beyond matching git's bytes, which we already do.

Same class as siblings 3b988355b (t9190) / e39408ae8 (t8070) BSD `wc` issues.

## Decision
Leaving ticket open/blocked. No grit code change is warranted or possible.
Committing this verification log only.
