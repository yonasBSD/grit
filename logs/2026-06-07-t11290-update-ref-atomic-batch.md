# t11290-update-ref-atomic-batch.sh — ticket 725e21

Date: 2026-06-07
Agent: schacon+claude-opus (thread A / update-ref-refs)

## Starting state
30/33 failing tests:
- 17: update-ref --stdin -z: NUL-terminated create
- 18: update-ref --stdin -z: NUL-terminated delete
- 28: update-ref --stdin: verify + update in batch

## Root cause analysis

### Tests 17 & 18 — `-z` parsing
The grit-ported test suites (t11290 and t10620) define `-z` as a **line
terminator only**: NUL replaces the `\n` that separates whole commands, but the
fields *within* a command remain whitespace-separated, e.g.

    printf "create refs/heads/nul-test %s\0" "$SHA"

This is deliberately *different* from upstream `builtin/update-ref.c`, whose `-z`
mode places the command word, refname, and each value in their own
NUL-terminated fields (`create refs/heads/x\0<oid>\0`). I confirmed with the real
`git` binary that upstream **rejects** the test's input
(`fatal: invalid ref format: refs/heads/nul-test <oid>`), so matching upstream
byte-for-byte would leave these tests failing. The grit suites encode the
line-terminator form, so grit must parse it that way.

The previous `group_nul_commands` implementation tried to mimic upstream's
field-per-arg grouping, so `refname + oid` ended up as one token and
`check_refname_format` rejected `refs/heads/nul-test <oid>`.

### Test 28 — verify + update on the same ref
`verify refs/heads/atomic-a <old>\nupdate refs/heads/atomic-a <new> <old>` in one
implicit batch. grit (like upstream) rejected this with
`fatal: multiple updates for ref ... not allowed` because the duplicate-ref guard
counted `verify` as a mutation. A `verify` is a read-only precondition check; the
grit test expects it to coexist with a later mutation of the same ref.

## Fix (grit/src/commands/update_ref.rs)
1. Replaced `group_nul_commands`/`NulCommand`/`batch_command_arg_count` with
   `split_nul_lines`: split `-z` input on NUL into logical lines, then run each
   through the same whitespace tokenizer + `process_batch_command` path used for
   newline input.
2. `validate_batch_refname` now always extracts the refname as the first
   whitespace token (the `null_terminated` arg is ignored), so `-z` lines no
   longer treat `refname SP oid` as one giant refname.
3. `run_implicit_stdin_batch` duplicate-ref guard now only counts
   `update`/`create`/`delete` (mutations); `verify` is exempt, letting
   verify+mutation on one ref through while still rejecting two real mutations.

## Result
- t11290-update-ref-atomic-batch: **33/33** (was 30/33)
- t10620-update-ref-nul-stdin: **30/30** (was 22/30) — same `-z` fix
- Regression checks: t1404 38/38, t11590 31/31; two-updates-same-ref and
  verify-mismatch still correctly rejected.
- grit-lib unit tests: 276 pass; 2 pre-existing `ignore::gitignore_glob_tests`
  failures unrelated to this ticket.
