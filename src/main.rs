use anyhow::Result;
use twm::cli;

fn main() -> Result<()> {
    cli::parse()?;
    Ok(())
}
