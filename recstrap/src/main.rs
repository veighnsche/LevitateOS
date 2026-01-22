//! recstrap - LevitateOS installer
//!
//! Installs LevitateOS from the live environment to disk.
//! Similar to archinstall for Arch Linux.
//!
//! Usage: recstrap /dev/vda
//!
//! This will:
//! 1. Partition the disk (GPT: 512MB EFI + rest as root)
//! 2. Format partitions (FAT32 for EFI, ext4 for root)
//! 3. Mount partitions
//! 4. Extract squashfs to disk
//! 5. Generate /etc/fstab
//! 6. Install systemd-boot bootloader
//! 7. Prompt for root password
//! 8. Unmount and done

use anyhow::{bail, Context, Result};
use clap::Parser;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(name = "recstrap")]
#[command(about = "LevitateOS installer - installs from live ISO to disk")]
struct Args {
    /// Target disk (e.g., /dev/vda, /dev/sda)
    disk: String,

    /// EFI partition size (default: 512M)
    #[arg(long, default_value = "512M")]
    efi_size: String,

    /// Skip partitioning and formatting (use existing partitions)
    #[arg(long)]
    no_format: bool,

    /// Skip bootloader installation
    #[arg(long)]
    no_bootloader: bool,

    /// Squashfs location (default: /media/cdrom/live/filesystem.squashfs)
    #[arg(long)]
    squashfs: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║              recstrap - LevitateOS Installer                  ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    // Validate disk exists
    if !Path::new(&args.disk).exists() {
        bail!("Disk {} not found", args.disk);
    }

    // Find squashfs
    let squashfs = args
        .squashfs
        .unwrap_or_else(|| "/media/cdrom/live/filesystem.squashfs".to_string());

    if !Path::new(&squashfs).exists() {
        bail!(
            "Squashfs not found at {}\n\
             Make sure you're running from the live ISO.",
            squashfs
        );
    }

    println!("Target disk: {}", args.disk);
    println!("Squashfs:    {}", squashfs);
    println!();

    // Confirm
    print!("This will ERASE ALL DATA on {}. Continue? [y/N] ", args.disk);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Aborted.");
        return Ok(());
    }

    // Determine partition names (nvme vs sd)
    let (efi_part, root_part) = if args.disk.contains("nvme") || args.disk.contains("mmcblk") {
        (format!("{}p1", args.disk), format!("{}p2", args.disk))
    } else {
        (format!("{}1", args.disk), format!("{}2", args.disk))
    };

    // Step 1: Partition
    if !args.no_format {
        println!("\n[1/7] Partitioning {}...", args.disk);
        partition_disk(&args.disk, &args.efi_size)?;
    } else {
        println!("\n[1/7] Skipping partitioning (--no-format)");
    }

    // Step 2: Format
    if !args.no_format {
        println!("\n[2/7] Formatting partitions...");
        format_partitions(&efi_part, &root_part)?;
    } else {
        println!("\n[2/7] Skipping formatting (--no-format)");
    }

    // Step 3: Mount
    println!("\n[3/7] Mounting partitions...");
    mount_partitions(&efi_part, &root_part)?;

    // Step 4: Extract
    println!("\n[4/7] Extracting system (this may take a moment)...");
    extract_squashfs(&squashfs)?;

    // Step 5: Generate fstab
    println!("\n[5/7] Generating /etc/fstab...");
    generate_fstab(&efi_part, &root_part)?;

    // Step 6: Bootloader
    if !args.no_bootloader {
        println!("\n[6/7] Installing bootloader...");
        install_bootloader()?;
    } else {
        println!("\n[6/7] Skipping bootloader (--no-bootloader)");
    }

    // Step 7: Set password
    println!("\n[7/7] Setting root password...");
    set_root_password()?;

    // Cleanup
    println!("\nUnmounting...");
    unmount()?;

    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                  Installation Complete!                       ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║  Remove the installation media and reboot.                    ║");
    println!("║                                                               ║");
    println!("║  After reboot, log in as root with the password you set.     ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");

    Ok(())
}

fn run(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .with_context(|| format!("Failed to run {}", cmd))?;

    if !status.success() {
        bail!("{} failed with exit code {:?}", cmd, status.code());
    }
    Ok(())
}

fn run_with_stdin(cmd: &str, args: &[&str], stdin_data: &str) -> Result<()> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to run {}", cmd))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(stdin_data.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        bail!("{} failed", cmd);
    }
    Ok(())
}

fn get_uuid(device: &str) -> Result<String> {
    let output = Command::new("blkid")
        .args(["-s", "UUID", "-o", "value", device])
        .output()
        .context("Failed to run blkid")?;

    let uuid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if uuid.is_empty() {
        bail!("Could not get UUID for {}", device);
    }
    Ok(uuid)
}

fn partition_disk(disk: &str, efi_size: &str) -> Result<()> {
    // Wipe existing partition table
    run("wipefs", &["--all", "--force", disk])?;

    // Create GPT partition table with sfdisk
    let script = format!(
        "label: gpt\n\
         size={}, type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B, name=\"EFI\"\n\
         type=4F68BCE3-E8CD-4DB1-96E7-FBCAF984B709, name=\"root\"\n",
        efi_size
    );

    run_with_stdin("sfdisk", &[disk], &script)?;

    // Re-read partition table
    let _ = Command::new("partprobe").arg(disk).status();
    std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}

fn format_partitions(efi_part: &str, root_part: &str) -> Result<()> {
    println!("  Formatting {} as FAT32 (EFI)...", efi_part);
    run("mkfs.fat", &["-F", "32", efi_part])?;

    println!("  Formatting {} as ext4 (root)...", root_part);
    run("mkfs.ext4", &["-F", root_part])?;

    Ok(())
}

fn mount_partitions(efi_part: &str, root_part: &str) -> Result<()> {
    // Create mount point
    std::fs::create_dir_all("/mnt")?;

    // Mount root
    run("mount", &[root_part, "/mnt"])?;

    // Create and mount boot
    std::fs::create_dir_all("/mnt/boot")?;
    run("mount", &[efi_part, "/mnt/boot"])?;

    Ok(())
}

fn extract_squashfs(squashfs: &str) -> Result<()> {
    // unsquashfs extracts to a directory
    // We want to extract directly to /mnt, but unsquashfs creates a subdir
    // So we extract to /mnt and use -f to force overwrite

    run("unsquashfs", &["-f", "-d", "/mnt", squashfs])?;
    Ok(())
}

fn generate_fstab(efi_part: &str, root_part: &str) -> Result<()> {
    let efi_uuid = get_uuid(efi_part)?;
    let root_uuid = get_uuid(root_part)?;

    let fstab = format!(
        "# /etc/fstab - Generated by recstrap\n\
         # <device>                                <mount>  <type>  <options>         <dump> <pass>\n\
         UUID={}  /boot    vfat    defaults,umask=0077   0      2\n\
         UUID={}  /        ext4    defaults              0      1\n",
        efi_uuid, root_uuid
    );

    std::fs::write("/mnt/etc/fstab", fstab)?;
    Ok(())
}

fn install_bootloader() -> Result<()> {
    // Install systemd-boot
    run("bootctl", &["--esp-path=/mnt/boot", "install"])?;

    // Create loader.conf
    let loader_conf = "default levitateos.conf\ntimeout 3\neditor no\n";
    std::fs::create_dir_all("/mnt/boot/loader")?;
    std::fs::write("/mnt/boot/loader/loader.conf", loader_conf)?;

    // Get root UUID for kernel cmdline
    // Find root partition from mount
    let output = Command::new("findmnt")
        .args(["-n", "-o", "SOURCE", "/mnt"])
        .output()?;
    let root_dev = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let root_uuid = get_uuid(&root_dev)?;

    // Create boot entry
    let entry = format!(
        "title   LevitateOS\n\
         linux   /vmlinuz\n\
         initrd  /initramfs.img\n\
         options root=UUID={} rw selinux=0\n",
        root_uuid
    );
    std::fs::create_dir_all("/mnt/boot/loader/entries")?;
    std::fs::write("/mnt/boot/loader/entries/levitateos.conf", entry)?;

    // Copy kernel and initramfs to boot partition
    if Path::new("/mnt/boot/vmlinuz").exists() {
        println!("  Kernel already in /boot");
    } else if Path::new("/mnt/usr/lib/modules").exists() {
        // Find kernel in modules directory
        println!("  Note: You may need to copy kernel to /boot manually");
    }

    Ok(())
}

fn set_root_password() -> Result<()> {
    println!("  Enter new root password:");

    // Use chroot to run passwd
    let status = Command::new("chroot")
        .args(["/mnt", "passwd", "root"])
        .status()?;

    if !status.success() {
        println!("  Warning: Failed to set password. You can set it after reboot.");
    }

    Ok(())
}

fn unmount() -> Result<()> {
    let _ = Command::new("umount").arg("/mnt/boot").status();
    let _ = Command::new("umount").arg("/mnt").status();
    Ok(())
}
