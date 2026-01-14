//! systemd init system builder.

use crate::builder::vendor;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Build systemd.
pub fn build() -> Result<()> {
    println!("=== Building systemd ===");

    let src = vendor::require("systemd")?;
    let build_dir = src.join("build");

    std::fs::create_dir_all(&build_dir)?;

    // Meson setup - use reconfigure if already configured
    let meson_args = if build_dir.join("build.ninja").exists() {
        vec![
            "setup",
            "build",
            ".",
            "--reconfigure",
            "-Dmode=release",
            "-Dtests=false",
            "-Dman=false",
            "-Dhtml=false",
            "-Dlibmount=enabled",
        ]
    } else {
        vec![
            "setup",
            "build",
            ".",
            "-Dmode=release",
            "-Dtests=false",
            "-Dman=false",
            "-Dhtml=false",
            "-Dlibmount=enabled",
        ]
    };

    run_cmd("meson", &meson_args, &src)?;
    run_cmd("ninja", &["-C", "build"], &src)?;

    println!("  Built: vendor/systemd/build/");
    Ok(())
}

/// Get paths to systemd shared libraries (used by glibc collection).
pub fn lib_paths() -> Vec<&'static str> {
    vec![
        "vendor/systemd/build/src/core/libsystemd-core-259.so",
        "vendor/systemd/build/src/shared/libsystemd-shared-259.so",
    ]
}

fn run_cmd(cmd: &str, args: &[&str], dir: &Path) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status()
        .context(format!("Failed to run {}", cmd))?;

    if !status.success() {
        bail!("{} failed", cmd);
    }
    Ok(())
}
