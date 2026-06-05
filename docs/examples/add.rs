use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let mut index = repo.index().read()?;

    index.add_path("src/main.rs")?;
    index.write()?;
    Ok(())
}
