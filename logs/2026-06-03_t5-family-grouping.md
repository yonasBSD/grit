# t5 family — dependency grouping and progress

## Dependency groups (work in order)

1. **Archive (t500x)** — `archive` command, attributes, zip/tar output
   - Non-green: `t5001-archive-attr` (23), `t5003-archive-zip` (4)
   - Note: `t5000-tar-tree` times out (0 tests)

2. **Pack/unpack foundation (t530x–t535x)** — pack-objects, unpack-objects, index, prune, bitmaps
   - Largest blockers: `t5310-pack-bitmaps` (97), `t5319-multi-pack-index` (53), `t5302-pack-index` (24)
   - Quick wins completed: `t5300-unpack-objects` (23/23)

3. **Send/receive pack (t540x)** — push/receive plumbing
   - Non-green: `t5400-send-pack` (6), `t5407-post-rewrite-hook` (6), `t5410-receive-pack` (2)

4. **Remote/fetch/push/pull (t550x–t557x)** — largest overall block
   - Largest: `t5516-fetch-push` (118), `t5505-remote` (65), `t5515-fetch-merge-logic` (64), `t5520-pull` (58)

5. **Clone (t560x)** — depends on fetch/transport
   - Largest: `t5601-clone` (49), `t5603-clone-dirname` (22)

6. **Protocol (t570x–t581x)** — upload-pack, protocol v2, proto-disable
   - Non-green: `t5703-upload-pack-ref-in-want` (19), `t5710-promisor-remote-capability` (13)

7. **Misc** — `t5150-request-pull` (8), `t5200-update-server-info` (2), `t5900-repo-selection` (4)

## Session log

### 2026-06-03 — t5300-unpack-objects (23/23)

**Root cause:** `Odb::write_local` skipped writing the well-known empty tree because
`exists_local` treats it as virtually present without a loose file.

**Fix:** Use `exists_materialized_in_objects_dir` in `write_local` / `write_raw_local` so
unpack-objects materializes the empty tree like Git.

### 2026-06-03 — t5302-show-index (investigation)

**Symptom:** `verify-pack -v` grep yields only the pack-id line (`baba8b01…`) on the first
call after `git repack` under the harness, while `grit show-index` lists all idx OIDs.

**Likely cause:** With the harness `PATH`, real `git repack` delegates to `grit pack-objects`,
which writes packs/rev sidecars that make the first `verify-pack` traversal use rev-index
summary mode. Follow-up investigation targets `grit pack-objects` / `.rev` writing.
