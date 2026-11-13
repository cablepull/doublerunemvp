use super::Failure;
use clap::Args as ClapArgs;
use std::path::{Path, PathBuf};

#[derive(ClapArgs)]
pub struct Args {
    /// JSON action plan and independent postconditions
    #[arg(long)]
    scenario: PathBuf,
    /// Timeout for each headless renderer invocation (1..=60 seconds)
    #[arg(long, default_value_t = 20)]
    timeout_seconds: u64,
}

pub(super) fn run(ws: &Path, a: &Args) -> Result<(), Failure> {
    let report = rune_ui::pilot(ws, &a.scenario, a.timeout_seconds).map_err(Failure::Other)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
