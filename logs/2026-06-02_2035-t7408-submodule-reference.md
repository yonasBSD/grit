# t7408-submodule-reference

## 2026-06-02 20:35

- Claimed `t7408-submodule-reference.sh` after `t7422-submodule-output.sh` reached 18/18.
- Starting baseline from `data/test-files.csv`: 8/16 passing, 8 failing.

## 2026-06-02 20:50

- Fixed local clone `--reference` behavior so explicit references are the only alternates and
  referenced objects are not copied when the reference can satisfy the clone.
- Added `submodule update --dissociate` parsing/propagation and prevented alternates from being
  rewritten after dissociation.
- Shared superproject-derived submodule alternate discovery between recursive clone and submodule
  update, including `die` vs `info` missing-alternate strategy handling.
- Made submodules cloned with derived alternates inherit `submodule.alternateLocation=superproject`
  so nested recursive updates can derive their own alternates.
- Preserved partial recursive clone worktrees on submodule clone failure and matched Git's missing
  alternate retry diagnostics for recursive submodule updates.
- Validation so far: direct `cd tests && sh t7408-submodule-reference.sh -v` passed 16/16, and
  `./scripts/run-tests.sh t7408-submodule-reference.sh` passed 16/16 and refreshed
  `data/test-files.csv` plus dashboards.
