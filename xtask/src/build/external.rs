//! TEAM_435: External project build commands
//!
//! Builds unmodified external projects (coreutils, brush) against our sysroot.
//! External projects are cloned at build time and gitignored (like npm/go modules).
//!
//! ## Future Enhancement
//!
//! Once LevitateOS has a dynamic linker (ld.so), we can download pre-built
//! binaries instead of building from source. This would eliminate the
//! clone+build step entirely.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

const COREUTILS_REPO: &str = "https://github.com/uutils/coreutils";
const BRUSH_REPO: &str = "https://github.com/reubeno/brush";

/// Clone coreutils if not present (idempotent)
fn clone_coreutils() -> Result<()> {
    let dir = PathBuf::from("toolchain/coreutils");
    if !dir.exists() {
        println!("📥 Cloning uutils/coreutils...");
        let status = Command::new("git")
            .args(["clone", "--depth=1", COREUTILS_REPO, "toolchain/coreutils"])
            .status()
            .context("Failed to clone coreutils")?;
        if !status.success() {
            bail!("Failed to clone coreutils repository");
        }
    }
    Ok(())
}

/// Clone brush if not present (idempotent)
fn clone_brush() -> Result<()> {
    let dir = PathBuf::from("toolchain/brush");
    if !dir.exists() {
        println!("📥 Cloning reubeno/brush...");
        let status = Command::new("git")
            .args(["clone", "--depth=1", BRUSH_REPO, "toolchain/brush"])
            .status()
            .context("Failed to clone brush")?;
        if !status.success() {
            bail!("Failed to clone brush repository");
        }
    }
    Ok(())
}

/// Get RUSTFLAGS for building against our sysroot
fn get_sysroot_rustflags() -> String {
    let sysroot_path = std::env::current_dir()
        .map(|p| p.join("toolchain/sysroot"))
        .unwrap_or_else(|_| PathBuf::from("toolchain/sysroot"));

    format!(
        "-C panic=abort \
         -C link-arg=-nostartfiles \
         -C link-arg=-static \
         -C link-arg=-Wl,--allow-multiple-definition \
         -C link-arg=-L{}/lib",
        sysroot_path.display()
    )
}

/// Build coreutils against our sysroot
pub fn build_coreutils(arch: &str) -> Result<()> {
    // Ensure cloned
    clone_coreutils()?;

    // Ensure sysroot exists
    if !super::sysroot::sysroot_exists() {
        println!("⚠️  Sysroot not found, building first...");
        super::sysroot::build_sysroot(arch)?;
    }

    let target = match arch {
        "x86_64" => "x86_64-unknown-linux-gnu",
        "aarch64" => "aarch64-unknown-linux-gnu",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    println!("🔧 Building coreutils for {}...", arch);

    // Limited feature set - only utilities that work with current c-gull
    // Missing libc functions: getpwuid, getgrgid (ls), nl_langinfo (date)
    let features = "cat,echo,head,mkdir,pwd,rm,tail,touch";

    let rustflags = get_sysroot_rustflags();

    let status = Command::new("cargo")
        .current_dir("toolchain/coreutils")
        .arg("+nightly-2025-04-28")
        .env_remove("RUSTUP_TOOLCHAIN")
        .env("RUSTFLAGS", &rustflags)
        .args([
            "build",
            "--release",
            "--target", target,
            "-Z", "build-std=std,panic_abort",
            "-Z", "build-std-features=panic_immediate_abort",
            "--no-default-features",
            "-p", "coreutils",
            "--features", features,
        ])
        .status()
        .context("Failed to build coreutils")?;

    if !status.success() {
        bail!("Failed to build coreutils");
    }

    // Copy to output directory
    let src = PathBuf::from(format!(
        "toolchain/coreutils/target/{}/release/coreutils",
        target
    ));
    let out_dir = PathBuf::from(format!("toolchain/coreutils-out/{}/release", target));
    std::fs::create_dir_all(&out_dir)?;
    let dst = out_dir.join("coreutils");

    if src.exists() {
        std::fs::copy(&src, &dst)?;
        println!("📦 Built coreutils: {}", dst.display());
    } else {
        bail!("Coreutils binary not found at {}", src.display());
    }

    println!("✅ Coreutils ready");
    Ok(())
}

/// Build brush shell against our sysroot
pub fn build_brush(arch: &str) -> Result<()> {
    // Ensure cloned
    clone_brush()?;

    // Ensure sysroot exists
    if !super::sysroot::sysroot_exists() {
        println!("⚠️  Sysroot not found, building first...");
        super::sysroot::build_sysroot(arch)?;
    }

    let target = match arch {
        "x86_64" => "x86_64-unknown-linux-gnu",
        "aarch64" => "aarch64-unknown-linux-gnu",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    println!("🔧 Building brush for {}...", arch);

    let rustflags = get_sysroot_rustflags();

    let status = Command::new("cargo")
        .current_dir("toolchain/brush")
        .arg("+nightly-2025-04-28")
        .env_remove("RUSTUP_TOOLCHAIN")
        .env("RUSTFLAGS", &rustflags)
        .args([
            "build",
            "--release",
            "--target", target,
            "-Z", "build-std=std,panic_abort",
            "-Z", "build-std-features=panic_immediate_abort",
            "-p", "brush",
        ])
        .status()
        .context("Failed to build brush")?;

    if !status.success() {
        bail!("Failed to build brush");
    }

    // Copy to output directory
    let src = PathBuf::from(format!(
        "toolchain/brush/target/{}/release/brush",
        target
    ));
    let out_dir = PathBuf::from(format!("toolchain/brush-out/{}/release", target));
    std::fs::create_dir_all(&out_dir)?;
    let dst = out_dir.join("brush");

    if src.exists() {
        std::fs::copy(&src, &dst)?;
        println!("📦 Built brush: {}", dst.display());
    } else {
        bail!("Brush binary not found at {}", src.display());
    }

    println!("✅ Brush ready");
    Ok(())
}

/// Check if coreutils output exists
pub fn coreutils_exists(arch: &str) -> bool {
    let target = match arch {
        "x86_64" => "x86_64-unknown-linux-gnu",
        "aarch64" => "aarch64-unknown-linux-gnu",
        _ => return false,
    };
    PathBuf::from(format!("toolchain/coreutils-out/{}/release/coreutils", target)).exists()
}

/// Check if brush output exists
pub fn brush_exists(arch: &str) -> bool {
    let target = match arch {
        "x86_64" => "x86_64-unknown-linux-gnu",
        "aarch64" => "aarch64-unknown-linux-gnu",
        _ => return false,
    };
    PathBuf::from(format!("toolchain/brush-out/{}/release/brush", target)).exists()
}
