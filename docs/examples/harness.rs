use std::process::Command;

fn main() -> anyhow::Result<()> {
    let output = Command::new("./scripts/run-tests.sh")
        .arg("t0000-basic.sh")
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
