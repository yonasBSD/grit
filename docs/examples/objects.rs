use grit_lib::Repository;

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let oid = repo.objects().write_blob(b"hello from grit
")?;

    let object = repo.objects().read(&oid)?;
    println!("{} {} bytes", oid, object.data().len());
    Ok(())
}
