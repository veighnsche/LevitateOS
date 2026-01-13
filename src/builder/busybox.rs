#![allow(dead_code)]
//! `BusyBox` build support
//!
//! `TEAM_451`: Single binary replaces coreutils + dash + custom init
//!
//! `BusyBox` provides:
//! - Init system
//! - Shell (ash)
//! - 300+ utilities (coreutils, grep, sed, awk, vi, etc.)
//!
//! Built statically with musl for `LevitateOS`.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Git repository URL for `BusyBox`
pub const REPO: &str = "https://git.busybox.net/busybox";

/// Get the clone directory for `BusyBox` source
pub fn clone_dir() -> PathBuf {
    PathBuf::from("toolchain/busybox")
}

/// Get the output directory for built binaries
pub fn output_dir(arch: &str) -> PathBuf {
    PathBuf::from(format!("toolchain/busybox-out/{arch}"))
}

/// Get the path to the built `BusyBox` binary
pub fn output_path(arch: &str) -> PathBuf {
    output_dir(arch).join("busybox")
}

/// Check if `BusyBox` has been built for the given architecture
pub fn exists(arch: &str) -> bool {
    output_path(arch).exists()
}

/// Clone the `BusyBox` repository if not present (idempotent)
pub fn clone_repo() -> Result<()> {
    let dir = clone_dir();
    if dir.exists() {
        // Validate it's a git repo
        if !dir.join(".git").exists() {
            bail!(
                "Directory {} exists but is not a git repository. \
                 Remove it and try again.",
                dir.display()
            );
        }
        return Ok(());
    }

    println!("📥 Cloning BusyBox...");
    let status = Command::new("git")
        .args(["clone", "--depth=1", REPO, &dir.to_string_lossy()])
        .status()
        .context("Failed to clone BusyBox")?;

    if !status.success() {
        bail!("Failed to clone BusyBox from {REPO}");
    }

    Ok(())
}

/// Build `BusyBox` - tries native build first, falls back to distrobox
/// `TEAM_451`: Alpine is built on musl, perfect for static `BusyBox` builds
/// `TEAM_452`: Fixed DNS issue that was blocking container networking
/// `TEAM_459`: Added cross-compilation support for aarch64
/// `TEAM_470`: Added native build support for CI (no distrobox required)
pub fn build(arch: &str) -> Result<()> {
    // Try native build first (works in CI without distrobox)
    if can_build_native(arch) {
        return build_native(arch);
    }

    // Fall back to distrobox build
    build_distrobox(arch)
}

/// Check if we can build natively without distrobox
fn can_build_native(arch: &str) -> bool {
    match arch {
        "x86_64" => musl_gcc_available(),
        "aarch64" => true, // We can download the cross-compiler
        _ => false,
    }
}

/// Build `BusyBox` natively without containers (CI-compatible)
/// `TEAM_470`: Added for GitHub Actions compatibility
fn build_native(arch: &str) -> Result<()> {
    clone_repo()?;

    let dir = clone_dir();
    let abs_dir =
        std::fs::canonicalize(&dir).context("Failed to get absolute path for BusyBox dir")?;

    // For aarch64, download the cross-compiler
    if arch == "aarch64" {
        setup_aarch64_cross_compiler()?;
    }

    let cross_note = if arch == "aarch64" {
        "cross-compiling for aarch64"
    } else {
        "native musl build"
    };

    println!("🔨 Building BusyBox natively ({cross_note})...");

    // Clean previous build
    let _ = Command::new("make")
        .current_dir(&abs_dir)
        .arg("clean")
        .status();

    // Configure
    println!("⚙️  Configuring BusyBox...");
    let defconfig_arch = if arch == "aarch64" { "arm64" } else { "x86_64" };
    let status = Command::new("make")
        .current_dir(&abs_dir)
        .arg(format!("ARCH={defconfig_arch}"))
        .arg("defconfig")
        .status()
        .context("Failed to run make defconfig")?;

    if !status.success() {
        bail!("BusyBox defconfig failed");
    }

    // Apply musl-compatible configuration
    apply_musl_config(&dir, arch)?;

    // Build
    println!("🔨 Compiling BusyBox...");
    let mut cmd = Command::new("sh");
    cmd.current_dir(&abs_dir);

    let build_cmd = if arch == "aarch64" {
        let toolchain = PathBuf::from("toolchain/aarch64-linux-musl-cross");
        let abs_toolchain = std::fs::canonicalize(&toolchain)?;
        format!(
            "export PATH=\"{}/bin:$PATH\" && make ARCH=arm64 CROSS_COMPILE=aarch64-linux-musl- LDFLAGS=-static -j{}",
            abs_toolchain.display(),
            num_cpus()
        )
    } else {
        format!("make CC=musl-gcc LDFLAGS=-static -j{}", num_cpus())
    };

    let status = cmd
        .arg("-c")
        .arg(&build_cmd)
        .status()
        .context("Failed to build BusyBox")?;

    if !status.success() {
        bail!("BusyBox build failed");
    }

    // Verify the binary was created
    let built_binary = dir.join("busybox");
    if !built_binary.exists() {
        bail!("BusyBox binary not found after build");
    }

    // Verify it's statically linked
    let file_output = Command::new("file")
        .arg(&built_binary)
        .output()
        .context("Failed to run file command")?;

    let file_info = String::from_utf8_lossy(&file_output.stdout);
    if !file_info.contains("statically linked") {
        println!("⚠️  Warning: BusyBox may not be statically linked");
        println!("    file output: {}", file_info.trim());
    }

    // Copy to output directory
    let out_dir = output_dir(arch);
    std::fs::create_dir_all(&out_dir)?;
    let dst = output_path(arch);
    std::fs::copy(&built_binary, &dst)?;

    // Show binary size
    let metadata = std::fs::metadata(&dst)?;
    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
    println!("✅ BusyBox built: {} ({:.2} MB)", dst.display(), size_mb);

    Ok(())
}

/// Build `BusyBox` using distrobox (Alpine container - native musl environment)
fn build_distrobox(arch: &str) -> Result<()> {
    // Ensure cloned
    clone_repo()?;

    // Check distrobox is available
    ensure_distrobox()?;

    let dir = clone_dir();
    let abs_dir =
        std::fs::canonicalize(&dir).context("Failed to get absolute path for BusyBox dir")?;

    // TEAM_459: For aarch64, ensure cross-compiler is available
    if arch == "aarch64" {
        setup_aarch64_cross_compiler()?;
    }

    let cross_note = if arch == "aarch64" {
        "cross-compiling for aarch64"
    } else {
        "native build"
    };

    println!("🐳 Building BusyBox via distrobox (Alpine, {cross_note})...");

    // Build script to run inside Alpine distrobox
    // Alpine is musl-native, so gcc IS musl-gcc
    let build_script = if arch == "aarch64" {
        // For aarch64, we need to use the downloaded cross-compiler
        let toolchain_dir = PathBuf::from("toolchain/aarch64-linux-musl-cross");
        let abs_toolchain = std::fs::canonicalize(&toolchain_dir)
            .context("Failed to get absolute path for toolchain")?;
        format!(
            r#"
set -e
cd "{abs_dir}"
export PATH="{toolchain}/bin:$PATH"

echo "🧹 Cleaning BusyBox..."
make clean 2>/dev/null || true

echo "⚙️  Configuring BusyBox..."
make ARCH=arm64 defconfig

# Enable static linking
sed -i 's/# CONFIG_STATIC is not set/CONFIG_STATIC=y/' .config
# Disable PIE for static
sed -i 's/CONFIG_PIE=y/# CONFIG_PIE is not set/' .config

echo "🔨 Building BusyBox (cross-compiling for aarch64)..."
make ARCH=arm64 CROSS_COMPILE=aarch64-linux-musl- LDFLAGS=-static -j$(nproc)

echo "✅ BusyBox build complete"
"#,
            abs_dir = abs_dir.display(),
            toolchain = abs_toolchain.display()
        )
    } else {
        format!(
            r#"
set -e
cd "{abs_dir}"

echo "🧹 Cleaning BusyBox..."
make clean 2>/dev/null || true

echo "⚙️  Configuring BusyBox..."
make ARCH=x86_64 defconfig

# Enable static linking
sed -i 's/# CONFIG_STATIC is not set/CONFIG_STATIC=y/' .config
# Disable PIE for static
sed -i 's/CONFIG_PIE=y/# CONFIG_PIE is not set/' .config

echo "🔨 Building BusyBox (native build)..."
make LDFLAGS=-static -j$(nproc)

echo "✅ BusyBox build complete"
"#,
            abs_dir = abs_dir.display()
        )
    };

    let status = Command::new("distrobox")
        .args(["enter", "Alpine", "--"])
        .arg("sh")
        .arg("-c")
        .arg(&build_script)
        .status()
        .context("Failed to build BusyBox")?;

    if !status.success() {
        bail!("BusyBox build failed");
    }

    // Verify the binary was created
    let built_binary = dir.join("busybox");
    if !built_binary.exists() {
        bail!("BusyBox binary not found after build");
    }

    // Verify it's statically linked
    let file_output = Command::new("file")
        .arg(&built_binary)
        .output()
        .context("Failed to run file command")?;

    let file_info = String::from_utf8_lossy(&file_output.stdout);
    if !file_info.contains("statically linked") {
        println!("⚠️  Warning: BusyBox may not be statically linked");
        println!("    file output: {}", file_info.trim());
    }

    // Copy to output directory
    let out_dir = output_dir(arch);
    std::fs::create_dir_all(&out_dir)?;
    let dst = output_path(arch);
    std::fs::copy(&built_binary, &dst)?;

    // Show binary size
    let metadata = std::fs::metadata(&dst)?;
    let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
    println!("✅ BusyBox built: {} ({:.2} MB)", dst.display(), size_mb);

    Ok(())
}

/// Apply musl-compatible configuration overrides
/// Based on Alpine Linux's approach (see .external-kernels/alpine-aports/main/busybox/)
fn apply_musl_config(dir: &PathBuf, _arch: &str) -> Result<()> {
    let config_path = dir.join(".config");
    let config = std::fs::read_to_string(&config_path).context("Failed to read .config")?;

    // TEAM_451: Enable static linking and disable musl-incompatible features
    // Based on Alpine's busyboxconfig with modifications for static build
    let config = config
        // Enable static linking (like Alpine's static build)
        .replace("# CONFIG_STATIC is not set", "CONFIG_STATIC=y")
        // Disable PIE for static build (Alpine does this for static)
        .replace("CONFIG_PIE=y", "# CONFIG_PIE is not set")
        // Disable musl-incompatible features
        .replace("CONFIG_SELINUX=y", "# CONFIG_SELINUX is not set")
        .replace(
            "CONFIG_FEATURE_HAVE_RPC=y",
            "# CONFIG_FEATURE_HAVE_RPC is not set",
        )
        .replace(
            "CONFIG_FEATURE_MOUNT_NFS=y",
            "# CONFIG_FEATURE_MOUNT_NFS is not set",
        )
        .replace(
            "CONFIG_FEATURE_INETD_RPC=y",
            "# CONFIG_FEATURE_INETD_RPC is not set",
        )
        .replace("CONFIG_PAM=y", "# CONFIG_PAM is not set")
        .replace(
            "CONFIG_FEATURE_SYSTEMD=y",
            "# CONFIG_FEATURE_SYSTEMD is not set",
        )
        // Disable networking utilities that need kernel headers musl doesn't have
        .replace("CONFIG_TC=y", "# CONFIG_TC is not set")
        .replace(
            "CONFIG_FEATURE_TC_INGRESS=y",
            "# CONFIG_FEATURE_TC_INGRESS is not set",
        )
        // Disable other problematic features
        .replace("CONFIG_NSENTER=y", "# CONFIG_NSENTER is not set")
        .replace("CONFIG_UNSHARE=y", "# CONFIG_UNSHARE is not set")
        // Disable console-tools that need linux/kd.h (not in musl)
        .replace("CONFIG_KBD_MODE=y", "# CONFIG_KBD_MODE is not set")
        .replace("CONFIG_LOADFONT=y", "# CONFIG_LOADFONT is not set")
        .replace("CONFIG_SETFONT=y", "# CONFIG_SETFONT is not set")
        .replace("CONFIG_LOADKMAP=y", "# CONFIG_LOADKMAP is not set")
        .replace("CONFIG_SETKEYCODES=y", "# CONFIG_SETKEYCODES is not set")
        .replace("CONFIG_SHOWKEY=y", "# CONFIG_SHOWKEY is not set")
        .replace("CONFIG_FGCONSOLE=y", "# CONFIG_FGCONSOLE is not set")
        .replace("CONFIG_CHVT=y", "# CONFIG_CHVT is not set")
        .replace("CONFIG_DEALLOCVT=y", "# CONFIG_DEALLOCVT is not set")
        .replace("CONFIG_DUMPKMAP=y", "# CONFIG_DUMPKMAP is not set")
        .replace("CONFIG_OPENVT=y", "# CONFIG_OPENVT is not set")
        .replace("CONFIG_SETCONSOLE=y", "# CONFIG_SETCONSOLE is not set")
        .replace("CONFIG_SETLOGCONS=y", "# CONFIG_SETLOGCONS is not set")
        // Ensure CROSS_COMPILER_PREFIX is empty - we pass CC directly
        // Setting a prefix makes BusyBox look for musl-ar, musl-ld etc which don't exist
        .replace(
            "CONFIG_CROSS_COMPILER_PREFIX=\"musl-\"",
            "CONFIG_CROSS_COMPILER_PREFIX=\"\"",
        );

    std::fs::write(&config_path, config).context("Failed to write .config")?;

    Ok(())
}

/// Get number of CPUs for parallel build
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(4)
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

/// Check if musl-gcc is available (non-failing version)
pub fn musl_gcc_available() -> bool {
    Command::new("musl-gcc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `TEAM_459`: Download and setup the aarch64-linux-musl cross-compiler from musl.cc
/// This provides a complete toolchain for cross-compiling to aarch64 with musl libc
/// Used by both `BusyBox` (C) and coreutils (Rust with musl target)
pub fn setup_aarch64_cross_compiler() -> Result<PathBuf> {
    let toolchain_dir = PathBuf::from("toolchain/aarch64-linux-musl-cross");

    // Check if already extracted
    if toolchain_dir.join("bin/aarch64-linux-musl-gcc").exists() {
        return Ok(toolchain_dir);
    }

    println!("📥 Downloading aarch64-linux-musl cross-compiler...");

    let tarball_path = PathBuf::from("toolchain/aarch64-linux-musl-cross.tgz");
    std::fs::create_dir_all("toolchain")?;

    // Download from musl.cc - prebuilt cross-compilers
    // Using the native x86_64 host version
    let url = "https://musl.cc/aarch64-linux-musl-cross.tgz";

    if !tarball_path.exists() {
        let status = Command::new("curl")
            .args(["-L", "-f", "-o", tarball_path.to_str().unwrap_or(""), url])
            .status()
            .context("Failed to download cross-compiler")?;

        if !status.success() {
            bail!("Failed to download aarch64-linux-musl cross-compiler from {url}");
        }
    }

    println!("📦 Extracting cross-compiler...");

    // Extract to toolchain directory
    let status = Command::new("tar")
        .args([
            "-xzf",
            tarball_path.to_str().unwrap_or(""),
            "-C",
            "toolchain",
        ])
        .status()
        .context("Failed to extract cross-compiler")?;

    if !status.success() {
        bail!("Failed to extract cross-compiler tarball");
    }

    // Verify extraction
    if !toolchain_dir.join("bin/aarch64-linux-musl-gcc").exists() {
        bail!("Cross-compiler extraction failed - gcc not found");
    }

    println!("✅ aarch64-linux-musl cross-compiler ready");
    Ok(toolchain_dir)
}

/// Check if distrobox is available and Alpine container exists
fn ensure_distrobox() -> Result<()> {
    // Check distrobox command exists
    let output = Command::new("distrobox").arg("--version").output();

    if output.is_err() || !output.unwrap().status.success() {
        bail!(
            "distrobox not found.\n\n\
             Install distrobox: https://github.com/89luca89/distrobox\n\
             Fedora: sudo dnf install distrobox\n\
             Or:     curl -s https://raw.githubusercontent.com/89luca89/distrobox/main/install | sudo sh"
        );
    }

    // Check Alpine container exists
    let output = Command::new("distrobox")
        .args(["list", "--no-color"])
        .output()
        .context("Failed to list distrobox containers")?;

    let list = String::from_utf8_lossy(&output.stdout);
    if !list.contains("Alpine") {
        bail!(
            "Alpine distrobox container not found.\n\n\
             Create it with: distrobox create --name Alpine --image alpine:3.20\n\
             Then install build tools: distrobox enter Alpine -- sudo apk add build-base linux-headers perl"
        );
    }

    Ok(())
}

/// Ensure `BusyBox` is built, building if necessary
pub fn ensure_built(arch: &str) -> Result<()> {
    if !exists(arch) {
        build(arch)?;
    }
    Ok(())
}

/// Require `BusyBox` to exist, returning path or error with helpful message
pub fn require(arch: &str) -> Result<PathBuf> {
    let path = output_path(arch);
    if !path.exists() {
        bail!(
            "BusyBox not found at {}.\n\
             Run 'cargo xtask build busybox' first.",
            path.display()
        );
    }
    Ok(path)
}

/// List of applets to create symlinks for in initramfs
/// Returns (name, directory) tuples - "bin" or "sbin"
pub fn applets() -> &'static [(&'static str, &'static str)] {
    &[
        // Init system (sbin)
        ("init", "sbin"),
        ("halt", "sbin"),
        ("poweroff", "sbin"),
        ("reboot", "sbin"),
        // Shell (bin)
        ("sh", "bin"),
        ("ash", "bin"),
        // Coreutils (bin)
        ("cat", "bin"),
        ("cp", "bin"),
        ("echo", "bin"),
        ("ls", "bin"),
        ("mkdir", "bin"),
        ("mv", "bin"),
        ("pwd", "bin"),
        ("rm", "bin"),
        ("rmdir", "bin"),
        ("touch", "bin"),
        ("ln", "bin"),
        ("chmod", "bin"),
        ("chown", "bin"),
        ("head", "bin"),
        ("tail", "bin"),
        ("true", "bin"),
        ("false", "bin"),
        ("test", "bin"),
        ("[", "bin"),
        ("stat", "bin"),
        ("wc", "bin"),
        // Text processing (bin)
        ("grep", "bin"),
        ("sed", "bin"),
        ("awk", "bin"),
        ("sort", "bin"),
        ("uniq", "bin"),
        ("cut", "bin"),
        ("tr", "bin"),
        ("tee", "bin"),
        // Search (bin)
        ("find", "bin"),
        ("xargs", "bin"),
        ("which", "bin"),
        // Archives (bin)
        ("tar", "bin"),
        ("gzip", "bin"),
        ("gunzip", "bin"),
        ("zcat", "bin"),
        // Editor (bin)
        ("vi", "bin"),
        // Process (bin)
        ("ps", "bin"),
        ("kill", "bin"),
        ("killall", "bin"),
        ("sleep", "bin"),
        // Filesystem (bin)
        ("mount", "bin"),
        ("umount", "bin"),
        ("df", "bin"),
        ("du", "bin"),
        // Misc (bin)
        ("date", "bin"),
        ("clear", "bin"),
        ("reset", "bin"),
        ("env", "bin"),
        ("printenv", "bin"),
        ("uname", "bin"),
        ("hostname", "bin"),
        ("id", "bin"),
        ("whoami", "bin"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_busybox_paths() {
        assert_eq!(clone_dir(), PathBuf::from("toolchain/busybox"));
        assert_eq!(
            output_path("x86_64"),
            PathBuf::from("toolchain/busybox-out/x86_64/busybox")
        );
    }

    #[test]
    fn test_applets_not_empty() {
        assert!(!applets().is_empty());
        // Should have init in sbin
        assert!(applets()
            .iter()
            .any(|(name, dir)| *name == "init" && *dir == "sbin"));
        // Should have sh in bin
        assert!(applets()
            .iter()
            .any(|(name, dir)| *name == "sh" && *dir == "bin"));
    }
}
