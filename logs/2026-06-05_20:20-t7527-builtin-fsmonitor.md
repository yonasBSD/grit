# t7527 builtin fsmonitor

Ticket: b311e7

The harness log reports:

```text
1..0 # SKIP fsmonitor--daemon is not supported on this platform
```

No subtests run on this platform, so there is no Rust behavior to fix here.
Per the ticket description and AGENTS.md metadata rules, the test file is
marked `in_scope = "skip"` so it is omitted from aggregate runs and counts in
this environment.
