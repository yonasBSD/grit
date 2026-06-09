# t5814-proto-disable-ext

Ticket: e89ecb — tests/t5814-proto-disable-ext.sh ("test disabling of remote-helper paths in clone/fetch")

## Result

27/27 passing (was 19/27).

## Root causes / fixes

The 8 failing subtests were all "fetch/push remote-helper (enabled)" cases over the `ext::`
transport. Three distinct bugs:

1. **Push over `ext::` was unimplemented.** `push_to_url` in `grit/src/commands/push.rs` bailed with
   `error: ext transport is not supported for push`. Implemented it by:
   - Adding `spawn_ext_receive_pack(ext_url)` to `grit/src/ext_transport.rs`, which spawns the user's
     ext helper with the `git-receive-pack` service (or rewrites a `grit upload-pack <dir>` fast-path
     to `grit receive-pack <dir>`).
   - Refactoring the post-spawn body of `push_to_ssh_url` into a shared
     `push_over_receive_pack_child(child, transport, ...)` that drives the protocol-v1 receive-pack
     advertisement + send-pack stream over a spawned child's stdio. Both SSH and ext push now use it.

2. **`%S` placeholder in the `ext::` URL was hard-coded to `git-upload-pack`.** `parse_remote_ext_url`
   always expanded `%s`/`%S` against `git-upload-pack`, so push invoked the helper with
   `git-upload-pack` instead of `git-receive-pack` (the far end hung up). Added a `service` parameter
   to `parse_remote_ext_url` and threaded the correct service through all four callers.

3. **`-c protocol.<name>.allow=...` was not honored.** `grit/src/protocol.rs::read_config_value`
   parsed `GIT_CONFIG_PARAMETERS` with a naive `split('\'')`, which cannot decode Git's
   `'protocol.ext.allow'='always'` quoting (key and value each single-quoted). Replaced the
   hand-rolled parser with `grit_lib::config::git_config_parameters_last_value`. This fixed the
   `protocol.ext.allow=always` and `protocol.ext.allow=user` config blocks (subtests 10-20), which
   had cascaded failures (a failed `clone --bare ... tmp.git` broke the following fetch/push).

## Notes

- The test's fake-remote helper invokes the system's real `git-upload-pack` / `git-receive-pack`
  (present on this machine), which is exactly how upstream's t5814 is designed to work. grit only
  spawns the user's helper command; it does not shell out to git itself.
- Subtests 5 and 24 only pass when `init.defaultBranch=main` (set by run-tests.sh via
  `GIT_TEST_DEFAULT_INITIAL_BRANCH_NAME=main`); running the file bare uses `master` and the bare
  clone's HEAD is dangling. The official harness run reports 27/27.

## Regression checks

- t5810-proto-disable-local: 54/54
- t5812-proto-disable-http: 29/29
- t5813-proto-disable-ssh: 81/81 (exercises the refactored SSH push path)
- grit-lib unit tests: only pre-existing failures in `ignore.rs` gitignore-glob tests (unrelated;
  ignore.rs untouched).
- t5516-fetch-push fails at setup with `mkdir: testrepo/.git/hooks: File exists` (a `mk_empty`
  helper issue), pre-existing and unrelated to these changes.
