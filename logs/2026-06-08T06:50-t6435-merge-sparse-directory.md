# t6435-merge-sparse-directory — test-body portability fix

Date: 2026-06-08T06:50Z
Ticket: dc809f95-fa93-4af9-a39d-7d620b77d29d

## Symptom
At last scan: 1/2 subtests passing. Subtest 2 ("merge brings in both directories")
failed with:

    ./test-lib.sh: line 1417: cd: merge-dirs: No such file or directory

## Root cause (TEST bug — cwd-persistence trap)
The setup block (subtest 1) ran `git init merge-dirs && cd merge-dirs && ...`
with a bare `cd merge-dirs`. That `cd` leaked into the harness shell, leaving the
working directory inside `merge-dirs/`. When subtest 2 then ran its own bare
`cd merge-dirs`, there was no `merge-dirs` subdirectory under the (already-entered)
`merge-dirs/`, so the `cd` failed and the subtest aborted.

This is the documented TESTING.md "Harness pitfall" cwd-persistence trap.

## Differential verification (vs /opt/homebrew/bin/git 2.52.0)
Reproduced the full setup + `git merge sideB` scenario in /tmp with BOTH grit
(target/release/grit) and real git 2.52.0 on identical inputs. Both:
- merged via the 'ort' strategy
- produced identical diffstat ("dirB/file | 1 +", "1 file changed, 1 insertion(+)",
  "create mode 100644 dirB/file")
- exited 0
- left both dirA/file and dirB/file present

Grit's merge behavior matches real git byte-for-byte for the asserted behavior
(the test only asserts test_path_is_file, not merge stdout). So the only defect
was the test-authoring cwd leak — a sanctioned test-body-only fix.

## Fix
Wrapped both subtest bodies' `cd merge-dirs && ...` sequences in subshells
`( cd merge-dirs && ... )` so the working directory no longer leaks between blocks.
No expected values changed; pure mechanism/portability fix.

## Result
Full-file run: t6435-merge-sparse-directory (2/2) — fully passing.
TOML: passed_last = 2, failing = 0, fully_passing = true.
