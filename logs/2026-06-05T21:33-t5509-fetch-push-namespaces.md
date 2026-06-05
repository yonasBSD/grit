# t5509-fetch-push-namespaces

Ticket 203816. Upstream test: fetch/push involving ref namespaces.

## Starting state
4/15 passing. Heavy use of `ext::git --namespace=... %s <repo>` transport,
`GIT_NAMESPACE`, and `transfer.hideRefs`.

## Root causes found
1. `grit upload-pack` (v0 advertisement in `grit/src/commands/upload_pack.rs`)
   ignored namespaces and `transfer.hideRefs` entirely. It always advertised
   `HEAD` (even when the namespace had no HEAD) and listed
   `refs/heads/`,`refs/tags/`,`refs/remotes/` with no hideRefs filtering and no
   namespace handling for the empty case.
2. `grit serve-v2` `ls-refs` (`grit/src/commands/serve_v2.rs`) did not apply
   `transfer.hideRefs` and did not strip the namespace prefix from the HEAD
   `symref-target`, so v2 clone picked the wrong HEAD under `GIT_NAMESPACE`.
3. `git push --tags` (and `--follow-tags`) over `ext::`/SSH (the
   `push_over_receive_pack_child` path in `grit/src/commands/push.rs`) never
   pushed tags at all — only the local-path push code handled `args.tags`.
4. The push client over `ext::` (`read_receive_pack_advertisement` in
   `grit/src/http_push_smart.rs`) did not emit `GIT_TRACE_PACKET` lines for the
   ref advertisement it read, so `test_grep refs/heads/foo/1 trace` aborted the
   whole test file (no `trace` file existed).

## Fixes
- Rewrote `write_ref_advertisement` to resolve namespaced HEAD, advertise only
  refs under the active namespace (logical/stripped names), apply
  `uploadpack`/`transfer.hideRefs`, advertise peeled tags, and emit
  `capabilities^{}` (not an unborn `HEAD`) for an empty namespace.
- Made `cmd_ls_refs` hideRefs-aware and strip namespace from symref-targets.
- Added `--tags`/`--follow-tags` handling to `push_over_receive_pack_child`.
- Added advertisement tracing to `read_receive_pack_advertisement`.

## Result
(to be filled in after final run)
