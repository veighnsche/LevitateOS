//! QEMU virtual machine runner.

use crate::builder::{initramfs, linux};
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Boot LevitateOS in QEMU.
pub fn run() -> Result<()> {
    println!("=== Booting LevitateOS ===\n");

    let kernel = linux::kernel_path();
    let initrd = initramfs::cpio_path();

    if !Path::new(kernel).exists() {
        bail!("Kernel not found. Run: builder linux");
    }
    if !Path::new(initrd).exists() {
        bail!("Initramfs not found. Run: builder initramfs");
    }

    let status = Command::new("qemu-system-x86_64")
        .args([
            "-kernel",
            kernel,
            "-initrd",
            initrd,
            "-append",
            "console=ttyS0 rw quiet",
            "-nographic",
            "-m",
            "512M",
            "-no-reboot",
        ])
        .status()
        .context("Failed to run QEMU")?;

    if !status.success() {
        bail!("QEMU exited with error");
    }

    Ok(())
}
