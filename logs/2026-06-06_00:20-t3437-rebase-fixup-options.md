# t3437-rebase-fixup-options

Ticket: de1f52
Claimed: 2026-06-06 00:20 Europe/Berlin

Goal: implement interactive rebase `fixup -C` and `fixup -c` behavior, including autosquash `amend!` handling, conflict continuation, editor invocation, author preservation, and adjacent squash/fixup message folding.

Initial run after the t3440 parser/message changes: `./scripts/run-tests.sh t3437-rebase-fixup-options.sh --timeout 180` reports 2/13. The prior ticket baseline was 1/13, so the new todo parser already fixed one subtest.

Progress:

- Added parsing and replay support for `fixup -C`, `fixup -c`, and glued `-C<rev>` / `-c<rev>` todo forms.
- Implemented fixup replacement-message folding, editor invocation for `-c`, conflict continuation handling, and author preservation for folded commits.
- Matched Git's squash-message templates for skipped fixup sections, explicit replacement sections, signoff display, and empty-resolution conflict skips.
- Preserved autosquash `amend!` commits as `fixup -C` todo entries so replacement messages are used instead of plain skipped fixups.

Results:

- `./scripts/run-tests.sh t3437-rebase-fixup-options.sh --timeout 180` progressed through 2/13, 4/13, 6/13, 7/13, 8/13, 9/13, 11/13, 12/13, then 13/13.
- Final run: `./scripts/run-tests.sh t3437-rebase-fixup-options.sh --timeout 180` reports 13/13.
