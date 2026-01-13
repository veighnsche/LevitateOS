//! wlroots build support
//!
//! wlroots is a modular Wayland compositor library used by sway.
//! We build it from source with minimal backends (drm, libinput).

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

use super::alpine;

/// Git repository URL for wlroots
pub const REPO: &str = "https://gitlab.freedesktop.org/wlroots/wlroots.git";

/// Version tag to build
pub const VERSION: &str = "0.18.2";

/// Get the clone directory for wlroots source
pub fn clone_dir() -> PathBuf {
    PathBuf::from("toolchain/wlroots")
}

/// Get the output directory for built libraries
pub fn output_dir(arch: &str) -> PathBuf {
    PathBuf::from(format!("toolchain/wlroots-out/{arch}"))
}

/// Get the build directory
pub fn build_dir() -> PathBuf {
    PathBuf::from("toolchain/wlroots-build")
}

/// Check if wlroots has been built for the given architecture
pub fn exists(arch: &str) -> bool {
    output_dir(arch).join("lib/libwlroots-0.18.so").exists()
}

/// Clone the wlroots repository if not present
pub fn clone_repo() -> Result<()> {
    let dir = clone_dir();
    if dir.exists() {
        if !dir.join(".git").exists() {
            bail!(
                "Directory {} exists but is not a git repository",
                dir.display()
            );
        }
        return Ok(());
    }

    println!("  Cloning wlroots {}...", VERSION);
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
        .context("Failed to clone wlroots")?;

    if !status.success() {
        bail!("Failed to clone wlroots from {REPO}");
    }

    Ok(())
}

/// Build wlroots using distrobox Alpine
pub fn build(arch: &str) -> Result<()> {
    // Ensure Alpine packages are installed
    alpine::ensure_wayland_packages(arch)?;

    // Clone source
    clone_repo()?;

    let src_dir = clone_dir();
    let abs_src = std::fs::canonicalize(&src_dir)
        .context("Failed to get absolute path for wlroots")?;

    // Create output directory and get absolute path
    let out_dir = output_dir(arch);
    std::fs::create_dir_all(&out_dir)?;
    let abs_out = std::fs::canonicalize(&out_dir)
        .context("Failed to get absolute path for wlroots output")?;

    println!("  Building wlroots for {arch}...");

    // Build script for distrobox Alpine
    // Note: Use system-installed packages via pkg-config, not extracted Alpine packages
    // The extracted packages are for runtime in initramfs, not for building
    // TEAM_477: Build against v3.21 to match runtime library versions
    let build_script = format!(
        r#"
set -e

# Ensure we're using Alpine v3.21 repos (not edge) for consistent library versions
if grep -q "/edge/" /etc/apk/repositories 2>/dev/null; then
    echo "  Switching Alpine repos to v3.21..."
    sudo sed -i 's|/edge/|/v3.21/|g' /etc/apk/repositories
    sudo apk update
fi

cd "{src}"

# Clean previous build
rm -rf build

echo "  Configuring wlroots..."
meson setup build \
    --prefix="{out}" \
    --buildtype=release \
    -Dbackends=drm,libinput \
    -Dxwayland=disabled \
    -Dexamples=false \
    -Drenderers=gles2 \
    -Dallocators=gbm

echo "  Compiling wlroots..."
ninja -C build

echo "  Installing wlroots..."
ninja -C build install

echo "  wlroots build complete"
"#,
        src = abs_src.display(),
        out = abs_out.display(),
    );

    let status = Command::new("distrobox")
        .args(["enter", "Alpine", "--"])
        .arg("sh")
        .arg("-c")
        .arg(&build_script)
        .status()
        .context("Failed to build wlroots")?;

    if !status.success() {
        bail!("wlroots build failed");
    }

    // Verify build
    if !exists(arch) {
        bail!("wlroots library not found after build");
    }

    println!("  wlroots built: {}", output_dir(arch).display());
    Ok(())
}

/// Ensure wlroots is built
pub fn ensure_built(arch: &str) -> Result<()> {
    if !exists(arch) {
        build(arch)?;
    }
    Ok(())
}

/// Require wlroots to exist
pub fn require(arch: &str) -> Result<PathBuf> {
    let path = output_dir(arch);
    if !exists(arch) {
        bail!(
            "wlroots not found at {}.\nRun 'cargo run -- build wlroots' first.",
            path.display()
        );
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths() {
        assert_eq!(clone_dir(), PathBuf::from("toolchain/wlroots"));
        assert_eq!(
            output_dir("x86_64"),
            PathBuf::from("toolchain/wlroots-out/x86_64")
        );
    }
}
