use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::{build, image};

#[derive(Subcommand)]
pub enum RunCommands {
    /// Run with GUI window (keyboard goes to QEMU window)
    Default {
        /// Boot from Limine ISO instead of -kernel
        #[arg(long)]
        iso: bool,
    },
    /// Run Pixel 6 Profile
    Pixel6,
    /// Run with VNC for browser verification
    Vnc,
    /// Run in terminal-only mode (WSL-like, keyboard in terminal)
    Term {
        /// Boot from Limine ISO instead of -kernel
        #[arg(long)]
        iso: bool,
    },
    /// TEAM_243: Run internal OS tests (for AI agent verification)
    Test,
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
    /// x86_64: 512MB RAM, 1 core, q35
    X86_64,
}

impl QemuProfile {
    pub fn machine(&self) -> String {
        match self {
            QemuProfile::Default => "virt".to_string(),
            QemuProfile::Pixel6 => "virt,gic-version=3".to_string(),
            QemuProfile::GicV3 => "virt,gic-version=3".to_string(),
            QemuProfile::X86_64 => "q35".to_string(),
        }
    }

    pub fn cpu(&self) -> &'static str {
        match self {
            QemuProfile::Default => "cortex-a53",
            QemuProfile::Pixel6 => "cortex-a76",
            QemuProfile::GicV3 => "cortex-a53",
            QemuProfile::X86_64 => "qemu64",
        }
    }

    pub fn memory(&self) -> &'static str {
        match self {
            QemuProfile::Default => "512M",
            QemuProfile::Pixel6 => "8G",
            QemuProfile::GicV3 => "512M",
            QemuProfile::X86_64 => "512M",
        }
    }

    /// Returns SMP topology string
    pub fn smp(&self) -> Option<&'static str> {
        match self {
            QemuProfile::Default => None,
            QemuProfile::Pixel6 => Some("8"),
            QemuProfile::GicV3 => None,
            QemuProfile::X86_64 => None,
        }
    }
}

pub fn run_qemu(profile: QemuProfile, headless: bool, iso: bool, arch: &str) -> Result<()> {
    image::create_disk_image_if_missing()?;
    // Userspace must be installed by build_all before this is called

    let kernel_bin = if arch == "aarch64" {
        "kernel64_rust.bin"
    } else {
        // x86_64 uses the ELF binary directly for multiboot2
        "target/x86_64-unknown-none/release/levitate-kernel"
    };

    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    let machine = profile.machine();
    let mut args = vec![
        "-M", machine.as_str(),
        "-cpu", profile.cpu(),
        "-m", profile.memory(),
        "-kernel", kernel_bin,
        "-device", "virtio-gpu-pci,xres=1920,yres=1080", // TEAM_115: Larger resolution
        "-device", if arch == "x86_64" { "virtio-keyboard-pci" } else { "virtio-keyboard-device" },
        "-device", if arch == "x86_64" { "virtio-tablet-pci" } else { "virtio-tablet-device" },
        "-device", if arch == "x86_64" { "virtio-net-pci,netdev=net0" } else { "virtio-net-device,netdev=net0" },
        "-netdev", "user,id=net0",
        "-drive", "file=tinyos_disk.img,format=raw,if=none,id=hd0",
        "-device", if arch == "x86_64" { "virtio-blk-pci,drive=hd0" } else { "virtio-blk-device,drive=hd0" },
        "-initrd", "initramfs.cpio",
        "-no-reboot",
    ];

    if iso {
        // If ISO boot, we use -cdrom instead of -kernel/-initrd
        // Note: Limine protocol works via -kernel too, but ISO is more "hardware-like"
        args.retain(|&arg| arg != "-kernel" && arg != "-initrd" && arg != kernel_bin && arg != "initramfs.cpio");
        args.extend(["-cdrom", "levitate.iso", "-boot", "d"]);
    }

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

    Command::new(qemu_bin)
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run QEMU")?;

    Ok(())
}

/// Run QEMU with VNC for browser-based GPU display verification.
pub fn run_qemu_vnc(arch: &str) -> Result<()> {
    println!("🖥️  Starting QEMU with VNC for browser-based display verification...\n");
    
    image::create_disk_image_if_missing()?;
    // Build kernel first (implies userspace build + install)
    build::build_all(arch)?;
    
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
    let kernel_bin = if arch == "aarch64" {
        "kernel64_rust.bin"
    } else {
        "target/x86_64-unknown-none/release/levitate-kernel"
    };

    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    let profile = if arch == "aarch64" {
        QemuProfile::Default
    } else {
        QemuProfile::X86_64
    };
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
    
    let qemu_result = Command::new(qemu_bin)
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

/// TEAM_243: Run QEMU with test runner for automated OS testing.
/// Builds test initramfs, runs headless, captures output, reports results.
pub fn run_qemu_test(arch: &str) -> Result<()> {
    println!("🧪 Running LevitateOS Internal Tests for {}...\n", arch);

    // Build everything including test runner
    build::build_userspace(arch)?;
    build::create_test_initramfs(arch)?;
    build::build_kernel_verbose(arch)?;
    image::create_disk_image_if_missing()?;

    let kernel_bin = if arch == "aarch64" {
        "kernel64_rust.bin"
    } else {
        "target/x86_64-unknown-none/release/levitate-kernel"
    };

    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    let profile = if arch == "aarch64" {
        QemuProfile::Default
    } else {
        QemuProfile::X86_64
    };

    let timeout_secs: u64 = 60;

    println!("Running QEMU (headless, {}s timeout)...\n", timeout_secs);

    // Build QEMU args - headless, serial to stdout
    let args = vec![
        format!("{}s", timeout_secs),
        qemu_bin.to_string(),
        "-M".to_string(), profile.machine().to_string(),
        "-cpu".to_string(), profile.cpu().to_string(),
        "-m".to_string(), profile.memory().to_string(),
        "-kernel".to_string(), kernel_bin.to_string(),
        "-display".to_string(), "none".to_string(),
        "-serial".to_string(), "stdio".to_string(),
        "-device".to_string(), "virtio-gpu-pci".to_string(),
        "-device".to_string(), "virtio-keyboard-device".to_string(),
        "-device".to_string(), "virtio-tablet-device".to_string(),
        "-device".to_string(), "virtio-net-device,netdev=net0".to_string(),
        "-netdev".to_string(), "user,id=net0".to_string(),
        "-drive".to_string(), "file=tinyos_disk.img,format=raw,if=none,id=hd0".to_string(),
        "-device".to_string(), "virtio-blk-device,drive=hd0".to_string(),
        "-initrd".to_string(), "initramfs_test.cpio".to_string(),
        "-no-reboot".to_string(),
    ];

    // Run QEMU with timeout
    let output = Command::new("timeout")
        .args(&args)
        .output()
        .context("Failed to run QEMU")?;

    // Print stdout (serial output)
    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);

    // Check for test results in output
    if stdout.contains("[TEST_RUNNER] RESULT: PASSED") {
        println!("\n✅ All OS internal tests passed!");
        Ok(())
    } else if stdout.contains("[TEST_RUNNER] RESULT: FAILED") {
        bail!("❌ Some OS internal tests failed!");
    } else if stdout.contains("[TEST_RUNNER]") {
        // Test runner started but didn't complete
        bail!("❌ Test runner did not complete (timeout or crash)");
    } else {
        // Test runner never started
        bail!("❌ Test runner failed to start - check initramfs");
    }
}

/// TEAM_139: Run QEMU in terminal-only mode (WSL-like).
/// No graphical window - keyboard input goes to terminal stdin.
/// Ctrl+A X to exit, Ctrl+A C to switch to QEMU monitor.
pub fn run_qemu_term(arch: &str, iso: bool) -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  LevitateOS Terminal Mode (WSL-like) - {}               ║", arch);
    println!("║                                                            ║");
    println!("║  Type directly here - keyboard goes to VM                  ║");
    println!("║  Ctrl+A X to exit QEMU                                     ║");
    println!("║  Ctrl+A C to switch to QEMU monitor                        ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    if iso {
        build::build_iso(arch)?;
    } else {
        image::create_disk_image_if_missing()?;
        build::build_all(arch)?;
    }

    let kernel_bin = if arch == "aarch64" {
        "kernel64_rust.bin"
    } else {
        "target/x86_64-unknown-none/release/levitate-kernel"
    };

    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    let profile = if arch == "aarch64" {
        QemuProfile::Default
    } else {
        QemuProfile::X86_64
    };

    let machine = profile.machine();
    let args = vec![
        "-M", machine.as_str(),
        "-cpu", profile.cpu(),
        "-m", profile.memory(),
        "-kernel", kernel_bin,
        "-nographic",  // No display, stdin goes to serial
        "-device", "virtio-gpu-pci,xres=1280,yres=800",
        "-device", if arch == "x86_64" { "virtio-keyboard-pci" } else { "virtio-keyboard-device" },
        "-device", if arch == "x86_64" { "virtio-tablet-pci" } else { "virtio-tablet-device" },
        "-device", if arch == "x86_64" { "virtio-net-pci,netdev=net0" } else { "virtio-net-device,netdev=net0" },
        "-netdev", "user,id=net0",
        "-drive", "file=tinyos_disk.img,format=raw,if=none,id=hd0",
        "-device", if arch == "x86_64" { "virtio-blk-pci,drive=hd0" } else { "virtio-blk-device,drive=hd0" },
        "-initrd", "initramfs.cpio",
        "-serial", "mon:stdio",
        "-qmp", "unix:./qmp.sock,server,nowait",
        "-no-reboot",
    ];

    let mut args = args;
    if iso {
        args.retain(|&arg| arg != "-kernel" && arg != "-initrd" && arg != kernel_bin && arg != "initramfs.cpio");
        args.extend(["-cdrom", "levitate.iso", "-boot", "d"]);
    }

    // Clean QMP socket
    let _ = std::fs::remove_file("./qmp.sock");

    Command::new(qemu_bin)
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run QEMU")?;

    Ok(())
}

/// TEAM_283: Find a binary in PATH or common locations
pub fn find_binary(name: &str) -> Result<String> {
    if let Ok(output) = Command::new("which").arg(name).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(path);
            }
        }
    }
    
    // Check common local bins
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let local_bin = format!("{}/.local/bin/{}", home, name);
    if std::path::Path::new(&local_bin).exists() {
        return Ok(local_bin);
    }

    bail!("Binary '{}' not found in PATH or ~/.local/bin", name)
}
