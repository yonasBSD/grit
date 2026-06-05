use grit_lib::{CommitOptions, Repository};

fn main() -> anyhow::Result<()> {
    let repo = Repository::open(".")?;
    let commit_id = repo.commit(CommitOptions {
        message: "commit from a Rust caller".into(),
        all: false,
        amend: false,
        signoff: false,
    })?;

    println!("created {commit_id}");
    Ok(())
}
