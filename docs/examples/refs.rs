use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;

    for reference in repo.refs().list()? {
        println!("{} -> {}", reference.name(), reference.target());
    }
    Ok(())
}
