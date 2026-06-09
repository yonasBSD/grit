# t10320-update-ref-verify-old

Ticket: 9b956d

## Goal
Make `tests/t10320-update-ref-verify-old.sh` fully pass. Previously 28/31 with 3
failing subtests around `-z` NUL-delimited `update-ref --stdin` parsing:
- not ok 26 - stdin -z: create with NUL-separated commands
- not ok 27 - stdin -z: verify with NUL-separated commands
- not ok 28 - stdin -z: delete with NUL-separated commands

The ticket noted the same root cause as t10020: `grit update-ref -z --stdin`
not parsing the NUL-terminated (`-z`) field format.

## Findings
On claiming the ticket and rebuilding the release binary
(`cargo build --release -p grit-cli -j 4`), a fresh run of the test file passed
all 31 subtests:

    ./scripts/run-tests.sh t10320-update-ref-verify-old.sh  =>  31/31
    tests/t10320-update-ref-verify-old.sh directly           =>  passed all 31

The shared `-z` NUL-delimited stdin parsing fix (made for the t10020 work in
the same `update-ref-refs` subsystem group) already resolves subtests 26-28.
No further code change was required for this file.

## Result
31/31 passing, fully_passing = true. Staged the status TOML and this log;
committed on grit-t5-progress.
