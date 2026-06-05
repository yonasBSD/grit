use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let status = repo.status()?;

    for entry in status.entries() {
        println!("{} {}", entry.kind(), entry.path().display());
    }
    Ok(())
}
