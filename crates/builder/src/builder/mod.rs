//! Build system for LevitateOS.
//!
//! Structure:
//! - `kernel` - Linux kernel builder (only source-built component)
//! - `fedora` - Fedora ISO extraction for userspace binaries
//! - `libraries` - Library collection from Fedora
//! - `auth/` - Authentication configuration
//! - `initramfs` - Initramfs CPIO builder
//! - `vendor` - Source fetching (kernel only)

pub mod auth;
pub mod fedora;
pub mod initramfs;
pub mod kernel;
pub mod libraries;
pub mod vendor;

use anyhow::Result;
use clap::Subcommand;

/// Build commands for the CLI.
#[derive(Subcommand)]
pub enum BuildCommands {
    /// Build everything (fetch + kernel + initramfs)
    All,
    /// Fetch source repositories (kernel only)
    Fetch {
        /// Source name (or omit for all)
        name: Option<String>,
    },
    /// Show cache status
    Status,
    /// Clean cached sources
    Clean {
        /// Source name (omit for all)
        name: Option<String>,
    },
    /// Build the Linux kernel
    Kernel,
    /// Create initramfs CPIO (includes libraries from Fedora)
    Initramfs,
}

/// Build everything: fetch sources, build kernel, create initramfs.
pub fn build_all() -> Result<()> {
    println!("=== Building LevitateOS ===\n");

    vendor::fetch_all()?;
    kernel::build()?;
    initramfs::create()?;

    println!("\n=== Build complete ===");
    println!("Run with: cargo xtask vm start");

    Ok(())
}
