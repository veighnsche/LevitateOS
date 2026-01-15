//! # LevitateOS Builder
//!
//! Build minimal Linux systems with Rust tooling.
//!
//! ## Usage
//!
//! ```bash
//! builder all            # Fetch + build kernel + create initramfs
//! builder kernel         # Build the Linux kernel
//! builder fetch          # Fetch kernel source
//! builder status         # Show cache status
//! builder initramfs      # Create initramfs CPIO
//! ```
//!
//! ## Architecture
//!
//! - Kernel: Built from source (vendor/linux)
//! - Userspace: Extracted from Fedora ISO (no compilation)

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
                builder::vendor::fetch(&name)?;
            } else {
                builder::vendor::fetch_all()?;
            }
        }
        builder::BuildCommands::Status => builder::vendor::status()?,
        builder::BuildCommands::Clean { name } => builder::vendor::clean(name.as_deref())?,
        builder::BuildCommands::Kernel => builder::kernel::build()?,
        builder::BuildCommands::Initramfs => builder::initramfs::create()?,
    }

    Ok(())
}
