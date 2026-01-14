//! Build system for `LevitateOS`.
//!
//! Structure:
//! - `components/` - Buildable components (linux, systemd, brush, etc.)
//!   - `registry` - Single source of truth for all components
//! - `auth/` - Authentication configuration
//! - `initramfs` - Initramfs CPIO builder
//! - `vendor` - Source fetching

pub mod auth;
pub mod components;
pub mod initramfs;
pub mod vendor;

// Re-export for direct access
pub use components::{glibc, registry};

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
    /// Build a specific component by name
    Build {
        /// Component name (use 'list' to see available)
        name: String,
    },
    /// List available components
    List,
    /// Collect glibc libraries
    Glibc,
    /// Create initramfs CPIO
    Initramfs,
}

/// Build all components.
pub fn build_all() -> Result<()> {
    println!("=== Building LevitateOS ===\n");

    vendor::fetch_all()?;

    // Build all registered components
    for component in registry::COMPONENTS {
        component.build()?;
    }

    initramfs::create()?;

    println!("\n=== Build complete ===");
    println!("Run with: cargo xtask vm start");

    Ok(())
}

/// Build a single component by name.
pub fn build_component(name: &str) -> Result<()> {
    if let Some(component) = registry::get(name) { component.build() } else {
        eprintln!("Unknown component: {name}");
        eprintln!("Available components:");
        for n in registry::names() {
            eprintln!("  - {n}");
        }
        anyhow::bail!("Unknown component: {name}")
    }
}

/// List all available components.
pub fn list_components() {
    println!("Available components:");
    for component in registry::COMPONENTS {
        println!(
            "  {} - {} binaries, {} symlinks",
            component.name(),
            component.binaries().len(),
            component.symlinks().len()
        );
    }
}
