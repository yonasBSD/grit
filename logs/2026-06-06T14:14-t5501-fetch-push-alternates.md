# t5501-fetch-push-alternates

Ticket: 00dec0ee-aa6f-4d14-8a79-ec2a2b80c4ed

## Initial state
- 2/3 passing. Failing: subtest 2 "pushing into a repository with the same alternate".
- Symptom: `receiver.count` = 306, expected 3. grit pushed the full shared history instead of only the new Z commit + tree + blob.

## Diagnosis
- Test: `one` is cloned from a `file://` original with `--reference=original`, so `one`'s object store
  shares the `original/.git/objects` alternate. The `receiver` repo is freshly `git init`ed with its
  `objects/info/alternates` pointing at `original/.git/objects` too (same alternate).
- Real Git: receive-pack advertises `.have` lines for the receiver's alternate ref tips
  (`for_each_alternate_ref`). The pusher uses those as negative boundaries, so the shared 101-commit
  history is excluded and only 3 new objects are sent.
- grit's local-push fast path (`grit/src/commands/push.rs`) builds the thin pack via
  `pack_objects::build_thin_push_pack_excluding_hidden`, whose negative boundary was ONLY the
  receiver's own refs. The receiver has no refs (fresh init), so nothing was excluded -> 306 objects.
- Note: receive-pack's own advertisement already computes alternate `.have` via
  `collect_alternate_have_oids`, but the in-process local push never consumes that advertisement; it
  builds the pack directly. So the boundary had to be added on the pack-build side.

## Fix
- `grit/src/commands/pack_objects.rs`: in `build_thin_push_pack_excluding_hidden`, also add the
  receiver's alternate ref tips (new helper `remote_alternate_have_roots`, mirroring receive-pack's
  `collect_alternate_have_oids` / Git's `for_each_alternate_ref`) to the negative have-roots, keeping
  only those whose closure is present locally.
