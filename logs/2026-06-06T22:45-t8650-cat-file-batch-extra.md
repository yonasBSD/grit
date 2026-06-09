# t8650-cat-file-batch-extra — work log

Ticket: 7a3ef8 — tests/t8650-cat-file-batch-extra.sh
Date: 2026-06-06T22:45Z
Agent: schacon+claude-t5

## Status

26/27 passing. The single failing subtest is:

- **4: cat-file --batch content matches cat-file -p**

## Root cause (NOT a grit bug — test portability)

Subtest 4 body:

```sh
git cat-file -p "$blob" >expected &&
echo "$blob" | git cat-file --batch >batch_out &&
tail -n +2 batch_out | head -n -1 >actual &&
test_cmp expected actual
```

The failure is `head: illegal line count -- -1`. `head -n -1` (print all but
the last N lines, GNU coreutils negative-count syntax) is **not supported by the
BSD `head`** shipped with macOS, which is what runs this harness. The pipeline
aborts before `test_cmp` ever runs.

### grit behavior is correct

grit's `cat-file --batch` output is **byte-for-byte identical** to upstream git
for this blob (verified with `xxd` and `wc -c` — both 57 bytes):

```
<oid> blob 8\n      # header
updated\n           # 8 bytes content
\n                  # batch trailing delimiter
```

`tail -n +2` then `head -n -1` is the test's way of stripping the header line and
the trailing `\n` delimiter to compare against `cat-file -p`. With GNU `head`
(`ghead -n -1`) the extracted content is exactly `updated\n`, matching
`cat-file -p`. So the test logic is sound; only the BSD `head` on the runner box
rejects the `-n -1` argument.

### Why there is no grit-side fix

- The `--batch` trailing-newline delimiter is required git-compatible behavior
  (other subtests in this very file, e.g. size/deterministic checks, depend on
  the exact 57-byte output). Dropping it to make `head -n -1` unnecessary would
  break git compatibility and other subtests.
- The break is entirely in the shell pipeline, before any comparison of grit
  output. No grit code path participates in the failure.

### Why I did not fix the test

This is a grit-authored test (no `git/t/t8650*` upstream equivalent). It is
already `test_expect_success`, so the only sanctioned test edit
(`test_expect_failure` -> `test_expect_success`) does not apply. The contract
forbids other test-file edits. The correct fix would be to make the test use a
portable construct (e.g. `sed '$d'` instead of `head -n -1`, or invoke `ghead`),
but that is out of scope for this ticket as written.

## Recommendation for mop-up / test owner

Replace `head -n -1` with a portable equivalent in subtest 4:

```sh
tail -n +2 batch_out | sed '$d' >actual
```

`sed '$d'` deletes the last line and is portable across BSD/GNU. With that change
the subtest passes (grit output already matches). Until the test is allowed to be
edited, this file is capped at 26/27 on a macOS/BSD runner.
