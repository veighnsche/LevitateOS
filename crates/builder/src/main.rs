//! # LevitateOS Builder
//!
//! Build minimal Linux systems from source with type-safe, fast Rust tooling.
//!
//! ## Usage
//!
//! ```bash
//! builder all            # Fetch + build all components + initramfs
//! builder list           # List available components
//! builder build <name>   # Build a specific component
//! builder fetch <name>   # Fetch a source repository
//! builder status         # Show cache status
//! builder glibc          # Collect system libraries
//! builder initramfs      # Create initramfs CPIO
//! ```
//!
//! ## Components
//!
//! All components are defined in `registry.rs`. Use `builder list` to see them.

use anyhow::Result;
use clap::Parser;

mod builder;

#[derive(Parser)]
#[command(name = "builder", about = "LevitateOS distribution builder")]
struct Cli {
    #[command(subcommand)]
    command: builder::BuildCommands,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        builder::BuildCommands::All => builder::build_all()?,
        builder::BuildCommands::Fetch { name } => {
            if let Some(name) = name {
                if name == "--all" {
                    builder::vendor::fetch_all()?;
                } else {
                    builder::vendor::fetch(&name)?;
                }
            } else {
                builder::vendor::list();
            }
        }
        builder::BuildCommands::Status => builder::vendor::status()?,
        builder::BuildCommands::Clean { name } => builder::vendor::clean(name.as_deref())?,
        builder::BuildCommands::Build { name } => builder::build_component(&name)?,
        builder::BuildCommands::List => builder::list_components(),
        builder::BuildCommands::Glibc => builder::glibc::collect()?,
        builder::BuildCommands::Initramfs => builder::initramfs::create()?,
    }

    Ok(())
}
