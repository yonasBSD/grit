use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let commit = repo.rev_parse_single("HEAD~3")?;

    println!("resolved to {commit}");
    Ok(())
}
