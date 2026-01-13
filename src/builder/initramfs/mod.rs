//! Initramfs builder module
//!
//! TEAM_474: Declarative initramfs builder with pure Rust CPIO writer and TUI dashboard.
//!
//! Builds initramfs CPIO archives from declarative TOML manifest (`initramfs/initramfs.toml`).

mod builder;
pub mod cpio;
mod manifest;
mod tui;

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Build initramfs for the given architecture
///
/// Loads `initramfs/initramfs.toml` and produces `target/initramfs/{arch}.cpio`
pub fn build_initramfs(arch: &str) -> Result<PathBuf> {
    let base_dir = PathBuf::from("initramfs");
    let manifest_path = base_dir.join("initramfs.toml");

    let manifest = manifest::Manifest::load(&manifest_path.to_string_lossy(), arch, &base_dir)?;

    // Validate with helpful error messages
    if let Err(e) = manifest.validate(&base_dir) {
        // Check if it's a missing busybox binary
        if e.to_string().contains("busybox") {
            eprintln!("  Hint: Run 'cargo xtask build busybox' first");
        }
        return Err(e);
    }

    let totals = manifest.get_totals();
    let builder = builder::InitramfsBuilder::new(manifest, arch, &base_dir);

    if tui::should_use_tui() {
        // TUI mode
        let totals_clone = totals.clone();
        tui::run_build_with_tui(arch, &totals, move |emit| {
            let _ = totals_clone; // capture for lifetime
            builder.build_with_events(move |e| emit(e))
        })
    } else {
        // Simple mode
        println!("  Creating initramfs for {}...", arch);
        builder.build_with_events(|event| {
            tui::print_simple_event(&event);
        })
    }
}

/// Create initramfs from declarative manifest
///
/// Builds initramfs and copies to repo root for QEMU.
pub fn create_initramfs(arch: &str) -> Result<()> {
    println!("  Building initramfs for {}...", arch);

    // Require BusyBox and OpenRC to be built first
    super::busybox::require(arch).context("BusyBox binary required")?;
    super::openrc::require(arch).context("OpenRC binaries required")?;

    let output = build_initramfs(arch)?;

    // Copy to repo root for QEMU
    let legacy_path = format!("initramfs_{arch}.cpio");
    std::fs::copy(&output, &legacy_path)?;

    let size_kb = std::fs::metadata(&legacy_path)?.len() / 1024;
    println!("  Initramfs created: {} ({} KB)", legacy_path, size_kb);

    Ok(())
}
