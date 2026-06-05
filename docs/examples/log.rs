use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;

    for commit in repo.log("HEAD")?.take(10) {
        println!("{} {}", commit.id().short(12), commit.summary());
    }
    Ok(())
}
