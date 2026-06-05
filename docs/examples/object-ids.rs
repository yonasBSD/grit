use grit_lib::ObjectId;

fn main() -> anyhow::Result<()> {
    let id = ObjectId::parse_hex("e69de29bb2d1d6434b8b29ae775ad8c2e48c5391")?;
    println!("short id: {}", id.short(12));
    Ok(())
}
