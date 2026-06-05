use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let base = repo.merge_base("main", "feature")?;

    println!("merge base: {base}");
    Ok(())
}
