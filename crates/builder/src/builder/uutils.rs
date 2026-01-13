//! uutils coreutils builder.

use crate::builder::vendor;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Build uutils coreutils.
pub fn build() -> Result<()> {
    println!("=== Building uutils ===");

    let src = vendor::require("uutils")?;

    run_cargo(&src, &["build", "--release", "--features", "unix"])?;

    println!("  Built: vendor/uutils/target/release/coreutils");
    Ok(())
}

/// Get path to coreutils binary.
pub fn binary_path() -> &'static str {
    "vendor/uutils/target/release/coreutils"
}

/// Commands to symlink from coreutils.
pub fn commands() -> &'static [&'static str] {
    &[
        "ls", "cat", "cp", "mv", "rm", "mkdir", "chmod", "chown", "ln", "echo", "env", "pwd",
        "head", "tail", "wc", "sort", "uniq", "tr", "cut", "grep", "test", "[", "true", "false",
        "sleep", "date", "uname", "mount", "umount", "id",
    ]
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
