//! findutils builder.

use super::Buildable;
use crate::builder::vendor;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// findutils component.
pub struct Findutils;

impl Buildable for Findutils {
    fn name(&self) -> &'static str {
        "findutils"
    }

    fn build(&self) -> Result<()> {
        println!("=== Building findutils ===");
        let src = vendor::require("findutils")?;
        run_cargo(&src, &["build", "--release"])?;
        println!("  Built: vendor/findutils/target/release/find, xargs");
        Ok(())
    }

    fn binaries(&self) -> &'static [(&'static str, &'static str)] {
        &[
            ("vendor/findutils/target/release/find", "bin/find"),
            ("vendor/findutils/target/release/xargs", "bin/xargs"),
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
