//! foot terminal build support
//!
//! foot is a fast, lightweight Wayland-native terminal emulator.
//! Perfect for a minimal Wayland environment.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;


/// Git repository URL for foot
pub const REPO: &str = "https://codeberg.org/dnkl/foot.git";

/// Version tag to build
pub const VERSION: &str = "1.20.2";

/// Get the clone directory for foot source
pub fn clone_dir() -> PathBuf {
    PathBuf::from("toolchain/foot")
}

/// Get the output directory for built binaries
pub fn output_dir(arch: &str) -> PathBuf {
    PathBuf::from(format!("toolchain/foot-out/{arch}"))
}

/// Get the path to the foot binary
pub fn binary_path(arch: &str) -> PathBuf {
    output_dir(arch).join("bin/foot")
}

/// Check if foot has been built for the given architecture
pub fn exists(arch: &str) -> bool {
    binary_path(arch).exists()
}

/// Clone the foot repository if not present
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

    println!("  Cloning foot {}...", VERSION);
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
        .context("Failed to clone foot")?;

    if !status.success() {
        bail!("Failed to clone foot from {REPO}");
    }

    Ok(())
}

/// Build foot using distrobox Alpine
pub fn build(arch: &str) -> Result<()> {
    // Clone source
    clone_repo()?;

    let src_dir = clone_dir();
    let abs_src = std::fs::canonicalize(&src_dir)
        .context("Failed to get absolute path for foot")?;
    let out_dir = output_dir(arch);
    std::fs::create_dir_all(&out_dir)?;
    let abs_out = std::fs::canonicalize(&out_dir)
        .context("Failed to get absolute path for foot output")?;

    println!("  Building foot for {arch}...");

    // Build script for distrobox Alpine
    // foot needs: wayland, fcft (font library), pixman, utf8proc
    // Use system packages from distrobox Alpine
    let build_script = format!(
        r#"
set -e
cd "{src}"

# Install foot-specific dependencies in Alpine
sudo apk add --no-cache fcft-dev utf8proc-dev tllist scdoc || true

# Clean previous build
rm -rf build

echo "  Configuring foot..."
meson setup build \
    --prefix="{out}" \
    --buildtype=release \
    --warnlevel=1 \
    -Dwerror=false \
    -Ddocs=disabled \
    -Dthemes=false \
    -Dterminfo=disabled

echo "  Compiling foot..."
ninja -C build

echo "  Installing foot..."
ninja -C build install

echo "  foot build complete"
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
        .context("Failed to build foot")?;

    if !status.success() {
        bail!("foot build failed");
    }

    // Verify build
    if !exists(arch) {
        bail!("foot binary not found after build");
    }

    // Show binary info
    let metadata = std::fs::metadata(binary_path(arch))?;
    let size_kb = metadata.len() as f64 / 1024.0;
    println!("  foot built: {} ({:.1} KB)", binary_path(arch).display(), size_kb);

    Ok(())
}

/// Ensure foot is built
pub fn ensure_built(arch: &str) -> Result<()> {
    if !exists(arch) {
        build(arch)?;
    }
    Ok(())
}

/// Require foot to exist
pub fn require(arch: &str) -> Result<PathBuf> {
    let path = binary_path(arch);
    if !exists(arch) {
        bail!(
            "foot not found at {}.\nRun 'cargo run -- build foot' first.",
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
        assert_eq!(clone_dir(), PathBuf::from("toolchain/foot"));
        assert_eq!(
            binary_path("x86_64"),
            PathBuf::from("toolchain/foot-out/x86_64/bin/foot")
        );
    }
}
