use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::process::Command;

// TEAM_326: Renamed from ImageCommands to DiskCommands
#[derive(Subcommand)]
pub enum DiskCommands {
    /// Create/Format the disk image
    Create,
    /// Install userspace apps to disk
    Install,
    /// Show disk image status and contents
    Status,
}

pub fn create_disk_image_if_missing() -> Result<()> {
    let disk_path = "tinyos_disk.img";
    if !std::path::Path::new(disk_path).exists() {
        println!("💿 Creating default 16MB FAT32 disk image ({disk_path})...");

        // Create blank file
        let status = Command::new("dd")
            .args([
                "if=/dev/zero",
                &format!("of={disk_path}"),
                "bs=1M",
                "count=16",
            ])
            .status()
            .context("Failed to run dd")?;
        if !status.success() {
            bail!("dd failed to create disk image");
        }

        // Create MBR partition table
        let status = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "echo 'label: dos\nstart=2048, type=b' | sfdisk {disk_path}"
            ))
            .status()
            .context("Failed to run sfdisk")?;
        if !status.success() {
            bail!("sfdisk failed to partition disk image");
        }

        // Format partition 1 as FAT32
        let status = Command::new("mformat")
            .args(["-i", &format!("{disk_path}@@1M"), "-F", "-v", "LEVITATE"])
            .status()
            .context("Failed to run mformat")?;
        if !status.success() {
            bail!("mformat failed to format partition");
        }
    }
    Ok(())
}

/// TEAM_476: This function is a no-op now.
/// The custom userspace crates were removed. Linux + BusyBox runs from initramfs.
pub fn install_userspace_to_disk(_arch: &str) -> Result<()> {
    create_disk_image_if_missing()?;
    // TEAM_476: No custom binaries to install - we use BusyBox from initramfs
    println!("💿 Disk ready (no custom binaries to install)");
    Ok(())
}

#[allow(dead_code)]
fn install_userspace_to_disk_legacy(arch: &str) -> Result<()> {
    create_disk_image_if_missing()?;

    // TEAM_121: Legacy - Install/Update userspace apps on the disk
    let binaries: Vec<String> = vec![]; // Was: crate::get_binaries(arch)?
    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {arch}"),
    };
    print!(
        "💿 Installing userspace apps to disk ({} binaries) for {}... ",
        binaries.len(),
        arch
    );
    let disk_path = "tinyos_disk.img";
    let mut count = 0;
    for bin in binaries {
        let src = format!("crates/userspace/target/{target}/release/{bin}");
        if std::path::Path::new(&src).exists() {
            let status = Command::new("mcopy")
                .args([
                    "-i",
                    &format!("{disk_path}@@1M"),
                    "-o",
                    &src,
                    &format!("::/{bin}"),
                ])
                .status()
                .context(format!("Failed to copy {bin} to disk"))?;
            if status.success() {
                count += 1;
            }
        }
    }
    println!("[DONE] ({count} installed)");

    Ok(())
}

/// `TEAM_116`: Show disk image status and list contents
pub fn show_disk_status() -> Result<()> {
    let disk_path = "tinyos_disk.img";
    if !std::path::Path::new(disk_path).exists() {
        println!("❌ Disk image {disk_path} not found.");
        return Ok(());
    }

    let metadata = std::fs::metadata(disk_path)?;
    println!("💾 Disk Image: {disk_path}");
    println!(
        "   Size: {} bytes ({:.2} MB)",
        metadata.len(),
        metadata.len() as f64 / 1024.0 / 1024.0
    );

    println!("\n📂 Contents (FAT32 partition):");
    let status = Command::new("mdir")
        .args(["-i", &format!("{disk_path}@@1M"), "::/"])
        .status()
        .context("Failed to run mdir")?;

    if !status.success() {
        println!("   (Failed to list contents - partition might be unformatted)");
    }

    Ok(())
}
