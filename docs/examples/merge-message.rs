use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let message = repo.format_merge_message(["feature"])?;

    println!("{message}");
    Ok(())
}
