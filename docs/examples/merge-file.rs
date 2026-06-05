use grit_lib::merge_file::merge_file;

fn main() -> anyhow::Result<()> {
    let result = merge_file("base
", "ours
", "theirs
")?;

    println!("{}", result.into_text());
    Ok(())
}
