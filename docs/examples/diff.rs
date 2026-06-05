use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let diff = repo.diff_worktree_to_index()?;

    for file in diff.files() {
        println!("{}", file.path().display());
    }
    Ok(())
}
