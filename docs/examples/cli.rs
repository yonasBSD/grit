use grit_cli::run;

fn main() -> anyhow::Result<()> {
    // The binary parser maps Git-compatible argv into library calls.
    run(["grit", "status", "--short"])?;
    Ok(())
}
