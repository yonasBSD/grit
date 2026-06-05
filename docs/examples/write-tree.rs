use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let index = repo.index().read()?;

    let tree_id = index.write_tree(repo.objects())?;
    println!("wrote tree {tree_id}");
    Ok(())
}
