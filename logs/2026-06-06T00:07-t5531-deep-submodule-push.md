# t5531-deep-submodule-push — submodule push recursion fixes

Ticket: 7d22c7
Result: 29/29 passing (was 21/29).

## Failing subtests addressed
9, 10, 12, 13 (recurse-submodules cmdline vs config precedence),
25, 26 (push.recurseSubmodules=only on super / super+sub),
27, 28, 29 (propagating remote name + refspec to a submodule).

## Root causes and fixes

### 1. `--no-recurse-submodules` did not override config (tests 9, 10, 13)
`effective_push_recurse_submodules` (grit/src/commands/push.rs) applied
`args.no_recurse_submodules` BEFORE reading config, so a later
`push.recurseSubmodules=on-demand` config value re-enabled recursion.
Fix: read config as the baseline first, then apply command-line tokens
(last-wins), then `--no-recurse-submodules` last so the command line always
overrides config. Matches Git argument-vs-config precedence.

### 2. submodule push skipped when superproject ref already up to date (test 9 step 4)
`collect_changed_gitlinks_for_push` (grit-lib/src/push_submodules.rs) fell
back to the *destination repo's* `refs/heads/*` as the negative side of the
super walk when there were no `refs/remotes/<name>/*` tracking refs (URL
push). That pruned the walk by the remote's current state, so when the super
ref was already pushed (via a prior `--no-recurse-submodules`) but the
submodule commit was not, an on-demand push found "nothing changed" and never
pushed the submodule. Git uses `--not --remotes=<name>` (the SUPERPROJECT's
tracking refs only) and lets the per-submodule check
(`submodule_needs_push_to_remote`) exclude already-pushed submodule commits.
Fix: removed the destination-heads fallback; the walk now covers full history
on URL pushes, matching `find_unpushed_submodules`.

### 3. `--recurse-submodules=only-is-on-demand` treated as a forced mode (tests 25, 26)
That token is NOT a recurse value. Per Git's `option_parse_recurse_submodules`
it leaves the current (config-derived) mode untouched UNLESS the mode is
`only`, in which case it becomes `on-demand` (with a warning). grit instead
forced `on-demand` unconditionally (both via the
`GRIT_PUSH_RECURSE_ONLY_IS_ON_DEMAND` env var and via parsing the token), so a
nested submodule with NO recurse config wrongly recursed into its own
submodule. Fix: in `effective_push_recurse_submodules`, treat the token / env
var as a conditional `only -> on-demand` transform applied after config + other
tokens, never forcing on-demand.

### 4. nested submodule push never received `only-is-on-demand` (test 26)
`nested_only` was derived from the env var, so it was only true when the
PARENT push was itself a nested-only push. Git's `push_submodule`
UNCONDITIONALLY adds `--recurse-submodules=only-is-on-demand` to every child
push. Fix: always set `nested_only = true` when recursing.

### 5. remote name + refspec propagation used wrong discriminator (tests 25, 27, 28, 29)
The nested-push gating used `path_style_remote` (is the URL a local path?) to
decide whether to validate and propagate the remote name + refspec to the
submodule. Git gates on `remote->origin != REMOTE_UNCONFIGURED` — i.e. whether
the superproject pushed to a CONFIGURED remote NAME vs an anonymous URL. A
local-path *configured* remote (e.g. `origin` -> `../upstream`) is path-style
but still configured, so its name+refspec must propagate. Fix: thread
`remote_is_configured_name` into `push_to_url` and use it for both the
`validate_submodule_push_refspecs` call and the `remote_specs` decision;
dropped the now-unused `path_style_remote` parameter.

## Coexistence note
push.rs is being concurrently edited by another agent (push `--porcelain`
work: new `grit-lib/src/push_report.rs`, `push_report` mod in lib.rs, and
porcelain code paths in push.rs). Their working-tree edits clobbered my push.rs
edits twice mid-session; re-applied each time. I committed only push.rs +
push_submodules.rs + the t5531 toml + this log, and deliberately did NOT stage
lib.rs or push_report.rs (their separate files).

## Regression checks (vs committed baselines)
- t5505-remote 121/121 = baseline (no regression)
- t5526-fetch-submodules 39/39 = baseline (no regression)
- t5543-atomic-push 11/13 = baseline (no regression)
- t5545-push-options 13/13 = baseline (no regression)
- t5572-pull-submodule, t5548-push-porcelain, t5516-fetch-push currently below
  their committed baselines, but caused by the OTHER agent's uncommitted
  in-flight fetch/clone/porcelain changes built into the shared binary
  (errors like "missing object ... while copying from remote" in fetch, and
  incomplete --porcelain formatting) — not by anything in this ticket. My
  changes only touch submodule-recursion push paths (`collect_changed_gitlinks_for_push`
  is push-only; pull uses `submodule_gitlinks_touched_in_range`, unchanged).
- Pre-existing unrelated unit-test failures: ignore::gitignore_glob_tests
  (gitignore globbing, not my domain).
