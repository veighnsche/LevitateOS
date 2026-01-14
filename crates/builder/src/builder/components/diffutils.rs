//! diffutils builder.

use super::Buildable;
use crate::builder::vendor;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// diffutils component.
pub struct Diffutils;

impl Buildable for Diffutils {
    fn name(&self) -> &'static str {
        "diffutils"
    }

    fn build(&self) -> Result<()> {
        println!("=== Building diffutils ===");
        let src = vendor::require("diffutils")?;
        run_cargo(&src, &["build", "--release"])?;
        println!("  Built: vendor/diffutils/target/release/diff, cmp");
        Ok(())
    }

    fn binaries(&self) -> &'static [(&'static str, &'static str)] {
        &[
            ("vendor/diffutils/target/release/diff", "bin/diff"),
            ("vendor/diffutils/target/release/cmp", "bin/cmp"),
        ]
    }
}

fn run_cargo(dir: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .args(args)
        .current_dir(dir)
        .status()
        .context("Failed to run cargo")?;
    if !status.success() {
        bail!("cargo build failed");
    }
    Ok(())
}
