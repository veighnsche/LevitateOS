//! util-linux builder (agetty, login, disk utilities).

use super::Buildable;
use crate::builder::vendor;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// util-linux component.
pub struct UtilLinux;

impl Buildable for UtilLinux {
    fn name(&self) -> &'static str {
        "util-linux"
    }

    fn build(&self) -> Result<()> {
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
            "-Dbuild-fdisks=enabled",
            "-Dbuild-mkfs=enabled",
            "-Dbuild-libblkid=enabled",
            "-Dbuild-lsblk=enabled",
            "-Dbuild-mount=enabled",
            "-Dbuild-losetup=enabled",
            "-Dbuild-libmount=enabled",
            "-Dbuild-libsmartcols=enabled",
            "-Dbuild-libfdisk=enabled",
            // Disable optional dependencies
            "-Dlibuser=disabled",
            "-Deconf=disabled",
            "-Dbuild-liblastlog2=disabled",
            "-Daudit=disabled",
            "-Dselinux=disabled",
            "-Dbuild-su=disabled",
            "-Dbuild-runuser=disabled",
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

    fn binaries(&self) -> &'static [(&'static str, &'static str)] {
        &[
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
        ]
    }

    fn lib_paths(&self) -> Vec<&'static str> {
        vec![
            "vendor/util-linux/build/libblkid.so.1",
            "vendor/util-linux/build/libmount.so.1",
            "vendor/util-linux/build/libsmartcols.so.1",
            "vendor/util-linux/build/libfdisk.so.1",
        ]
    }
}

fn run_cmd(cmd: &str, args: &[&str], dir: &Path) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status()
        .context(format!("Failed to run {cmd}"))?;

    if !status.success() {
        bail!("{cmd} failed");
    }
    Ok(())
}
