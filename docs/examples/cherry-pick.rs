use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let picks = repo.detect_cherry_picks("main", "topic")?;

    for pick in picks {
        println!("{} matches {}", pick.left(), pick.right());
    }
    Ok(())
}
