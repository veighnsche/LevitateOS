mod common;
mod vm;
mod test;
mod check;
mod gen;
mod ci;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Global options available to all commands
#[derive(Parser, Debug, Clone)]
pub struct GlobalArgs {
    /// Enable verbose output
    #[clap(long, short = 'v', global = true)]
    pub verbose: bool,

    /// Suppress non-error output
    #[clap(long, short = 'q', global = true)]
    pub quiet: bool,
}

/// Internal tooling for LevitateOS
#[derive(Parser)]
#[command(name = "xtask")]
#[command(version, about = "Internal tooling for LevitateOS")]
struct Cli {
    #[clap(flatten)]
    global: GlobalArgs,

    #[clap(subcommand)]
    command: Command,
}

/// Top-level tool categories
#[derive(Subcommand)]
enum Command {
    /// VM and QEMU management
    #[clap(subcommand)]
    Vm(vm::VmCommand),

    /// Test orchestration
    #[clap(subcommand)]
    Test(test::TestCommand),

    /// Code quality checks
    #[clap(subcommand)]
    Check(check::CheckCommand),

    /// Generate artifacts
    #[clap(subcommand)]
    Gen(gen::GenCommand),

    /// CI workflow tasks
    #[clap(subcommand)]
    Ci(ci::CiCommand),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Vm(cmd) => vm::run(&cmd),
        Command::Test(cmd) => test::run(&cmd, &cli.global),
        Command::Check(cmd) => check::run(&cmd, &cli.global),
        Command::Gen(cmd) => gen::run(&cmd, &cli.global),
        Command::Ci(cmd) => ci::run(&cmd, &cli.global),
    }
}
