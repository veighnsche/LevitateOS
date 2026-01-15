//! Linux kernel builder.
//!
//! Builds the Linux kernel from vendor/linux source.

use crate::builder::vendor;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Build the Linux kernel.
pub fn build() -> Result<()> {
    println!("=== Building Linux kernel ===");

    let src = vendor::require("linux")?;

    // Copy config if it exists
    if Path::new("config/linux.config").exists() {
        std::fs::copy("config/linux.config", src.join(".config"))?;
    }

    run_make(&src, &["olddefconfig"])?;
    run_make(&src, &["-j", &cpus(), "bzImage"])?;

    println!("  Built: vendor/linux/arch/x86/boot/bzImage");
    Ok(())
}

fn run_make(dir: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("make")
        .args(args)
        .current_dir(dir)
        .status()?;

    if !status.success() {
        anyhow::bail!("make failed");
    }
    Ok(())
}

fn cpus() -> String {
    std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1)
        .to_string()
}
