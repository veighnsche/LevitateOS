//! Helix editor builder.

use super::Buildable;
use crate::builder::vendor;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Helix editor component.
pub struct Helix;

impl Buildable for Helix {
    fn name(&self) -> &'static str {
        "helix"
    }

    fn build(&self) -> Result<()> {
        println!("=== Building helix ===");
        let src = vendor::require("helix")?;
        run_cargo(&src, &["build", "--release"])?;
        println!("  Built: vendor/helix/target/release/hx");
        Ok(())
    }

    fn binaries(&self) -> &'static [(&'static str, &'static str)] {
        &[("vendor/helix/target/release/hx", "bin/hx")]
    }

    fn symlinks(&self) -> &'static [(&'static str, &'static str)] {
        &[("vi", "hx"), ("vim", "hx")]
    }

    fn runtime_dirs(&self) -> &'static [(&'static str, &'static str)] {
        &[("vendor/helix/runtime", "usr/share/helix/runtime")]
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
