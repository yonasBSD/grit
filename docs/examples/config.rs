use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let config = repo.config()?;

    let name = config.get_string("user.name")?;
    let rebase = config.get_bool("pull.rebase").unwrap_or(false);
    println!("{name} prefers rebase: {rebase}");
    Ok(())
}
