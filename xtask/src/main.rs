use anyhow::Result;
use clap::Parser;

mod app;
mod cli;
mod tasks;
mod util;

fn main() -> Result<()> {
    let cli = crate::cli::Cli::parse();
    crate::app::run(cli)
}
