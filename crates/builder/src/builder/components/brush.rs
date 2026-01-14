//! brush shell builder.

use super::Buildable;
use crate::builder::vendor;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// brush shell component.
pub struct Brush;

impl Buildable for Brush {
    fn name(&self) -> &'static str {
        "brush"
    }

    fn build(&self) -> Result<()> {
        println!("=== Building brush ===");
        let src = vendor::require("brush")?;
        run_cargo(&src, &["build", "--release", "-p", "brush-shell"])?;
        println!("  Built: vendor/brush/target/release/brush");
        Ok(())
    }

    fn binaries(&self) -> &'static [(&'static str, &'static str)] {
        &[("vendor/brush/target/release/brush", "bin/brush")]
    }

    fn symlinks(&self) -> &'static [(&'static str, &'static str)] {
        &[("sh", "brush")]
    }
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
