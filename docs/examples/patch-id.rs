use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let diff = repo.diff_commit("HEAD")?;
    let patch_id = repo.patch_id(&diff)?;

    println!("stable patch id: {patch_id}");
    Ok(())
}
