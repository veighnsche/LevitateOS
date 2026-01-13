//! Linux kernel build module
//!
//! TEAM_474: Linux kernel pivot - build Linux instead of custom kernel

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Path to Linux kernel source
const LINUX_SRC: &str = "linux";

/// Build Linux kernel for specified architecture
pub fn build_linux_kernel(arch: &str) -> Result<()> {
    let linux_path = Path::new(LINUX_SRC);

    if !linux_path.exists() {
        bail!("Linux kernel source not found at '{LINUX_SRC}'. Run 'git submodule update --init'");
    }

    let (make_arch, defconfig, image_target, output_path) = match arch {
        "x86_64" => (
            "x86_64",
            "levitate_defconfig",
            "bzImage",
            "linux/arch/x86/boot/bzImage",
        ),
        "aarch64" => (
            "arm64",
            "levitate_defconfig",
            "Image",
            "linux/arch/arm64/boot/Image",
        ),
        _ => bail!("Unsupported architecture: {arch}"),
    };

    // Check for cross-compiler if needed
    let cross_compile = if arch == "aarch64" && cfg!(target_arch = "x86_64") {
        "aarch64-linux-gnu-"
    } else {
        ""
    };

    println!("Building Linux kernel for {arch}...");

    // Step 1: Configure kernel
    println!("  Configuring with {defconfig}...");
    let config_status = Command::new("make")
        .current_dir(linux_path)
        .env("ARCH", make_arch)
        .env("CROSS_COMPILE", cross_compile)
        .args([defconfig])
        .status()
        .context("Failed to run make defconfig")?;

    if !config_status.success() {
        bail!("Kernel configuration failed");
    }

    // Step 2: Build kernel
    let cpus = num_cpus::get();
    println!("  Building {image_target} with {cpus} jobs...");

    let build_status = Command::new("make")
        .current_dir(linux_path)
        .env("ARCH", make_arch)
        .env("CROSS_COMPILE", cross_compile)
        .args(["-j", &cpus.to_string(), image_target])
        .status()
        .context("Failed to build kernel")?;

    if !build_status.success() {
        bail!("Kernel build failed");
    }

    // Verify output exists
    if !Path::new(output_path).exists() {
        bail!("Kernel image not found at {output_path}");
    }

    let size = std::fs::metadata(output_path)?.len();
    println!(
        "  Built: {} ({:.1} MB)",
        output_path,
        size as f64 / 1_000_000.0
    );

    Ok(())
}
