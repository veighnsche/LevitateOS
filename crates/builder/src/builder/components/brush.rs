//! brush shell builder.

use crate::builder::vendor;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Build brush shell.
pub fn build() -> Result<()> {
    println!("=== Building brush ===");

    let src = vendor::require("brush")?;

    run_cargo(&src, &["build", "--release", "-p", "brush-shell"])?;

    println!("  Built: vendor/brush/target/release/brush");
    Ok(())
}

/// Get path to brush binary.
pub fn binary_path() -> &'static str {
    "vendor/brush/target/release/brush"
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
