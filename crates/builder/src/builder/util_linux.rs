//! util-linux builder (agetty, login, disk utilities).

use crate::builder::vendor;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Build util-linux.
pub fn build() -> Result<()> {
    println!("=== Building util-linux ===");

    let src = vendor::require("util-linux")?;
    let build_dir = src.join("build");

    std::fs::create_dir_all(&build_dir)?;

    // Meson setup - enable only what we need
    let base_args = vec![
        // Login utilities
        "-Dbuild-agetty=enabled",
        "-Dbuild-login=enabled",
        "-Dbuild-nologin=enabled",
        // Disk utilities
        "-Dbuild-fdisks=enabled",    // fdisk, sfdisk, cfdisk
        "-Dbuild-mkfs=enabled",
        "-Dbuild-libblkid=enabled",
        "-Dbuild-lsblk=enabled",
        "-Dbuild-mount=enabled",
        "-Dbuild-losetup=enabled",
        "-Dbuild-libmount=enabled",
        "-Dbuild-libsmartcols=enabled",
        "-Dbuild-libfdisk=enabled",
        // Disable optional dependencies (leave ncurses/readline as auto)
        "-Dlibuser=disabled",
        "-Deconf=disabled",
        "-Dbuild-liblastlog2=disabled",
        "-Daudit=disabled",
        "-Dselinux=disabled",
        // Disable su - use sudo-rs instead
        "-Dbuild-su=disabled",
        "-Dbuild-runuser=disabled",
        // Disable other unused utilities
        "-Dbuild-chfn-chsh=disabled",
        "-Dbuild-newgrp=disabled",
        "-Dbuild-python=disabled",
        "-Dbuild-bash-completion=disabled",
    ];

    let meson_args = if build_dir.join("build.ninja").exists() {
        let mut args = vec!["setup", "build", ".", "--reconfigure"];
        args.extend(base_args.iter());
        args
    } else {
        let mut args = vec!["setup", "build", "."];
        args.extend(base_args.iter());
        args
    };

    run_cmd("meson", &meson_args, &src)?;
    run_cmd("ninja", &["-C", "build"], &src)?;

    println!("  Built: vendor/util-linux/build/");
    Ok(())
}

/// Get path to agetty binary.
pub fn agetty_path() -> &'static str {
    "vendor/util-linux/build/agetty"
}

/// Get path to login binary.
pub fn login_path() -> &'static str {
    "vendor/util-linux/build/login"
}

/// Get path to sulogin binary.
pub fn sulogin_path() -> &'static str {
    "vendor/util-linux/build/sulogin"
}

/// Get path to nologin binary.
pub fn nologin_path() -> &'static str {
    "vendor/util-linux/build/nologin"
}

/// Get path to fdisk binary.
pub fn fdisk_path() -> &'static str {
    "vendor/util-linux/build/fdisk"
}

/// Get path to sfdisk binary.
pub fn sfdisk_path() -> &'static str {
    "vendor/util-linux/build/sfdisk"
}

/// Get path to mkfs binary.
pub fn mkfs_path() -> &'static str {
    "vendor/util-linux/build/mkfs"
}

/// Get path to blkid binary.
pub fn blkid_path() -> &'static str {
    "vendor/util-linux/build/blkid"
}

/// Get path to lsblk binary.
pub fn lsblk_path() -> &'static str {
    "vendor/util-linux/build/lsblk"
}

/// Get path to mount binary.
pub fn mount_path() -> &'static str {
    "vendor/util-linux/build/mount"
}

/// Get path to umount binary.
pub fn umount_path() -> &'static str {
    "vendor/util-linux/build/umount"
}

/// Get path to losetup binary.
pub fn losetup_path() -> &'static str {
    "vendor/util-linux/build/losetup"
}

/// Get paths to util-linux shared libraries.
pub fn lib_paths() -> Vec<&'static str> {
    vec![
        "vendor/util-linux/build/libblkid.so.1",
        "vendor/util-linux/build/libmount.so.1",
        "vendor/util-linux/build/libsmartcols.so.1",
        "vendor/util-linux/build/libfdisk.so.1",
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
