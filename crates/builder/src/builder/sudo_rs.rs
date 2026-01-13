//! sudo-rs builder.

use crate::builder::vendor;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Build sudo-rs.
pub fn build() -> Result<()> {
    println!("=== Building sudo-rs ===");

    let src = vendor::require("sudo-rs")?;

    run_cargo(&src, &["build", "--release"])?;

    println!("  Built: vendor/sudo-rs/target/release/{{sudo,su}}");
    Ok(())
}

/// Get path to sudo binary.
pub fn sudo_path() -> &'static str {
    "vendor/sudo-rs/target/release/sudo"
}

/// Get path to su binary.
pub fn su_path() -> &'static str {
    "vendor/sudo-rs/target/release/su"
}

fn run_cargo(dir: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .args(args)
        .current_dir(dir)
        .env("CARGO_UNSTABLE_WORKSPACES", "disable-inheritance")
        .status()
        .context("Failed to run cargo")?;

    if !status.success() {
        bail!("cargo build failed");
    }
    Ok(())
}
