# Fix: restore "error: " prefix on apply corrupt-patch error — 2026-06-08

The Phase-5 apply extraction (grit_lib::apply, commit c98c70383) converted the
corrupt-patch ExplicitExit{128} to Error::Message but dropped the leading
"error: " that git's error() emits (git/apply.c:1922 `error("corrupt patch at %s:%d")`).
main.rs renders Error::Message verbatim, so `grit apply` printed
`corrupt patch at <file>:<line>` instead of `error: corrupt patch at <file>:<line>`,
breaking t4012-diff-binary subtests 7-8 ("apply detecting corrupt patch correctly",
which match `error.*<file>:<lineno>$`).

Fix: prefix the message with "error: " in grit-lib/src/apply.rs (and its unit test).
This was the ONLY real regression from the entire libification — verified by a full
pre-libify-binary vs libified-binary comparison: the other 31 "regressed" files fail
identically on both binaries (pre-existing drift that was masked by stale TOMLs).

Verified: t4012-diff-binary 11/13 -> 13/13; cargo test -p grit-lib --lib apply:: 5/5.
