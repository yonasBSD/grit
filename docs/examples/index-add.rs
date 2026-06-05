use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let mut index = repo.index().read()?;

    index.add_path("README.md")?;
    index.sort_and_refresh()?;
    index.write()?;
    Ok(())
}
