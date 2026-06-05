use grit_lib::{PushOptions, Repository};

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let report = repo.push("origin", ["main"], PushOptions::default())?;

    println!("pushed {} refs", report.updated_refs().len());
    Ok(())
}
