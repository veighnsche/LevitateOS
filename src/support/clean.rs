use anyhow::Result;
use std::process::Command;

/// Kill any running QEMU instances
pub fn kill_qemu(arch: &str) -> Result<()> {
    println!("🔪 Killing QEMU instances for {arch}...");
    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => return Ok(()), // Should not happen
    };
    let status = Command::new("pkill").args(["-f", qemu_bin]).status()?;
    if status.success() {
        println!("✅ QEMU processes killed.");
    } else {
        println!("ℹ️  No QEMU processes found.");
    }
    // Also kill websockify if running
    let _ = Command::new("pkill").args(["-f", "websockify"]).status();
    // Remove QMP socket
    if std::path::Path::new("./qmp.sock").exists() {
        let _ = std::fs::remove_file("./qmp.sock");
        println!("✅ Removed qmp.sock");
    }
    Ok(())
}

pub fn clean(arch: &str) -> Result<()> {
    println!("🧹 Cleaning for {arch}...");
    kill_qemu(arch)?;

    // TEAM_294: Remove generated artifacts and staging directories
    // TEAM_327: Added arch-specific initramfs files
    let artifacts = [
        "initramfs_aarch64.cpio",
        "initramfs_x86_64.cpio",
        "initramfs_test.cpio",
        "tinyos_disk.img",
        "levitate.iso",
        "kernel64_rust.bin",
    ];
    for artifact in artifacts {
        if std::path::Path::new(artifact).exists() {
            let _ = std::fs::remove_file(artifact);
            println!("✅ Removed {artifact}");
        }
    }

    let dirs = ["initrd_root", "initrd_test_root", "iso_root", "limine-bin"];
    for dir in dirs {
        if std::path::Path::new(dir).exists() {
            let _ = std::fs::remove_dir_all(dir);
            println!("✅ Removed directory {dir}");
        }
    }

    Ok(())
}
