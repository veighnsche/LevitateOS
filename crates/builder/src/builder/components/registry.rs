//! Component registry - single source of truth for all buildable components.
//!
//! Add new components here. Everything else (CLI, build_all, initramfs) uses this.

use anyhow::Result;
use std::path::Path;
use std::process::Command;

use super::{brush, linux, sudo_rs, systemd, util_linux};
use crate::builder::vendor;

/// A buildable component.
pub struct Component {
    /// Component name (used for CLI command)
    pub name: &'static str,
    /// Vendor directory name (defaults to name if None)
    pub vendor_dir: Option<&'static str>,
    /// Build function
    pub build: fn() -> Result<()>,
    /// Binaries to copy to initramfs: (source, dest)
    pub binaries: &'static [(&'static str, &'static str)],
    /// Symlinks to create: (link_name, target) - created in /bin
    pub symlinks: &'static [(&'static str, &'static str)],
    /// Runtime directories to copy: (source, dest)
    pub runtime_dirs: &'static [(&'static str, &'static str)],
}

/// All registered components.
/// Order matters for build_all - dependencies should come first.
pub static COMPONENTS: &[Component] = &[
    // Kernel (no binaries copied to initramfs directly)
    Component {
        name: "linux",
        vendor_dir: None,
        build: linux::build,
        binaries: &[],
        symlinks: &[],
        runtime_dirs: &[],
    },
    // Init system
    Component {
        name: "systemd",
        vendor_dir: None,
        build: systemd::build,
        binaries: &[
            ("vendor/systemd/build/systemd", "sbin/init"),
            ("vendor/systemd/build/systemd-executor", "usr/lib/systemd/systemd-executor"),
        ],
        symlinks: &[],
        runtime_dirs: &[],
    },
    // Login utilities
    Component {
        name: "util-linux",
        vendor_dir: Some("util-linux"),
        build: util_linux::build,
        binaries: &[
            ("vendor/util-linux/build/agetty", "sbin/agetty"),
            ("vendor/util-linux/build/login", "bin/login"),
            ("vendor/util-linux/build/sulogin", "sbin/sulogin"),
            ("vendor/util-linux/build/nologin", "sbin/nologin"),
            ("vendor/util-linux/build/fdisk", "sbin/fdisk"),
            ("vendor/util-linux/build/sfdisk", "sbin/sfdisk"),
            ("vendor/util-linux/build/mkfs", "sbin/mkfs"),
            ("vendor/util-linux/build/blkid", "sbin/blkid"),
            ("vendor/util-linux/build/lsblk", "bin/lsblk"),
            ("vendor/util-linux/build/mount", "bin/mount"),
            ("vendor/util-linux/build/umount", "bin/umount"),
            ("vendor/util-linux/build/losetup", "sbin/losetup"),
        ],
        symlinks: &[],
        runtime_dirs: &[],
    },
    // Coreutils
    Component {
        name: "uutils",
        vendor_dir: None,
        build: build_uutils,
        binaries: &[
            ("vendor/uutils/target/release/coreutils", "bin/coreutils"),
        ],
        symlinks: &[
            // Symlinks from coreutils multicall binary
            ("ls", "coreutils"), ("cat", "coreutils"), ("cp", "coreutils"),
            ("mv", "coreutils"), ("rm", "coreutils"), ("mkdir", "coreutils"),
            ("chmod", "coreutils"), ("chown", "coreutils"), ("ln", "coreutils"),
            ("echo", "coreutils"), ("env", "coreutils"), ("pwd", "coreutils"),
            ("head", "coreutils"), ("tail", "coreutils"), ("wc", "coreutils"),
            ("sort", "coreutils"), ("uniq", "coreutils"), ("tr", "coreutils"),
            ("cut", "coreutils"), ("grep", "coreutils"), ("test", "coreutils"),
            ("[", "coreutils"), ("true", "coreutils"), ("false", "coreutils"),
            ("sleep", "coreutils"), ("date", "coreutils"), ("uname", "coreutils"),
            ("id", "coreutils"), ("whoami", "coreutils"), ("basename", "coreutils"),
            ("dirname", "coreutils"), ("touch", "coreutils"), ("rmdir", "coreutils"),
            ("readlink", "coreutils"), ("realpath", "coreutils"),
        ],
        runtime_dirs: &[],
    },
    // Findutils
    Component {
        name: "findutils",
        vendor_dir: None,
        build: build_findutils,
        binaries: &[
            ("vendor/findutils/target/release/find", "bin/find"),
            ("vendor/findutils/target/release/xargs", "bin/xargs"),
        ],
        symlinks: &[],
        runtime_dirs: &[],
    },
    // Diffutils
    Component {
        name: "diffutils",
        vendor_dir: None,
        build: build_diffutils,
        binaries: &[
            ("vendor/diffutils/target/release/diff", "bin/diff"),
            ("vendor/diffutils/target/release/cmp", "bin/cmp"),
        ],
        symlinks: &[],
        runtime_dirs: &[],
    },
    // Sudo
    Component {
        name: "sudo-rs",
        vendor_dir: Some("sudo-rs"),
        build: sudo_rs::build,
        binaries: &[
            ("vendor/sudo-rs/target/release/sudo", "bin/sudo"),
            ("vendor/sudo-rs/target/release/su", "bin/su"),
        ],
        symlinks: &[],
        runtime_dirs: &[],
    },
    // Shell
    Component {
        name: "brush",
        vendor_dir: None,
        build: brush::build,
        binaries: &[
            ("vendor/brush/target/release/brush", "bin/brush"),
        ],
        symlinks: &[
            ("sh", "brush"),
        ],
        runtime_dirs: &[],
    },
    // Editor
    Component {
        name: "helix",
        vendor_dir: None,
        build: build_helix,
        binaries: &[
            ("vendor/helix/target/release/hx", "bin/hx"),
        ],
        symlinks: &[
            ("vi", "hx"),
            ("vim", "hx"),
        ],
        runtime_dirs: &[
            ("vendor/helix/runtime", "usr/share/helix/runtime"),
        ],
    },
];

/// Get component by name.
pub fn get(name: &str) -> Option<&'static Component> {
    COMPONENTS.iter().find(|c| c.name == name)
}

/// List all component names.
pub fn names() -> impl Iterator<Item = &'static str> {
    COMPONENTS.iter().map(|c| c.name)
}

// Build functions for cargo-based projects

fn build_uutils() -> Result<()> {
    use anyhow::Context;
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

fn build_findutils() -> Result<()> {
    println!("=== Building findutils ===");
    let src = vendor::require("findutils")?;
    run_cargo(&src, &["build", "--release"])?;
    println!("  Built: vendor/findutils/target/release/find, xargs");
    Ok(())
}

fn build_diffutils() -> Result<()> {
    println!("=== Building diffutils ===");
    let src = vendor::require("diffutils")?;
    run_cargo(&src, &["build", "--release"])?;
    println!("  Built: vendor/diffutils/target/release/diff, cmp");
    Ok(())
}

fn build_helix() -> Result<()> {
    println!("=== Building helix ===");
    let src = vendor::require("helix")?;
    run_cargo(&src, &["build", "--release"])?;
    println!("  Built: vendor/helix/target/release/hx");
    Ok(())
}

fn run_cargo(dir: &Path, args: &[&str]) -> Result<()> {
    use anyhow::{bail, Context};
    let status = Command::new("cargo")
        .args(args)
        .current_dir(dir)
        .status()
        .context("Failed to run cargo")?;

    if !status.success() {
        bail!("cargo build failed");
    }
    Ok(())
}
