use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;

    for commit in repo.rev_walk(["HEAD"])? {
        println!("{} {}", commit.id(), commit.summary());
    }
    Ok(())
}
