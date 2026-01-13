//! Build orchestration module
//!
//! TEAM_466: Extracted from commands.rs during refactor.
//! TEAM_476: Rewritten for Linux + BusyBox + OpenRC build path.
//!
//! High-level build coordination functions.

use anyhow::Result;

/// Build all components for a bootable Linux distribution.
///
/// This builds:
/// 1. Linux kernel (from linux/ submodule)
/// 2. BusyBox (static, musl-linked)
/// 3. OpenRC (static, musl-linked)
/// 4. Initramfs (CPIO archive with all components)
pub fn build_all(arch: &str) -> Result<()> {
    println!("Building LevitateOS for {arch}...\n");

    // Build Linux kernel
    super::linux::build_linux_kernel(arch)?;

    // Build BusyBox
    super::busybox::ensure_built(arch)?;

    // Build OpenRC
    super::openrc::build(arch)?;

    // Create initramfs with OpenRC
    super::initramfs::create_initramfs(arch)?;

    println!("\n✅ Build complete for {arch}");
    Ok(())
}
