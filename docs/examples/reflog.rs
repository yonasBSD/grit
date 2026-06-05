use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;

    repo.reflogs().append("HEAD", "checkout: moving to main")?;
    for entry in repo.reflogs().read("HEAD")? {
        println!("{} {}", entry.new_oid(), entry.message());
    }
    Ok(())
}
