use grit_lib::{FetchOptions, Repository};

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let report = repo.fetch("origin", FetchOptions::default())?;

    println!("fetched {} refs", report.updated_refs().len());
    Ok(())
}
