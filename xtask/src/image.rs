use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::process::Command;


#[derive(Subcommand)]
pub enum ImageCommands {
    /// Create/Format the disk image
    Create,
    /// Install userspace apps to disk
    Install,
    /// Dump framebuffer to file (QMP)
    Screenshot {
        #[arg(default_value = "screenshot.png")]
        output: String,
    },
}

pub fn create_disk_image_if_missing() -> Result<()> {
    let disk_path = "tinyos_disk.img";
    if !std::path::Path::new(disk_path).exists() {
        println!("💿 Creating default 16MB FAT32 disk image ({})...", disk_path);
        
        // Create blank file
        let status = Command::new("dd")
            .args(["if=/dev/zero", &format!("of={}", disk_path), "bs=1M", "count=16"])
            .status()
            .context("Failed to run dd")?;
        if !status.success() {
            bail!("dd failed to create disk image");
        }

        // Create MBR partition table
        let status = Command::new("sh")
            .arg("-c")
            .arg(format!("echo 'label: dos\nstart=2048, type=b' | sfdisk {}", disk_path))
            .status()
            .context("Failed to run sfdisk")?;
        if !status.success() {
            bail!("sfdisk failed to partition disk image");
        }

        // Format partition 1 as FAT32
        let status = Command::new("mformat")
            .args(["-i", &format!("{}@@1M", disk_path), "-F", "-v", "LEVITATE"])
            .status()
            .context("Failed to run mformat")?;
        if !status.success() {
            bail!("mformat failed to format partition");
        }
    }
    Ok(())
}

pub fn install_userspace_to_disk() -> Result<()> {
    create_disk_image_if_missing()?;
    
    // TEAM_121: Install/Update userspace apps on the disk
    // We do this even if the disk image already exists to ensure binaries reflect latest build
    println!("📦 Installing/Updating userspace apps on disk image...");
    let disk_path = "tinyos_disk.img";
    let binaries = ["init", "shell"];
    for bin in binaries {
        let src = format!("userspace/target/aarch64-unknown-none/release/{}", bin);
        if std::path::Path::new(&src).exists() {
            let status = Command::new("mcopy")
                .args(["-i", &format!("{}@@1M", disk_path), "-o", &src, &format!("::/{}", bin)])
                .status()
                .context(format!("Failed to copy {} to disk", bin))?;
            if status.success() {
                println!("  - Installed '{}' to disk", bin);
            }
        }
    }

    Ok(())
}
