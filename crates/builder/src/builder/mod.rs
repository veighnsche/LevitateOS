//! Build system modules for LevitateOS components.

pub mod brush;
pub mod glibc;
pub mod initramfs;
pub mod linux;
pub mod qemu;
pub mod sudo_rs;
pub mod systemd;
pub mod uutils;
pub mod vendor;

use anyhow::Result;
use clap::Subcommand;

/// Build commands for the CLI.
#[derive(Subcommand)]
pub enum BuildCommands {
    /// Build everything (fetch + all components + initramfs)
    All,
    /// Fetch source repositories
    Fetch {
        /// Source name (or --all)
        name: Option<String>,
    },
    /// Show cache status
    Status,
    /// Clean cached sources
    Clean {
        /// Source name (omit for all)
        name: Option<String>,
    },
    /// Build Linux kernel
    Linux,
    /// Build systemd
    Systemd,
    /// Build uutils (coreutils)
    Uutils,
    /// Build sudo-rs
    SudoRs,
    /// Build brush shell
    Brush,
    /// Collect glibc libraries
    Glibc,
    /// Create initramfs CPIO
    Initramfs,
    /// Boot in QEMU
    Run,
}

/// Build all components.
pub fn build_all() -> Result<()> {
    println!("=== Building LevitateOS ===\n");

    vendor::fetch_all()?;
    linux::build()?;
    systemd::build()?;
    uutils::build()?;
    sudo_rs::build()?;
    brush::build()?;
    glibc::collect()?;
    initramfs::create()?;

    println!("\n=== Build complete ===");
    println!("Run with: builder run");

    Ok(())
}
