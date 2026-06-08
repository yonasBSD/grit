# Libify: format-patch core -> grit_lib::porcelain::format_patch — 2026-06-08

Recovered from the phase-7 workflow's last target (the agent completed the
extraction but the workflow harness failed to capture its structured output).

Extracted the format-patch assembly core (~849 lines out of grit/src/commands/format_patch.rs)
into a new grit_lib::porcelain::format_patch module; the CLI keeps clap args,
file output, and pager. Incidental: rustfmt normalized grit-lib/src/porcelain/status.rs
(pure formatting, no behavior change).

Verified byte-exact: cargo build clean; t4014-format-patch 215/215, t4021-format-patch-numbered
14/14, t7508-status 126/126; t4052-stat-output 81/89 == its pre-existing baseline
(known grit --graph --stat bug, not a regression).
