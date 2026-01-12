//! External Rust application registry and build abstraction
//!
//! TEAM_444: Migrated from c-gull to musl libc.
//!
//! All external Rust apps (coreutils, brush, etc.) follow the same pattern:
//! 1. Clone from git if not present
//! 2. Build with --target x86_64-unknown-linux-musl (standard Rust target)
//! 3. Copy output to toolchain/{name}-out/{target}/release/
//!
//! This is much simpler than the old c-gull approach which required:
//! - Custom sysroot build
//! - -Z build-std flags
//! - Complex RUSTFLAGS
//!
//! With musl, we just use standard Rust targets.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// An external Rust application that can be built with musl
#[derive(Debug, Clone)]
pub struct ExternalApp {
    /// Short name (e.g., "coreutils", "brush")
    pub name: &'static str,
    /// Git repository URL
    pub repo: &'static str,
    /// Cargo package name to build (may differ from name)
    pub package: &'static str,
    /// Output binary name
    pub binary: &'static str,
    /// Cargo features to enable (comma-separated, or empty)
    pub features: &'static str,
    /// Whether this app is required for a complete initramfs
    pub required: bool,
    /// Symlinks to create in initramfs (for multi-call binaries like coreutils)
    pub symlinks: &'static [&'static str],
}

/// Registry of all external Rust applications
///
/// TEAM_444: brush removed - it's for far future. Use built-in shell first
/// to verify musl works, then add dash (simpler), then brush (complex).
/// TEAM_459: coreutils no longer required - BusyBox provides all utilities.
pub static APPS: &[ExternalApp] = &[
    ExternalApp {
        name: "coreutils",
        repo: "https://github.com/uutils/coreutils",
        package: "coreutils",
        binary: "coreutils",
        // TEAM_444: With musl, we can potentially enable more features
        // since musl has better libc coverage than c-gull
        features: "cat,echo,head,mkdir,pwd,rm,tail,touch",
        // TEAM_459: Not required - BusyBox provides all utilities now.
        // Can still be built manually with: cargo xtask build coreutils
        required: false,
        symlinks: &["cat", "echo", "head", "mkdir", "pwd", "rm", "tail", "touch"],
    },
    // NOTE: brush removed from default builds - it's complex and for later.
    // Shell progression: built-in shell → dash → brush
];

impl ExternalApp {
    /// Get the clone directory for this app
    pub fn clone_dir(&self) -> PathBuf {
        PathBuf::from(format!("toolchain/{}", self.name))
    }

    /// Get the output directory for built binaries
    pub fn output_dir(&self, arch: &str) -> PathBuf {
        let target = musl_target(arch);
        PathBuf::from(format!("toolchain/{}-out/{}/release", self.name, target))
    }

    /// Get the path to the built binary
    pub fn output_path(&self, arch: &str) -> PathBuf {
        self.output_dir(arch).join(self.binary)
    }

    /// Check if the app has been built for the given architecture
    pub fn exists(&self, arch: &str) -> bool {
        self.output_path(arch).exists()
    }

    /// Clone the repository if not present (idempotent)
    pub fn clone_repo(&self) -> Result<()> {
        let dir = self.clone_dir();
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

        println!("📥 Cloning {}...", self.name);
        let status = Command::new("git")
            .args(["clone", "--depth=1", self.repo, &dir.to_string_lossy()])
            .status()
            .with_context(|| format!("Failed to clone {}", self.name))?;

        if !status.success() {
            bail!("Failed to clone {} from {}", self.name, self.repo);
        }

        Ok(())
    }

    /// Build the app with musl target
    ///
    /// TEAM_444: Simplified from c-gull approach. No more:
    /// - Custom sysroot
    /// - -Z build-std
    /// - Complex RUSTFLAGS
    pub fn build(&self, arch: &str) -> Result<()> {
        // Ensure cloned
        self.clone_repo()?;

        // Ensure musl target is installed
        ensure_musl_target(arch)?;

        let target = musl_target(arch);
        println!("🔧 Building {} for {} (musl)...", self.name, arch);

        // Simple build command - just use the musl target!
        let mut args = vec![
            "build".to_string(),
            "--release".to_string(),
            "--target".to_string(),
            target.to_string(),
            "-p".to_string(),
            self.package.to_string(),
        ];

        if !self.features.is_empty() {
            args.push("--no-default-features".to_string());
            args.push("--features".to_string());
            args.push(self.features.to_string());
        }

        let status = Command::new("cargo")
            .current_dir(self.clone_dir())
            .args(&args)
            .status()
            .with_context(|| format!("Failed to build {}", self.name))?;

        if !status.success() {
            bail!("Failed to build {}", self.name);
        }

        // Copy to output directory
        let src = self
            .clone_dir()
            .join("target")
            .join(target)
            .join("release")
            .join(self.binary);

        let out_dir = self.output_dir(arch);
        std::fs::create_dir_all(&out_dir)?;
        let dst = out_dir.join(self.binary);

        if src.exists() {
            std::fs::copy(&src, &dst)?;
            println!("📦 Built {}: {}", self.name, dst.display());
        } else {
            bail!("{} binary not found at {}", self.name, src.display());
        }

        println!("✅ {} ready", self.name);
        Ok(())
    }

    /// Ensure the app is built, or fail with a clear error
    pub fn require(&self, arch: &str) -> Result<PathBuf> {
        let path = self.output_path(arch);
        if !path.exists() {
            bail!(
                "{} not found at {}.\n\
                 Run 'cargo xtask build {}' first, or use 'cargo xtask build all' to build everything.",
                self.name,
                path.display(),
                self.name
            );
        }
        Ok(path)
    }

    /// Build if not already built (for build all/iso commands)
    pub fn ensure_built(&self, arch: &str) -> Result<()> {
        if !self.exists(arch) {
            self.build(arch)?;
        }
        Ok(())
    }
}

/// Get an app by name
pub fn get_app(name: &str) -> Option<&'static ExternalApp> {
    APPS.iter().find(|app| app.name == name)
}

/// Get all required apps
pub fn required_apps() -> impl Iterator<Item = &'static ExternalApp> {
    APPS.iter().filter(|app| app.required)
}

/// Get all optional apps
#[allow(dead_code)] // Available for future use
pub fn optional_apps() -> impl Iterator<Item = &'static ExternalApp> {
    APPS.iter().filter(|app| !app.required)
}

/// Build all apps that aren't already built (for build all/iso)
/// TEAM_459: Only build required apps (not all apps in registry)
/// Apps like coreutils are optional now that BusyBox is the primary.
pub fn ensure_all_built(arch: &str) -> Result<()> {
    for app in required_apps() {
        app.ensure_built(arch)?;
    }
    Ok(())
}

/// Require all required apps to be built, fail fast if any missing
#[allow(dead_code)] // Available for future use
pub fn require_all(arch: &str) -> Result<()> {
    for app in required_apps() {
        app.require(arch)?;
    }
    Ok(())
}

/// Convert architecture to musl target triple
fn musl_target(arch: &str) -> &'static str {
    match arch {
        "x86_64" => "x86_64-unknown-linux-musl",
        "aarch64" => "aarch64-unknown-linux-musl",
        _ => "x86_64-unknown-linux-musl", // fallback
    }
}

/// Ensure the musl target is installed via rustup
fn ensure_musl_target(arch: &str) -> Result<()> {
    let target = musl_target(arch);

    // Check if target is installed
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .context("Failed to run rustup")?;

    let installed = String::from_utf8_lossy(&output.stdout);
    if installed.contains(target) {
        return Ok(());
    }

    // Install the target
    println!("📥 Installing Rust musl target: {}", target);
    let status = Command::new("rustup")
        .args(["target", "add", target])
        .status()
        .context("Failed to run rustup target add")?;

    if !status.success() {
        bail!(
            "Failed to install {} target.\n\
             Try running: rustup target add {}",
            target,
            target
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_paths() {
        let app = get_app("coreutils").unwrap();
        assert_eq!(app.clone_dir(), PathBuf::from("toolchain/coreutils"));
        // TEAM_444: Now uses musl target
        assert_eq!(
            app.output_path("x86_64"),
            PathBuf::from("toolchain/coreutils-out/x86_64-unknown-linux-musl/release/coreutils")
        );
    }

    #[test]
    fn test_required_apps() {
        let required: Vec<_> = required_apps().collect();
        assert!(required.iter().any(|a| a.name == "coreutils"));
        assert!(!required.iter().any(|a| a.name == "brush"));
    }

    #[test]
    fn test_musl_target() {
        assert_eq!(musl_target("x86_64"), "x86_64-unknown-linux-musl");
        assert_eq!(musl_target("aarch64"), "aarch64-unknown-linux-musl");
    }
}
