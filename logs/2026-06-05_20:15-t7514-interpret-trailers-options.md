# t7514 interpret-trailers options

Ticket: 3ba5b5

Initial state: `./scripts/run-tests.sh t7514-interpret-trailers-options.sh`
reported 1/10 passing. A direct reproduction showed
`git interpret-trailers --where=end ...` failing with
`error: unknown option '--where=end'`.

Finding: the manual CLI parser handled only space-separated option values
(`--where end`, `--if-exists replace`, etc.), while Git accepts both that
form and `--option=value`. The test file uses the equals form.

Implemented:
- Added equals-form parsing for `--where=`, `--if-exists=`,
  `--if-missing=`, and `--trailer=`.
- Preserved existing stateful behavior where those settings apply to
  following `--trailer` arguments.

Result: `./scripts/run-tests.sh t7514-interpret-trailers-options.sh`
now reports 10/10 passing.
