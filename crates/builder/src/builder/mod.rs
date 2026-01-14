//! Build system for LevitateOS.
//!
//! Structure:
//! - `components/` - Buildable components (linux, systemd, brush, etc.)
//! - `auth/` - Authentication configuration
//! - `initramfs` - Initramfs CPIO builder
//! - `vendor` - Source fetching

pub mod auth;
pub mod components;
pub mod initramfs;
pub mod vendor;

// Re-export components for convenience
pub use components::{brush, glibc, linux, sudo_rs, systemd, util_linux, uutils};

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
    /// Build util-linux (agetty, login, disk utilities)
    UtilLinux,
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
}

/// Build all components.
pub fn build_all() -> Result<()> {
    println!("=== Building LevitateOS ===\n");

    vendor::fetch_all()?;
    linux::build()?;
    systemd::build()?;
    util_linux::build()?;
    uutils::build()?;
    sudo_rs::build()?;
    brush::build()?;
    initramfs::create()?;

    println!("\n=== Build complete ===");
    println!("Run with: cargo xtask vm start");

    Ok(())
}
