//! sway build support
//!
//! sway is a tiling Wayland compositor compatible with i3.
//! It uses wlroots as its compositor library.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

use super::wlroots;

/// Git repository URL for sway
pub const REPO: &str = "https://github.com/swaywm/sway.git";

/// Version tag to build
pub const VERSION: &str = "1.10.1";

/// Get the clone directory for sway source
pub fn clone_dir() -> PathBuf {
    PathBuf::from("toolchain/sway")
}

/// Get the output directory for built binaries
pub fn output_dir(arch: &str) -> PathBuf {
    PathBuf::from(format!("toolchain/sway-out/{arch}"))
}

/// Get the path to the sway binary
pub fn binary_path(arch: &str) -> PathBuf {
    output_dir(arch).join("bin/sway")
}

/// Check if sway has been built for the given architecture
pub fn exists(arch: &str) -> bool {
    binary_path(arch).exists()
}

/// Clone the sway repository if not present
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

    println!("  Cloning sway {}...", VERSION);
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
        .context("Failed to clone sway")?;

    if !status.success() {
        bail!("Failed to clone sway from {REPO}");
    }

    Ok(())
}

/// Build sway using distrobox Alpine
pub fn build(arch: &str) -> Result<()> {
    // Ensure wlroots is built
    wlroots::ensure_built(arch)?;

    // Clone source
    clone_repo()?;

    let src_dir = clone_dir();
    let abs_src = std::fs::canonicalize(&src_dir)
        .context("Failed to get absolute path for sway")?;
    let out_dir = output_dir(arch);
    std::fs::create_dir_all(&out_dir)?;
    let abs_out = std::fs::canonicalize(&out_dir)
        .context("Failed to get absolute path for sway output")?;

    let wlroots_dir = wlroots::output_dir(arch);
    let abs_wlroots = std::fs::canonicalize(&wlroots_dir)
        .context("Failed to get absolute path for wlroots")?;

    println!("  Building sway for {arch}...");

    // Build script for distrobox Alpine
    // Note: Use system packages for dependencies, only need local wlroots
    // TEAM_477: Build against v3.21 to match runtime library versions
    let build_script = format!(
        r#"
set -e

# Ensure we're using Alpine v3.21 repos
if grep -q "/edge/" /etc/apk/repositories 2>/dev/null; then
    echo "  Switching Alpine repos to v3.21..."
    sudo sed -i 's|/edge/|/v3.21/|g' /etc/apk/repositories
    sudo apk update
fi

cd "{src}"

# Add wlroots to pkg-config path (system libs are already available)
export PKG_CONFIG_PATH="{wlroots}/lib/pkgconfig:$PKG_CONFIG_PATH"

# Add wlroots to library path for linking
export LIBRARY_PATH="{wlroots}/lib:$LIBRARY_PATH"
export LD_LIBRARY_PATH="{wlroots}/lib:$LD_LIBRARY_PATH"
export C_INCLUDE_PATH="{wlroots}/include/wlroots-0.18:$C_INCLUDE_PATH"

# Clean previous build
rm -rf build

echo "  Configuring sway..."
meson setup build \
    --prefix="{out}" \
    --buildtype=release \
    -Dman-pages=disabled \
    -Dgdk-pixbuf=disabled \
    -Dsd-bus-provider=auto \
    -Dtray=disabled \
    -Dbash-completions=false \
    -Dzsh-completions=false \
    -Dfish-completions=false \
    -Ddefault-wallpaper=false

echo "  Compiling sway..."
ninja -C build

echo "  Installing sway..."
ninja -C build install

echo "  sway build complete"
"#,
        src = abs_src.display(),
        out = abs_out.display(),
        wlroots = abs_wlroots.display(),
    );

    let status = Command::new("distrobox")
        .args(["enter", "Alpine", "--"])
        .arg("sh")
        .arg("-c")
        .arg(&build_script)
        .status()
        .context("Failed to build sway")?;

    if !status.success() {
        bail!("sway build failed");
    }

    // Verify build
    if !exists(arch) {
        bail!("sway binary not found after build");
    }

    // Show binary info
    let metadata = std::fs::metadata(binary_path(arch))?;
    let size_kb = metadata.len() as f64 / 1024.0;
    println!("  sway built: {} ({:.1} KB)", binary_path(arch).display(), size_kb);

    Ok(())
}

/// Ensure sway is built
pub fn ensure_built(arch: &str) -> Result<()> {
    if !exists(arch) {
        build(arch)?;
    }
    Ok(())
}

/// Require sway to exist
pub fn require(arch: &str) -> Result<PathBuf> {
    let path = binary_path(arch);
    if !exists(arch) {
        bail!(
            "sway not found at {}.\nRun 'cargo run -- build sway' first.",
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
        assert_eq!(clone_dir(), PathBuf::from("toolchain/sway"));
        assert_eq!(
            binary_path("x86_64"),
            PathBuf::from("toolchain/sway-out/x86_64/bin/sway")
        );
    }
}
