//! VM management for testing LevitateOS.
//!
//! Uses QEMU with virtio-gpu for Wayland desktop testing.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// VM configuration paths
fn vm_dir() -> PathBuf {
    let dir = project_root().join(".vm");
    fs::create_dir_all(&dir).ok();
    dir
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn disk_image() -> PathBuf {
    vm_dir().join("levitate-test.qcow2")
}

fn pid_file() -> PathBuf {
    vm_dir().join("qemu.pid")
}

fn monitor_socket() -> PathBuf {
    vm_dir().join("qemu-monitor.sock")
}

fn serial_log() -> PathBuf {
    vm_dir().join("serial.log")
}

const SSH_PORT: u16 = 2222;
const DISK_SIZE: &str = "20G";

/// Check if QEMU is available
fn check_qemu() -> Result<()> {
    which::which("qemu-system-x86_64")
        .context("qemu-system-x86_64 not found. Install QEMU.")?;
    Ok(())
}

/// Check if VM is running
fn is_running() -> bool {
    if let Ok(pid_str) = fs::read_to_string(pid_file()) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            // Check if process exists
            return std::path::Path::new(&format!("/proc/{}", pid)).exists();
        }
    }
    false
}

/// Start the VM
pub fn start(
    detach: bool,
    gui: bool,
    memory: u32,
    cpus: u32,
    cdrom: Option<String>,
    uefi: bool,
) -> Result<()> {
    check_qemu()?;

    if is_running() {
        bail!("VM is already running. Use 'cargo xtask vm stop' first.");
    }

    let disk = disk_image();
    if !disk.exists() {
        bail!(
            "Disk image not found at {:?}\nRun 'cargo xtask vm setup' first.",
            disk
        );
    }

    let mut args = vec![
        "-enable-kvm".to_string(),
        "-cpu".to_string(), "host".to_string(),
        "-m".to_string(), format!("{}M", memory),
        "-smp".to_string(), format!("{}", cpus),
        // Disk
        "-drive".to_string(),
        format!("file={},format=qcow2,if=virtio", disk.display()),
        // Network with SSH forwarding
        "-netdev".to_string(),
        format!("user,id=net0,hostfwd=tcp::{}-:22", SSH_PORT),
        "-device".to_string(), "virtio-net-pci,netdev=net0".to_string(),
        // Serial console to file and stdio
        "-serial".to_string(), "mon:stdio".to_string(),
        // Monitor socket for control
        "-monitor".to_string(),
        format!("unix:{},server,nowait", monitor_socket().display()),
        // PID file
        "-pidfile".to_string(), pid_file().display().to_string(),
    ];

    // UEFI boot (requires OVMF package)
    if uefi {
        // Common OVMF paths on different distros
        let ovmf_paths = [
            "/usr/share/edk2/ovmf/OVMF_CODE.fd",           // Fedora/RHEL
            "/usr/share/OVMF/OVMF_CODE.fd",                // Debian/Ubuntu
            "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd",       // Arch
            "/usr/share/qemu/OVMF_CODE.fd",                // Generic
        ];

        let ovmf = ovmf_paths.iter().find(|p| std::path::Path::new(p).exists());
        match ovmf {
            Some(path) => {
                args.extend([
                    "-drive".to_string(),
                    format!("if=pflash,format=raw,readonly=on,file={}", path),
                ]);
            }
            None => {
                bail!("OVMF not found. Install edk2-ovmf package for UEFI support.");
            }
        }
    }

    // CDROM/ISO for installation
    if let Some(iso) = &cdrom {
        let iso_path = if iso == "arch" {
            // Shortcut: look for arch.iso in .vm directory
            let auto_path = vm_dir().join("arch.iso");
            if !auto_path.exists() {
                bail!("Arch ISO not found at {:?}\nDownload with: curl -LO https://mirrors.kernel.org/archlinux/iso/latest/archlinux-x86_64.iso && mv archlinux-*.iso {:?}", auto_path, auto_path);
            }
            auto_path.display().to_string()
        } else {
            iso.clone()
        };

        if !std::path::Path::new(&iso_path).exists() {
            bail!("ISO file not found: {}", iso_path);
        }

        args.extend([
            "-cdrom".to_string(), iso_path.clone(),
            "-boot".to_string(), "d".to_string(),  // Boot from CD first
        ]);
        println!("  CDROM: {}", iso_path);
    }

    if gui {
        // GUI mode with virtio-gpu for Wayland/OpenGL
        args.extend([
            "-device".to_string(), "virtio-vga-gl".to_string(),
            "-display".to_string(), "gtk,gl=on".to_string(),
            "-device".to_string(), "virtio-keyboard".to_string(),
            "-device".to_string(), "virtio-mouse".to_string(),
            // Audio (for a complete desktop experience)
            "-device".to_string(), "intel-hda".to_string(),
            "-device".to_string(), "hda-duplex".to_string(),
        ]);
    } else {
        // Headless mode - still need basic VGA for boot
        args.extend([
            "-nographic".to_string(),
        ]);
    }

    if detach && !gui {
        args.push("-daemonize".to_string());
    }

    println!("Starting VM...");
    println!("  Memory: {} MB", memory);
    println!("  CPUs: {}", cpus);
    println!("  SSH: localhost:{}", SSH_PORT);
    println!("  GUI: {}", if gui { "enabled" } else { "disabled" });
    println!("  UEFI: {}", if uefi { "enabled" } else { "disabled (BIOS)" });

    let status = Command::new("qemu-system-x86_64")
        .args(&args)
        .status()
        .context("Failed to start QEMU")?;

    if detach && !gui {
        if status.success() {
            println!("\nVM started in background.");
            println!("  SSH: ssh -p {} root@localhost", SSH_PORT);
            println!("  Stop: cargo xtask vm stop");
        } else {
            bail!("Failed to start VM");
        }
    }

    Ok(())
}

/// Stop the VM
pub fn stop() -> Result<()> {
    if !is_running() {
        println!("VM is not running.");
        return Ok(());
    }

    // Try graceful shutdown via monitor
    let monitor = monitor_socket();
    if monitor.exists() {
        println!("Sending shutdown signal...");
        let _ = Command::new("sh")
            .args(["-c", &format!("echo 'system_powerdown' | socat - UNIX-CONNECT:{}", monitor.display())])
            .status();

        // Wait a bit for graceful shutdown
        std::thread::sleep(std::time::Duration::from_secs(3));
    }

    // Force kill if still running
    if is_running() {
        if let Ok(pid_str) = fs::read_to_string(pid_file()) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                println!("Force killing VM (PID {})...", pid);
                let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
            }
        }
    }

    // Cleanup
    let _ = fs::remove_file(pid_file());
    let _ = fs::remove_file(monitor_socket());

    println!("VM stopped.");
    Ok(())
}

/// Show VM status
pub fn status() -> Result<()> {
    if is_running() {
        let pid = fs::read_to_string(pid_file())
            .unwrap_or_default()
            .trim()
            .to_string();
        println!("VM is running (PID {})", pid);
        println!("  SSH: ssh -p {} root@localhost", SSH_PORT);
    } else {
        println!("VM is not running.");
    }

    let disk = disk_image();
    if disk.exists() {
        let meta = fs::metadata(&disk)?;
        println!("  Disk: {:?} ({:.1} GB)", disk, meta.len() as f64 / 1e9);
    } else {
        println!("  Disk: not created (run 'cargo xtask vm setup')");
    }

    Ok(())
}

/// Send command to VM via SSH
pub fn send(command: &str) -> Result<()> {
    if !is_running() {
        bail!("VM is not running. Start it with 'cargo xtask vm start'");
    }

    let status = Command::new("ssh")
        .args([
            "-o", "StrictHostKeyChecking=no",
            "-o", "UserKnownHostsFile=/dev/null",
            "-o", "LogLevel=ERROR",
            "-p", &SSH_PORT.to_string(),
            "root@localhost",
            command,
        ])
        .status()
        .context("Failed to SSH")?;

    if !status.success() {
        bail!("Command failed with exit code {:?}", status.code());
    }

    Ok(())
}

/// Show serial log
pub fn log(follow: bool) -> Result<()> {
    let log = serial_log();
    if !log.exists() {
        bail!("No log file found. Has the VM been started?");
    }

    if follow {
        // Use tail -f
        Command::new("tail")
            .args(["-f", &log.display().to_string()])
            .status()
            .context("Failed to tail log")?;
    } else {
        let content = fs::read_to_string(&log)?;
        println!("{}", content);
    }

    Ok(())
}

/// SSH into VM
pub fn ssh() -> Result<()> {
    if !is_running() {
        bail!("VM is not running. Start it with 'cargo xtask vm start'");
    }

    Command::new("ssh")
        .args([
            "-o", "StrictHostKeyChecking=no",
            "-o", "UserKnownHostsFile=/dev/null",
            "-p", &SSH_PORT.to_string(),
            "root@localhost",
        ])
        .status()
        .context("Failed to SSH")?;

    Ok(())
}

/// Setup/create the base Arch Linux image
pub fn setup(force: bool) -> Result<()> {
    check_qemu()?;

    let disk = disk_image();
    if disk.exists() && !force {
        println!("Disk image already exists at {:?}", disk);
        println!("Use --force to recreate.");
        return Ok(());
    }

    println!("=== LevitateOS VM Setup ===\n");

    // Check for required tools
    which::which("qemu-img").context("qemu-img not found")?;

    // Create disk image
    println!("Creating {} disk image...", DISK_SIZE);
    let status = Command::new("qemu-img")
        .args(["create", "-f", "qcow2", &disk.display().to_string(), DISK_SIZE])
        .status()?;

    if !status.success() {
        bail!("Failed to create disk image");
    }

    println!("\nDisk image created: {:?}", disk);
    println!("\n=== Next Steps ===\n");
    println!("1. Download Arch Linux ISO:");
    println!("   curl -LO https://mirrors.kernel.org/archlinux/iso/latest/archlinux-x86_64.iso");
    println!("   mv archlinux-*.iso {:?}", vm_dir().join("arch.iso"));
    println!();
    println!("2. Boot installer and install Arch:");
    println!("   cargo xtask vm start --gui");
    println!("   # Add: -cdrom {:?}", vm_dir().join("arch.iso"));
    println!();
    println!("3. Inside VM, install with:");
    println!("   - pacstrap /mnt base linux linux-firmware sway foot waybar");
    println!("   - Enable sshd, set root password");
    println!();
    // Download Arch ISO
    let iso = vm_dir().join("arch.iso");
    if !iso.exists() {
        println!("\nDownloading Arch Linux ISO...");
        let status = Command::new("curl")
            .args([
                "-L",
                "-o", &iso.display().to_string(),
                "https://mirrors.kernel.org/archlinux/iso/latest/archlinux-x86_64.iso"
            ])
            .status();

        if status.is_ok() && status.unwrap().success() {
            println!("Downloaded: {:?}", iso);
        } else {
            println!("Warning: Failed to download ISO. Download manually:");
            println!("  curl -LO https://mirrors.kernel.org/archlinux/iso/latest/archlinux-x86_64.iso");
            println!("  mv archlinux-*.iso {:?}", iso);
        }
    } else {
        println!("Arch ISO already exists: {:?}", iso);
    }

    println!("\n=== Next Steps ===\n");
    println!("1. Prepare levitate binary and recipes:");
    println!("   cargo xtask vm prepare");
    println!();
    println!("2. Boot the Arch installer:");
    println!("   cargo xtask vm start --gui --cdrom arch --uefi");
    println!();
    println!("3. Inside the VM, run the install script:");
    println!("   cargo xtask vm install-script  # (shows what to run)");
    println!();

    Ok(())
}

/// Build levitate binary and prepare files for VM
pub fn prepare() -> Result<()> {
    println!("=== Preparing LevitateOS files for VM ===\n");

    // Build levitate binary for release
    println!("[1/3] Building levitate binary...");
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "levitate-recipe", "--bin", "levitate"])
        .current_dir(project_root())
        .status()
        .context("Failed to build levitate")?;

    if !status.success() {
        bail!("Failed to build levitate binary");
    }

    // Copy to .vm directory
    let levitate_src = project_root().join("target/release/levitate");
    let levitate_dst = vm_dir().join("levitate");

    if levitate_src.exists() {
        fs::copy(&levitate_src, &levitate_dst)?;
        println!("   Built: {:?}", levitate_dst);
    } else {
        bail!("Binary not found at {:?}", levitate_src);
    }

    // Copy recipes
    println!("[2/3] Copying recipes...");
    let recipes_src = project_root().join("crates/recipe/examples");
    let recipes_dst = vm_dir().join("recipes");
    fs::create_dir_all(&recipes_dst)?;

    let mut count = 0;
    for entry in fs::read_dir(&recipes_src)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "recipe").unwrap_or(false) {
            let dest = recipes_dst.join(path.file_name().unwrap());
            fs::copy(&path, &dest)?;
            count += 1;
        }
    }
    println!("   Copied {} recipes to {:?}", count, recipes_dst);

    // Copy install script
    println!("[3/3] Preparing install script...");
    let script_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/scripts/install-arch.sh");
    let script_dst = vm_dir().join("install-arch.sh");

    if script_src.exists() {
        fs::copy(&script_src, &script_dst)?;
        println!("   Copied: {:?}", script_dst);
    }

    println!("\n=== Preparation Complete ===\n");
    println!("Files ready in {:?}", vm_dir());
    println!();
    println!("Next: cargo xtask vm start --gui --cdrom arch --uefi");

    Ok(())
}

/// Show the install script to run inside VM
pub fn install_script() -> Result<()> {
    println!("=== Run this inside the Arch live environment ===\n");
    println!("# First, start a web server to serve the files (on host):");
    println!("cd {:?} && python3 -m http.server 8080", vm_dir());
    println!();
    println!("# Then, in the VM (find host IP with: ip route | grep default):");
    println!("curl -O http://10.0.2.2:8080/install-arch.sh");
    println!("curl -O http://10.0.2.2:8080/levitate");
    println!("mkdir -p /tmp/recipes");
    println!("curl -O http://10.0.2.2:8080/recipes/sway.recipe  # etc.");
    println!("chmod +x install-arch.sh");
    println!("./install-arch.sh");
    println!();
    println!("# Or, simpler - just do manual install:");
    println!();
    println!("# === QUICK MANUAL INSTALL ===");
    println!("# Partition disk (UEFI):");
    println!("parted -s /dev/vda mklabel gpt");
    println!("parted -s /dev/vda mkpart ESP fat32 1MiB 513MiB");
    println!("parted -s /dev/vda set 1 esp on");
    println!("parted -s /dev/vda mkpart root ext4 513MiB 100%");
    println!();
    println!("# Format:");
    println!("mkfs.fat -F32 /dev/vda1");
    println!("mkfs.ext4 /dev/vda2");
    println!();
    println!("# Mount:");
    println!("mount /dev/vda2 /mnt");
    println!("mkdir -p /mnt/boot");
    println!("mount /dev/vda1 /mnt/boot");
    println!();
    println!("# Install:");
    println!("pacstrap -K /mnt base linux linux-firmware base-devel git \\");
    println!("  meson ninja cmake networkmanager openssh sudo \\");
    println!("  mesa wayland wayland-protocols libxkbcommon libinput \\");
    println!("  cairo pango gdk-pixbuf2 scdoc");
    println!();
    println!("# Configure:");
    println!("genfstab -U /mnt >> /mnt/etc/fstab");
    println!("arch-chroot /mnt");
    println!("  ln -sf /usr/share/zoneinfo/UTC /etc/localtime");
    println!("  echo 'en_US.UTF-8 UTF-8' >> /etc/locale.gen && locale-gen");
    println!("  echo 'levitate-test' > /etc/hostname");
    println!("  echo 'root:live' | chpasswd");
    println!("  useradd -m -G wheel -s /bin/bash live");
    println!("  echo 'live:live' | chpasswd");
    println!("  echo '%wheel ALL=(ALL:ALL) NOPASSWD: ALL' >> /etc/sudoers");
    println!("  systemctl enable NetworkManager sshd");
    println!("  bootctl install");
    println!("  # Create /boot/loader/entries/arch.conf");
    println!("  exit");
    println!();
    println!("# Reboot:");
    println!("umount -R /mnt");
    println!("reboot");
    println!();
    println!("# After reboot, login as live:live");
    println!("# Then copy levitate binary and recipes, and run:");
    println!("#   levitate desktop");
    println!("#   sway");

    Ok(())
}

/// Copy levitate binary and recipes to running VM via SCP
pub fn copy_files() -> Result<()> {
    if !is_running() {
        bail!("VM is not running. Start it first with: cargo xtask vm start --gui");
    }

    let levitate = vm_dir().join("levitate");
    let recipes = vm_dir().join("recipes");

    if !levitate.exists() {
        bail!("Levitate binary not found. Run: cargo xtask vm prepare");
    }

    println!("=== Copying files to VM ===\n");

    let scp_opts = [
        "-o", "StrictHostKeyChecking=no",
        "-o", "UserKnownHostsFile=/dev/null",
        "-o", "LogLevel=ERROR",
        "-P", &SSH_PORT.to_string(),
    ];

    // Copy levitate binary
    println!("[1/3] Copying levitate binary...");
    let status = Command::new("scp")
        .args(&scp_opts)
        .arg(&levitate)
        .arg("live@localhost:/tmp/levitate")
        .status()
        .context("Failed to SCP levitate")?;

    if !status.success() {
        bail!("Failed to copy levitate binary. Is SSH running in the VM?");
    }

    // Install it to /usr/local/bin (needs sudo)
    println!("   Installing to /usr/local/bin...");
    let status = Command::new("ssh")
        .args([
            "-o", "StrictHostKeyChecking=no",
            "-o", "UserKnownHostsFile=/dev/null",
            "-o", "LogLevel=ERROR",
            "-p", &SSH_PORT.to_string(),
            "live@localhost",
            "sudo install -m755 /tmp/levitate /usr/local/bin/levitate",
        ])
        .status()?;

    if !status.success() {
        eprintln!("   Warning: Could not install to /usr/local/bin");
    }

    // Create recipes directory
    println!("[2/3] Creating recipes directory...");
    let _ = Command::new("ssh")
        .args([
            "-o", "StrictHostKeyChecking=no",
            "-o", "UserKnownHostsFile=/dev/null",
            "-o", "LogLevel=ERROR",
            "-p", &SSH_PORT.to_string(),
            "live@localhost",
            "sudo mkdir -p /usr/share/levitate/recipes && sudo chown -R live:live /usr/share/levitate",
        ])
        .status();

    // Copy recipes
    println!("[3/3] Copying recipes...");
    if recipes.exists() {
        let status = Command::new("scp")
            .args(&scp_opts)
            .arg("-r")
            .arg(format!("{}/*", recipes.display()))
            .arg("live@localhost:/usr/share/levitate/recipes/")
            .status();

        if status.is_err() || !status.unwrap().success() {
            // Try individual files if glob doesn't work
            for entry in fs::read_dir(&recipes)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "recipe").unwrap_or(false) {
                    let _ = Command::new("scp")
                        .args(&scp_opts)
                        .arg(&path)
                        .arg("live@localhost:/usr/share/levitate/recipes/")
                        .status();
                }
            }
        }

        let count = fs::read_dir(&recipes)?
            .filter(|e| e.as_ref().map(|e| e.path().extension().map(|x| x == "recipe").unwrap_or(false)).unwrap_or(false))
            .count();
        println!("   Copied {} recipes", count);
    }

    println!("\n=== Copy Complete ===\n");
    println!("You can now SSH into the VM and run:");
    println!("  levitate list              # See available packages");
    println!("  levitate desktop           # Install Sway desktop");
    println!("  sway                       # Start Sway");
    println!();
    println!("Or SSH in with: cargo xtask vm ssh");

    Ok(())
}
