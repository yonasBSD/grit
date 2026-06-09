# t3405-rebase-malformed (ticket 286b64)

## Outcome
Already fully passing: **5/5**.

## Investigation
The ticket described subtest 4 (`git rebase -m main empty-message-merge`) hanging /
aborting the whole test run, leaving incomplete `.git/rebase-merge` state, so the
harness recorded 0/0 ("FATAL: Unexpected exit with code 0").

On a fresh build (`cargo build --release -p grit-cli -j 4`) the full file runs green:

```
./scripts/run-tests.sh t3405-rebase-malformed.sh
  ✓ t3405-rebase-malformed (5/5)
```

### Clean repro of the previously-failing subtest
Built a minimal repo with an empty-message commit and ran `rebase -m`:

```
GIT_EDITOR=: grit rebase -m main empty-message-merge
=> Successfully rebased and updated refs/heads/empty-message-merge.
   exit=0
   no leftover .git/rebase-merge dir
```

So the merge-backend rebase now carries an empty commit message through without
invoking an editor and without aborting. This matches real git 2.52 behavior. The
root cause described in the ticket was fixed by an earlier agent on the shared
rebase / commit-message subsystem (not changed by me in this run).

## Unit tests
`cargo test -p grit-lib --lib`: 276 passed; only the 2 known pre-existing
`ignore::gitignore_glob_tests` failures remain (unrelated to this ticket).

## Files changed by me
None (Rust). Only this log file. Status TOML
(`data/tests/t3/t3405-rebase-malformed.toml`) already reflects 5/5 fully_passing.
