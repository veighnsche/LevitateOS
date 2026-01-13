//! OpenRC build support
//!
//! TEAM_475: Build OpenRC init system from source with musl.
//!
//! OpenRC provides:
//! - Service dependency management
//! - Runlevel support (sysinit, boot, default, shutdown)
//! - rc-service, rc-update, rc-status tools
//!
//! Built statically with musl for LevitateOS.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Git repository URL for OpenRC
pub const REPO: &str = "https://github.com/OpenRC/openrc.git";

/// OpenRC version tag to build
pub const VERSION: &str = "0.54";

/// Get the clone directory for OpenRC source
pub fn clone_dir() -> PathBuf {
    PathBuf::from("toolchain/openrc")
}

/// Get the build directory (separate from source)
pub fn build_dir() -> PathBuf {
    PathBuf::from("toolchain/openrc-build")
}

/// Get the output directory for built binaries
pub fn output_dir(arch: &str) -> PathBuf {
    PathBuf::from(format!("toolchain/openrc-out/{arch}"))
}

/// Check if OpenRC has been built for the given architecture
pub fn exists(arch: &str) -> bool {
    output_dir(arch).join("sbin/openrc").exists()
}

/// Clone the OpenRC repository if not present
pub fn clone_repo() -> Result<()> {
    let dir = clone_dir();
    if dir.exists() {
        if !dir.join(".git").exists() {
            bail!(
                "Directory {} exists but is not a git repository. Remove it and try again.",
                dir.display()
            );
        }
        return Ok(());
    }

    println!("Cloning OpenRC v{}...", VERSION);
    let status = Command::new("git")
        .args([
            "clone",
            "--depth=1",
            "--branch",
            VERSION,
            REPO,
            &dir.to_string_lossy(),
        ])
        .status()
        .context("Failed to clone OpenRC")?;

    if !status.success() {
        bail!("Failed to clone OpenRC from {REPO}");
    }

    Ok(())
}

/// Check if meson is available
fn ensure_meson() -> Result<()> {
    let output = Command::new("meson").arg("--version").output();

    if output.is_err() || !output.unwrap().status.success() {
        bail!(
            "meson not found.\n\n\
             Install meson build system:\n\
             Fedora: sudo dnf install meson\n\
             Ubuntu: sudo apt install meson\n\
             Arch:   sudo pacman -S meson"
        );
    }
    Ok(())
}

/// Check if ninja is available
fn ensure_ninja() -> Result<()> {
    let output = Command::new("ninja").arg("--version").output();

    if output.is_err() || !output.unwrap().status.success() {
        bail!(
            "ninja not found.\n\n\
             Install ninja build tool:\n\
             Fedora: sudo dnf install ninja-build\n\
             Ubuntu: sudo apt install ninja-build\n\
             Arch:   sudo pacman -S ninja"
        );
    }
    Ok(())
}

/// Check if musl-gcc is available
fn ensure_musl_gcc() -> Result<()> {
    let output = Command::new("musl-gcc").arg("--version").output();

    if output.is_err() || !output.unwrap().status.success() {
        bail!(
            "musl-gcc not found.\n\n\
             Install musl development tools:\n\
             Fedora: sudo dnf install musl-gcc musl-devel\n\
             Ubuntu: sudo apt install musl-tools musl-dev\n\
             Arch:   sudo pacman -S musl"
        );
    }
    Ok(())
}

/// Build OpenRC with meson and musl
pub fn build(arch: &str) -> Result<()> {
    if arch != "x86_64" {
        bail!("OpenRC build currently only supports x86_64");
    }

    // Check dependencies
    ensure_meson()?;
    ensure_ninja()?;
    ensure_musl_gcc()?;

    // Clone if needed
    clone_repo()?;

    let src_dir = clone_dir();
    let src_abs = std::fs::canonicalize(&src_dir)?;
    let build_dir = build_dir();
    let install_dir = output_dir(arch);

    // Clean previous build
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir)?;
    }
    std::fs::create_dir_all(&build_dir)?;
    let build_abs = std::fs::canonicalize(&build_dir)?;

    // Create install directory
    std::fs::create_dir_all(&install_dir)?;
    let abs_install = std::fs::canonicalize(&install_dir)?;

    println!("Configuring OpenRC with meson...");

    // Create a cross-file for musl static build
    let cross_file = src_dir.join("musl-cross.txt");
    let abs_cross_file = std::fs::canonicalize(&src_dir)?.join("musl-cross.txt");
    std::fs::write(
        &cross_file,
        r#"[binaries]
c = 'musl-gcc'

[built-in options]
# TEAM_475: Use -idirafter for kernel headers so musl headers take precedence
c_args = ['-static', '-Os', '-idirafter', '/usr/include']
c_link_args = ['-static']

[properties]
# Disable features that don't work well with static musl
needs_exe_wrapper = false
"#,
    )?;

    // Configure with meson (run from project root, use absolute paths)
    // TEAM_475: Use prefix=/ so DESTDIR gives us clean paths like /sbin/openrc
    let status = Command::new("meson")
        .args([
            "setup",
            &build_abs.to_string_lossy(),
            &src_abs.to_string_lossy(),
            "--prefix=/",
            "--cross-file",
            &abs_cross_file.to_string_lossy(),
            // TEAM_475: Build static libraries to avoid shared object conflicts with -static
            "--default-library=static",
            // Disable optional features for minimal build
            "-Dpam=false",           // boolean
            "-Daudit=disabled",      // feature
            "-Dselinux=disabled",    // feature
            "-Dbash-completions=false",  // boolean
            "-Dzsh-completions=false",   // boolean
            "-Dpkgconfig=false",     // boolean
            // Use built-in implementations
            "-Dnewnet=false",        // boolean
            "-Dsysvinit=false",      // TEAM_475: Disable to avoid agetty service dependencies
            "-Dos=Linux",            // target OS
        ])
        .status()
        .context("meson setup failed")?;

    if !status.success() {
        bail!("meson configuration failed");
    }

    println!("Building OpenRC...");

    // Build
    let status = Command::new("ninja")
        .current_dir(&build_abs)
        .status()
        .context("ninja build failed")?;

    if !status.success() {
        bail!("OpenRC build failed");
    }

    println!("Installing OpenRC to {}...", install_dir.display());

    // Install using DESTDIR to redirect all absolute paths to our output directory
    let status = Command::new("ninja")
        .current_dir(&build_abs)
        .env("DESTDIR", &abs_install)
        .args(["install"])
        .status()
        .context("ninja install failed")?;

    if !status.success() {
        bail!("OpenRC install failed");
    }

    // Verify key binaries exist (rc-status is in bin/, others in sbin/)
    let binaries = ["sbin/openrc", "sbin/openrc-run", "sbin/rc-service", "sbin/rc-update", "bin/rc-status"];
    for bin in &binaries {
        let path = install_dir.join(bin);
        if !path.exists() {
            bail!("Expected binary not found: {}", path.display());
        }
    }

    // Check if statically linked
    let openrc_bin = install_dir.join("sbin/openrc");
    let file_output = Command::new("file")
        .arg(&openrc_bin)
        .output()
        .context("Failed to run file command")?;

    let file_info = String::from_utf8_lossy(&file_output.stdout);
    if file_info.contains("statically linked") {
        println!("  OpenRC is statically linked");
    } else if file_info.contains("dynamically linked") {
        println!("  Warning: OpenRC is dynamically linked");
        println!("  file output: {}", file_info.trim());
    }

    let size = std::fs::metadata(&openrc_bin)?.len();
    println!("  Built: {} ({:.1} KB)", openrc_bin.display(), size as f64 / 1024.0);
    println!("OpenRC build complete");

    Ok(())
}

/// Ensure OpenRC is built, or fail with a clear error
pub fn require(arch: &str) -> Result<PathBuf> {
    let path = output_dir(arch);
    if !path.join("sbin/openrc").exists() {
        bail!(
            "OpenRC not found at {}.\nRun 'cargo xtask build openrc' first.",
            path.display()
        );
    }
    Ok(path)
}

/// Clean OpenRC build artifacts
pub fn clean() -> Result<()> {
    let build = build_dir();
    if build.exists() {
        println!("Removing OpenRC build directory...");
        std::fs::remove_dir_all(&build)?;
    }

    for arch in ["x86_64", "aarch64"] {
        let out = output_dir(arch);
        if out.exists() {
            println!("Removing OpenRC output for {}...", arch);
            std::fs::remove_dir_all(&out)?;
        }
    }

    Ok(())
}
