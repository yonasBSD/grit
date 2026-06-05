use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let mut index = repo.index().read()?;

    for entry in index.entries() {
        println!("{} {}", entry.mode(), entry.path().display());
    }
    index.write()?;
    Ok(())
}
