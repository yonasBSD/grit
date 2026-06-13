# grit-simple

`grit-simple` provides `gs`, a small opinionated command line interface backed by [`grit-lib`](https://crates.io/crates/grit-lib).

It is not intended to be a drop-in replacement for Git. For Git-compatible command behavior, use the `grit` binary from the `grit-cli` crate. `gs` is a simpler interface for workflows built on Grit's Rust implementation.

## Install

```sh
cargo install grit-simple
```

This installs the `gs` executable.

## Commands

`gs` favors one obvious way to do the common thing, plain-language output, and a
status screen that doubles as the home base.

### `gs` / `gs status`

Running `gs` with no arguments shows the dashboard: the current branch, a
shortlog of the commits you're ahead of the target branch by, your staged and
unstaged changes, untracked files, and a hint for what to do next.

```sh
gs
# explicit form / alias:
gs status
gs st
```

```text
On feature/example  ·  2 ahead of origin/main

  abc1234  Add example
  fed9876  Refine output

Staged
  +  new-file.txt                      new
  ~  src/main.rs                       modified

Changed (not staged)
  ~  README.md                         modified

Untracked
  ?  notes.md

→ gs add <file> to stage  ·  gs commit "message" to commit
```

### `gs add`

Stage changes. With no paths, stages **everything** that `gs status` reports as
changed — modifications, deletions, and untracked files alike. Pass paths to
stage a subset.

```sh
gs add            # stage all changes
gs add src/ a.txt # stage only these paths
```

### `gs commit`

Record the staged changes as a new commit. The message can be a positional
argument or `-m`; `-a` stages every change first.

```sh
gs commit "what changed"
gs commit -m "what changed"
gs commit -a "stage everything, then commit"
```

Author/committer identity comes from `user.name` / `user.email` (honoring the
`GIT_AUTHOR_DATE` / `GIT_COMMITTER_DATE` overrides).

### `gs branch`

List branches, or create / delete one.

```sh
gs branch            # list (current marked with *)
gs branch feature    # create "feature" at HEAD (does not switch)
gs branch -d feature # delete "feature"
```

### `gs switch`

Move to another branch, updating the working tree. Refuses to switch with
uncommitted changes, and won't clobber an untracked file the destination needs.

```sh
gs switch main          # switch to an existing branch
gs switch -c feature    # create "feature" and switch to it
gs checkout main        # aliases: checkout, co
```

### `gs merge`

Merge another branch into the current one — fast-forwarding when possible,
otherwise recording a merge commit. Conflicts are reported without leaving a
half-finished merge (resolving them is out of scope for `gs`).

```sh
gs merge feature
```

### `gs fetch` / `gs pull` / `gs push`

Talk to remotes. `gs` keeps these argument-free: they default to `origin` and
the current branch's same-named counterpart — no `-u origin <branch>` ceremony.

```sh
gs fetch          # download refs/objects from origin (or: gs fetch <remote>)
gs pull           # fetch, then fast-forward / merge the upstream in
gs push           # publish the current branch to origin
```

Local (`file://` / path), `git://`, `ssh`, and `http(s)` remotes are supported.

### `gs shortlog`

Show the current branch, the target branch, and commits that are reachable from
`HEAD` but not from the target branch.

```sh
gs shortlog
# alias:
gs sl
```

Target branch lookup uses the first available value from:

1. `target.branch` in Git config
2. `origin/master`
3. `origin/main`
4. `master`
5. `main`

Example:

```text
On feature/example
Ahead of origin/main by 2 commits
abc1234 Add example
fed9876 Refine output
```
