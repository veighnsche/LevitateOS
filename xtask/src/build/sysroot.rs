//! TEAM_435: c-gull sysroot build commands
//!
//! Builds libc.a from c-gull/c-ward for linking userspace programs.
//! This replaces the Eyra approach with a pre-built sysroot.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

const C_WARD_REPO: &str = "https://github.com/sunfishcode/c-ward";

/// Clone c-ward repository if not present (idempotent)
fn clone_c_ward() -> Result<()> {
    let dir = PathBuf::from("toolchain/c-ward");
    if !dir.exists() {
        println!("📥 Cloning c-ward...");
        let status = Command::new("git")
            .args(["clone", "--depth=1", C_WARD_REPO, "toolchain/c-ward"])
            .status()
            .context("Failed to clone c-ward")?;
        if !status.success() {
            bail!("Failed to clone c-ward repository");
        }
    }
    Ok(())
}

/// Build libc.a from libc-levitateos wrapper crate
fn build_libc(arch: &str) -> Result<()> {
    let target = match arch {
        "x86_64" => "x86_64-unknown-linux-gnu",
        "aarch64" => "aarch64-unknown-linux-gnu",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    println!("🔧 Building libc.a for {}...", arch);

    let status = Command::new("cargo")
        .current_dir("toolchain/libc-levitateos")
        .arg("+nightly-2025-04-28")
        .env_remove("RUSTUP_TOOLCHAIN")
        .args([
            "build",
            "--release",
            "--target", target,
            "-Z", "build-std=std,panic_abort",
            "-Z", "build-std-features=panic_immediate_abort",
        ])
        .status()
        .context("Failed to build libc-levitateos")?;

    if !status.success() {
        bail!("Failed to build libc.a");
    }

    Ok(())
}

/// Create sysroot directory structure with libc.a
fn create_sysroot(arch: &str) -> Result<()> {
    let target = match arch {
        "x86_64" => "x86_64-unknown-linux-gnu",
        "aarch64" => "aarch64-unknown-linux-gnu",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    let sysroot_lib = PathBuf::from("toolchain/sysroot/lib");
    std::fs::create_dir_all(&sysroot_lib)?;

    // Copy the built library to sysroot
    let src = PathBuf::from(format!(
        "toolchain/libc-levitateos/target/{}/release/liblibc_levitateos.a",
        target
    ));
    let dst = sysroot_lib.join("libc.a");

    if src.exists() {
        std::fs::copy(&src, &dst)?;
        println!("📦 Created sysroot/lib/libc.a");
    } else {
        bail!("libc.a not found at {}", src.display());
    }

    Ok(())
}

/// Main entry point: build complete c-gull sysroot
pub fn build_sysroot(arch: &str) -> Result<()> {
    println!("🔧 Building c-gull sysroot for {}...", arch);

    // 1. Clone c-ward if needed
    clone_c_ward()?;

    // 2. Build libc.a
    build_libc(arch)?;

    // 3. Create sysroot structure
    create_sysroot(arch)?;

    println!("✅ Sysroot ready at toolchain/sysroot/");
    Ok(())
}

/// Check if sysroot exists and is valid
pub fn sysroot_exists() -> bool {
    PathBuf::from("toolchain/sysroot/lib/libc.a").exists()
}
