use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let head = repo.state().resolve_head()?;

    println!("HEAD points at {}", head.target());
    Ok(())
}
