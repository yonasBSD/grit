# t2032-checkout-index-parallel.sh — investigation (ticket 337079)

Date: 2026-06-08
Result: 25/28. BLOCKED — the 3 failing subtests assert behavior that contradicts
both upstream Git's C implementation AND a sibling grit test (t8350) that is
currently fully passing. Cannot be fixed in grit code without regressing t8350.

## Failing subtests
- 5:  `checkout-index file_1.txt` (modified file in the way, no --force) must exit 0 and not overwrite.
- 12: `checkout-index -q file_99.txt` (file exists, no --force) must exit 0 with empty stderr.
- 18: `checkout-index untracked.txt` (file in the way, no --force) must exit 0, content="blocker".

All three require: "existing file in the way, without --force" => SKIP with exit 0 (success).

## Root cause: contradictory tests (not a grit bug)

Real upstream Git returns EXIT 1 (error) when a changed/existing file is in the
way without --force. Proven directly with git 2.52.0:

    # exact replay of t2032 setup through subtest 5, real /usr/bin/git:
    echo "modified" >file_1.txt && git checkout-index file_1.txt
    -> "file_1.txt already exists, no checkout"  EXIT=1

C source confirms (git/entry.c, checkout_entry_ca):
    if (!changed) return 0;
    if (!state->force) {
        if (!state->quiet)
            fprintf(stderr, "%s already exists, no checkout\n", path.buf);
        return -1;          // <-- ERROR, propagates to exit 1
    }

builtin/checkout-index.c accumulates errs and exits non-zero.

So t2032 subtests 5/12/18 (and t2030 subtest 8, same pattern) would FAIL on real
upstream git too — they assert exit 0 where git returns exit 1.

## Why it can't be fixed in grit

A sibling grit-authored test, t8350-checkout-index-force.sh, is currently fully
passing (30/30) and asserts the OPPOSITE for the identical operation:

    test_expect_success 'checkout-index without --force refuses existing dirty file' '
        echo dirty >a.txt &&
        test_must_fail grit checkout-index a.txt 2>err &&   # requires EXIT != 0
        test "$(cat a.txt)" = "dirty" &&
        grep -i "already exists" err                         # requires the message
    '

The two scenarios are byte-for-byte the same operation (committed file, content
changed on disk, `checkout-index <file>` with no --force). There is no flag,
mode (--all vs named), stat, or config discriminator between them — verified by
replaying both through both /usr/bin/git and grit; grit and git both return
EXIT 1 in both. Therefore:

- Making grit skip-with-success (exit 0) to satisfy t2032/5,12,18 and t2030/8
  would turn t8350's `test_must_fail` into a pass-of-the-command => t8350 drops
  to 29/30 (regression), and also diverges from documented C behavior.
- Keeping the current error (exit 1) keeps t8350 green but leaves t2032/5,12,18 red.

grit's CONTENT behavior is already correct in all cases (it never overwrites
without --force; -q already suppresses the stderr message — T12 stderr is empty).
Only the exit code is contested, and it cannot be both 0 and non-zero.

## Note on the docs ambiguity
git-checkout-index.adoc says the default "does not overwrite existing files" and
-q is "be quiet if files exist", which reads like a soft skip; but the actual C
code treats it as a hard error (exit 1). t2032/t2030 followed the doc reading;
t8350 followed the code. The code is ground truth, and t8350 already encodes it.

## Decision
Documented proof; ticket marked blocked. No grit code change made (any change
that satisfies t2032 regresses the passing t8350). Not modifying tests per the
One Rule.
