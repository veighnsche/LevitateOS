//! uutils (coreutils) builder.

use super::Buildable;
use crate::builder::vendor;
use anyhow::{Context, Result};
use std::process::Command;

/// uutils coreutils component.
pub struct Uutils;

impl Buildable for Uutils {
    fn name(&self) -> &'static str {
        "uutils"
    }

    fn build(&self) -> Result<()> {
        println!("=== Building uutils ===");
        let src = vendor::require("uutils")?;
        let status = Command::new("cargo")
            .args(["build", "--release", "--features", "unix"])
            .current_dir(&src)
            .env("CARGO_UNSTABLE_WORKSPACES", "disable-inheritance")
            .status()
            .context("Failed to run cargo")?;
        if !status.success() {
            anyhow::bail!("cargo build failed");
        }
        println!("  Built: vendor/uutils/target/release/coreutils");
        Ok(())
    }

    fn binaries(&self) -> &'static [(&'static str, &'static str)] {
        &[("vendor/uutils/target/release/coreutils", "bin/coreutils")]
    }

    fn symlinks(&self) -> &'static [(&'static str, &'static str)] {
        &[
            ("ls", "coreutils"),
            ("cat", "coreutils"),
            ("cp", "coreutils"),
            ("mv", "coreutils"),
            ("rm", "coreutils"),
            ("mkdir", "coreutils"),
            ("chmod", "coreutils"),
            ("chown", "coreutils"),
            ("ln", "coreutils"),
            ("echo", "coreutils"),
            ("env", "coreutils"),
            ("pwd", "coreutils"),
            ("head", "coreutils"),
            ("tail", "coreutils"),
            ("wc", "coreutils"),
            ("sort", "coreutils"),
            ("uniq", "coreutils"),
            ("tr", "coreutils"),
            ("cut", "coreutils"),
            ("grep", "coreutils"),
            ("test", "coreutils"),
            ("[", "coreutils"),
            ("true", "coreutils"),
            ("false", "coreutils"),
            ("sleep", "coreutils"),
            ("date", "coreutils"),
            ("uname", "coreutils"),
            ("id", "coreutils"),
            ("whoami", "coreutils"),
            ("basename", "coreutils"),
            ("dirname", "coreutils"),
            ("touch", "coreutils"),
            ("rmdir", "coreutils"),
            ("readlink", "coreutils"),
            ("realpath", "coreutils"),
        ]
    }
}
