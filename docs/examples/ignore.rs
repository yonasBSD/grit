use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let ignores = repo.ignore_matcher()?;

    println!("target ignored? {}", ignores.is_ignored("target/debug/grit"));
    Ok(())
}
