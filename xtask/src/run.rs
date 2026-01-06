use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::{build, image};

#[derive(Subcommand)]
pub enum RunCommands {
    /// Run with GUI window (keyboard goes to QEMU window)
    Default,
    /// Run Pixel 6 Profile
    Pixel6,
    /// Run with VNC for browser verification
    Vnc,
    /// Run in terminal-only mode (WSL-like, keyboard in terminal)
    Term,
}

/// QEMU hardware profiles
/// TEAM_042: Added Pixel 6 profile for target hardware testing
#[derive(Clone, Copy)]
pub enum QemuProfile {
    /// Default: 512MB RAM, 1 core, cortex-a53
    Default,
    /// Pixel 6: 8GB RAM, 8 cores, cortex-a76, GICv3
    Pixel6,
    /// Test: GICv3 on default machine
    GicV3,
}

impl QemuProfile {
    pub fn machine(&self) -> String {
        match self {
            QemuProfile::Default => "virt".to_string(),
            QemuProfile::Pixel6 => "virt,gic-version=3".to_string(),
            QemuProfile::GicV3 => "virt,gic-version=3".to_string(),
        }
    }

    pub fn cpu(&self) -> &'static str {
        match self {
            QemuProfile::Default => "cortex-a53",
            QemuProfile::Pixel6 => "cortex-a76",
            QemuProfile::GicV3 => "cortex-a53",
        }
    }

    pub fn memory(&self) -> &'static str {
        match self {
            QemuProfile::Default => "512M",
            QemuProfile::Pixel6 => "8G",
            QemuProfile::GicV3 => "512M",
        }
    }

    /// Returns SMP topology string
    pub fn smp(&self) -> Option<&'static str> {
        match self {
            QemuProfile::Default => None,
            QemuProfile::Pixel6 => Some("8"),
            QemuProfile::GicV3 => None,
        }
    }
}

pub fn run_qemu(profile: QemuProfile, headless: bool) -> Result<()> {
    image::create_disk_image_if_missing()?;
    // Userspace must be installed by build_all before this is called

    let kernel_bin = "kernel64_rust.bin";

    let machine = profile.machine();
    let mut args = vec![
        "-M", machine.as_str(),
        "-cpu", profile.cpu(),
        "-m", profile.memory(),
        "-kernel", kernel_bin,
        "-device", "virtio-gpu-pci,xres=1920,yres=1080", // TEAM_115: Larger resolution
        "-device", "virtio-keyboard-device",
        "-device", "virtio-tablet-device",
        "-device", "virtio-net-device,netdev=net0",
        "-netdev", "user,id=net0",
        "-drive", "file=tinyos_disk.img,format=raw,if=none,id=hd0",
        "-device", "virtio-blk-device,drive=hd0",
        "-initrd", "initramfs.cpio",
        "-no-reboot",
    ];

    if let Some(smp) = profile.smp() {
        args.extend(["-smp", smp]);
    }

    // TEAM_139: Use mon:stdio for serial+monitor multiplexing
    // User can switch with Ctrl+A C, exit with Ctrl+A X
    if headless {
        args.extend(["-display", "none", "-serial", "mon:stdio"]);
    } else {
        // TEAM_139: Explicit GTK display for proper window sizing
        args.extend(["-display", "gtk,zoom-to-fit=off,window-close=off", "-serial", "mon:stdio"]);
    }

    Command::new("qemu-system-aarch64")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run QEMU")?;

    Ok(())
}

/// Run QEMU with VNC for browser-based GPU display verification.
pub fn run_qemu_vnc() -> Result<()> {
    println!("🖥️  Starting QEMU with VNC for browser-based display verification...\n");
    
    image::create_disk_image_if_missing()?;
    // Build kernel first (implies userspace build + install)
    build::build_all()?;
    
    // Check for noVNC
    let novnc_path = PathBuf::from("/tmp/novnc");
    if !novnc_path.exists() {
        println!("📥 Downloading noVNC...");
        let status = Command::new("git")
            .args(["clone", "--depth", "1", "https://github.com/novnc/noVNC.git", "/tmp/novnc"])
            .status()
            .context("Failed to clone noVNC")?;
        if !status.success() {
            bail!("Failed to download noVNC");
        }
    }
    
    // Find websockify - check multiple locations
    let websockify_path = find_websockify()?;
    
    // Kill any existing VNC-related processes (idempotency)
    // Use specific patterns to avoid killing unrelated QEMU instances
    println!("🧹 Cleaning up existing processes...");
    let _ = Command::new("pkill").args(["-f", "websockify.*6080"]).status();
    let _ = Command::new("pkill").args(["-f", "qemu.*-vnc.*:0"]).status();
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Start websockify
    println!("🔌 Starting websockify proxy...");
    let mut websockify = Command::new(&websockify_path)
        .args(["--web=/tmp/novnc", "6080", "localhost:5900"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start websockify")?;
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    // Verify websockify started
    match websockify.try_wait() {
        Ok(Some(status)) => {
            bail!("websockify exited immediately with status: {}. Port 6080 may be in use.", status);
        }
        Ok(None) => {} // Still running, good
        Err(e) => {
            bail!("Failed to check websockify status: {}", e);
        }
    }
    
    println!("");
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║  🌐 BROWSER URL: http://localhost:6080/vnc.html                        ║");
    println!("║                                                                         ║");
    println!("║  📋 AI AGENT INSTRUCTIONS:                                              ║");
    println!("║     1. Navigate browser to the URL above                                ║");
    println!("║     2. Click 'Connect' button                                           ║");
    println!("║     3. Check what displays:                                             ║");
    println!("║        • 'Display output is not active' = GPU BROKEN ❌                 ║");
    println!("║        • Terminal text visible = GPU WORKING ✅                         ║");
    println!("║                                                                         ║");
    println!("║  Serial console is in THIS terminal (Ctrl+C to quit)                    ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝");
    println!("");
    
    // Clean QMP socket if it exists
    if std::path::Path::new("./qmp.sock").exists() {
        let _ = std::fs::remove_file("./qmp.sock");
    }
    
    // Run QEMU with VNC
    let kernel_bin = "kernel64_rust.bin";
    let profile = QemuProfile::Default;
    let machine = profile.machine();
    
    let mut args = vec![
        "-M", machine.as_str(),
        "-cpu", profile.cpu(),
        "-m", profile.memory(),
        "-kernel", kernel_bin,
        "-display", "none",
        "-vnc", ":0",
        "-device", "virtio-gpu-pci,xres=1920,yres=1080", // TEAM_115: Larger resolution for VNC
        "-device", "virtio-keyboard-device",
        "-device", "virtio-tablet-device",
        "-device", "virtio-net-device,netdev=net0",
        "-netdev", "user,id=net0",
        "-drive", "file=tinyos_disk.img,format=raw,if=none,id=hd0",
        "-device", "virtio-blk-device,drive=hd0",
        "-initrd", "initramfs.cpio",
        "-serial", "mon:stdio",
        "-qmp", "unix:./qmp.sock,server,nowait",
        "-no-reboot",
    ];
    
    if let Some(smp) = profile.smp() {
        args.extend(["-smp", smp]);
    }
    
    let qemu_result = Command::new("qemu-system-aarch64")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    
    // Cleanup
    let _ = websockify.kill();
    
    qemu_result.context("Failed to run QEMU")?;
    
    Ok(())
}

/// Find websockify binary in various possible locations
fn find_websockify() -> Result<String> {
    // Check PATH first (covers system installs and activated venvs)
    if let Ok(output) = Command::new("which").arg("websockify").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                println!("  Found websockify at: {}", path);
                return Ok(path);
            }
        }
    }
    
    // Check common pip user install location
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let pip_path = format!("{}/.local/bin/websockify", home);
    if std::path::Path::new(&pip_path).exists() {
        println!("  Found websockify at: {}", pip_path);
        return Ok(pip_path);
    }
    
    // Check for pipx installation
    let pipx_path = format!("{}/.local/pipx/venvs/websockify/bin/websockify", home);
    if std::path::Path::new(&pipx_path).exists() {
        println!("  Found websockify at: {}", pipx_path);
        return Ok(pipx_path);
    }
    
    bail!(
        "websockify not found!\n\
        \n\
        Install with one of:\n\
        • pip3 install websockify\n\
        • pipx install websockify\n\
        • sudo dnf install python3-websockify  (Fedora)\n\
        • sudo apt install websockify  (Debian/Ubuntu)"
    )
}

/// TEAM_139: Run QEMU in terminal-only mode (WSL-like).
/// No graphical window - keyboard input goes to terminal stdin.
/// Ctrl+A X to exit, Ctrl+A C to switch to QEMU monitor.
pub fn run_qemu_term() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  LevitateOS Terminal Mode (WSL-like)                       ║");
    println!("║                                                            ║");
    println!("║  Type directly here - keyboard goes to VM                  ║");
    println!("║  Ctrl+A X to exit QEMU                                     ║");
    println!("║  Ctrl+A C to switch to QEMU monitor                        ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    image::create_disk_image_if_missing()?;
    build::build_all()?;

    let kernel_bin = "kernel64_rust.bin";
    let args = vec![
        "-M", "virt",
        "-cpu", "cortex-a72",
        "-m", "1G",
        "-kernel", kernel_bin,
        "-nographic",  // No display, stdin goes to serial
        "-device", "virtio-gpu-pci,xres=1280,yres=800",
        "-device", "virtio-keyboard-device",
        "-device", "virtio-tablet-device",
        "-device", "virtio-net-device,netdev=net0",
        "-netdev", "user,id=net0",
        "-drive", "file=tinyos_disk.img,format=raw,if=none,id=hd0",
        "-device", "virtio-blk-device,drive=hd0",
        "-initrd", "initramfs.cpio",
        "-serial", "mon:stdio",
        "-qmp", "unix:./qmp.sock,server,nowait",
        "-no-reboot",
    ];

    // Clean QMP socket
    let _ = std::fs::remove_file("./qmp.sock");

    Command::new("qemu-system-aarch64")
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run QEMU")?;

    Ok(())
}
