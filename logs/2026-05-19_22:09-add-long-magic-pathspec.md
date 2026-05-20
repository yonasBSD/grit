# git add long magic pathspecs

- Claimed `t3703-add-magic-pathspec.sh` as an independent add/pathspec fix.
- Reproduced the remaining failure: `git add -n ":(icase)ha"` treated the magic pathspec as a literal file named `:(icase)ha`.
- Changed `git add` to route long magic pathspecs through pathspec matching, while keeping literal paths like `./:(icase)ha` on the normal direct path.
