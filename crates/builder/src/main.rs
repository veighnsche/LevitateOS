//! # LevitateOS Builder
//!
//! Build minimal Linux systems from source with type-safe, fast Rust tooling.
//!
//! ## Usage
//!
//! ```bash
//! builder all           # Fetch + build all components + initramfs
//! builder fetch <name>  # Fetch a source repository
//! builder status        # Show cache status
//! builder linux         # Build Linux kernel
//! builder systemd       # Build systemd
//! builder uutils        # Build uutils (coreutils)
//! builder sudo-rs       # Build sudo-rs
//! builder brush         # Build brush shell
//! builder glibc         # Collect system libraries
//! builder initramfs     # Create initramfs CPIO
//! builder run           # Boot in QEMU
//! ```
//!
//! ## Components Built
//!
//! - **Linux kernel** (v6.18)
//! - **systemd** (init system, v259)
//! - **glibc** (dynamic linking, from host)
//! - **uutils** (coreutils replacement)
//! - **sudo-rs** (sudo/su)
//! - **brush** (shell)

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
        builder::BuildCommands::Linux => builder::linux::build()?,
        builder::BuildCommands::Systemd => builder::systemd::build()?,
        builder::BuildCommands::UtilLinux => builder::util_linux::build()?,
        builder::BuildCommands::Uutils => builder::uutils::build()?,
        builder::BuildCommands::SudoRs => builder::sudo_rs::build()?,
        builder::BuildCommands::Brush => builder::brush::build()?,
        builder::BuildCommands::Glibc => builder::glibc::collect()?,
        builder::BuildCommands::Initramfs => builder::initramfs::create()?,
    }

    Ok(())
}
