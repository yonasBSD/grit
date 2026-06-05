use grit_lib::objects::{ObjectKind, ObjectWriter};

fn main() -> anyhow::Result<()> {
    let mut writer = ObjectWriter::new(ObjectKind::Blob);
    writer.write_all(b"content to hash
")?;

    let oid = writer.finish()?;
    println!("{oid}");
    Ok(())
}
