# t5515-fetch-merge-logic (ticket 08cdf4) — Mop-up round 1

## Starting state

Prior agent (rounds 1-4, commits e73d7a547..1f9816cb8) reported 65/65 FULLY PASSING and
closed ticket 08cdf4. But a fresh run showed **1/65 passing** — a regression introduced
after round 4 by changes elsewhere in the tree.

## Root cause of the regression

The test forces `GIT_TEST_PROTOCOL_VERSION=0`. Under protocol v0/v1, `git upload-pack`
advertises each annotated tag's peeled commit as an extra ref-advertisement line:

    <tag-oid>     refs/tags/tag-main
    <peeled-oid>  refs/tags/tag-main^{}

`read_advertisement` in `grit/src/fetch_transport.rs` only skipped the `capabilities^{}`
no-refs carrier line — it recorded the `refs/tags/<name>^{}` peeled lines as if they were
real refs. The fetch then:
  - wrote bogus on-disk refs `refs/tags/tag-main^{}`, `tag-three^{}`, `tag-three-file^{}`,
    `tag-two^{}`, and
  - emitted spurious `tag 'tag-main^{}' of ../` FETCH_HEAD lines,
breaking every FETCH_HEAD / show-ref comparison (tests 2-65). The peeled refs persisted
across the test's `for-each-ref refs/tags | update-ref -d` reset loop because
`for-each-ref` does not list `^{}` names, so they were never deleted.

Manual fetches with default (v2) protocol were clean — which is why the bug only surfaced
under the harness, which sets v0.

## Fix

`grit/src/fetch_transport.rs`, `read_advertisement`: after the `capabilities^{}` guard,
skip any advertised refname ending in `^{}` (a peeled-object advertisement, not a ref).

## Verification

- `cargo build --release -p grit-cli -j 4` — clean (no new warnings in fetch_transport.rs).
- `./scripts/run-tests.sh t5515-fetch-merge-logic.sh` — **65/65 FULLY PASSING**.
- No regressions: t5503-tagfollow 12/12, t5510-fetch 215/215, t5514-fetch-multiple 25/25.
  t5601-clone 111/115, t5516-fetch-push 95/124, t5505-remote 122/130 — unchanged vs.
  committed baseline (those files have their own open tickets; not regressed by this change).
- `cargo test -p grit-lib --lib` — only pre-existing `ignore::gitignore_glob_tests` x2 fail
  (unrelated to fetch).

## Commit

`2e217796d` on branch `grit-t5-progress`:
  fix: skip peeled ^{} refs in v0/v1 fetch advertisement (t5515-fetch-merge-logic)

The t5515 TOML (passed_last=65, fully_passing=true) was already captured into the committed
tree by the workspace, so only the source fix + this log needed committing.
