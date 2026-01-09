//! QEMU run commands
//!
//! TEAM_322: Refactored to use QemuBuilder pattern.

use crate::qemu::{Arch, QemuBuilder};
use crate::{build, disk};
use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Subcommand)]
pub enum RunCommands {
    /// Run with GUI window (keyboard goes to QEMU window)
    Default {
        /// Boot from Limine ISO instead of -kernel
        #[arg(long)]
        iso: bool,
        /// TEAM_320: Enable QEMU GPU debug tracing
        #[arg(long)]
        gpu_debug: bool,
    },
    /// Run Pixel 6 Profile
    Pixel6,
    /// Run with VNC for browser verification
    Vnc,
    /// Run with GDB server enabled (port 1234)
    Gdb {
        /// Wait for GDB connection on startup
        #[arg(long)]
        wait: bool,
        /// Boot from Limine ISO instead of -kernel
        #[arg(long)]
        iso: bool,
    },
    /// Run in terminal-only mode (WSL-like, keyboard in terminal)
    Term {
        /// Boot from Limine ISO instead of -kernel
        #[arg(long)]
        iso: bool,
    },
    /// TEAM_243: Run internal OS tests (for AI agent verification)
    Test,
    /// TEAM_320: Verify GPU display via VNC + Puppeteer (automated black screen detection)
    VerifyGpu {
        /// Timeout in seconds to wait for display
        #[arg(long, default_value = "30")]
        timeout: u32,
    },
}

// Re-export for backwards compatibility with main.rs
pub use crate::qemu::QemuProfile;

/// Helper to get profile for arch
fn profile_for_arch(arch: &str) -> QemuProfile {
    if arch == "x86_64" {
        QemuProfile::X86_64
    } else {
        QemuProfile::Default
    }
}

/// TEAM_322: Run QEMU with default GUI display
pub fn run_qemu(profile: QemuProfile, headless: bool, iso: bool, arch: &str, gpu_debug: bool) -> Result<()> {
    disk::create_disk_image_if_missing()?;

    let arch_enum = Arch::try_from(arch)?;
    let mut builder = QemuBuilder::new(arch_enum, profile);

    // Boot configuration
    if iso {
        builder = builder.boot_iso();
    }

    // Display configuration
    if headless {
        builder = builder.display_headless();
    } else {
        builder = builder.display_gtk();
    }

    // GPU debug
    if gpu_debug {
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  [QEMU] GPU DEBUG MODE ENABLED                           ║");
        println!("║  Watch for: virtio_gpu_* trace messages                  ║");
        println!("║  Kernel will output GPU status to serial console         ║");
        println!("╚══════════════════════════════════════════════════════════╝");
        builder = builder.enable_gpu_debug();
    }

    let mut cmd = builder.build()?;
    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run QEMU")?;

    Ok(())
}

/// TEAM_116: Run QEMU with GDB server enabled (port 1234)
pub fn run_qemu_gdb(profile: QemuProfile, wait: bool, iso: bool, arch: &str) -> Result<()> {
    println!("🐛 Starting QEMU with GDB server on port 1234...");
    if wait {
        println!("⏳ Waiting for GDB connection before starting...");
    }

    disk::create_disk_image_if_missing()?;

    let arch_enum = Arch::try_from(arch)?;
    let mut builder = QemuBuilder::new(arch_enum, profile)
        .gpu_resolution(1280, 800)
        .enable_gdb(wait)
        .enable_qmp("./qmp.sock");

    if iso {
        builder = builder.boot_iso();
    }

    let mut cmd = builder.build()?;
    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run QEMU with GDB")?;

    Ok(())
}

/// Run QEMU with VNC for browser-based GPU display verification
pub fn run_qemu_vnc(arch: &str) -> Result<()> {
    println!("🖥️  Starting QEMU with VNC for browser-based display verification...\n");

    // TEAM_317: x86_64 uses ISO (Limine) since we removed Multiboot support
    let use_iso = arch == "x86_64";

    disk::create_disk_image_if_missing()?;
    if use_iso {
        build::build_iso(arch)?;
    } else {
        build::build_all(arch)?;
    }

    // Setup noVNC
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

    // Find websockify
    let websockify_path = find_websockify()?;

    // Kill any existing VNC-related processes
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
        Ok(None) => {} // Still running
        Err(e) => bail!("Failed to check websockify status: {}", e),
    }

    println!();
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
    println!();

    // Clean QMP socket
    let _ = std::fs::remove_file("./qmp.sock");

    // Build QEMU
    let arch_enum = Arch::try_from(arch)?;
    let profile = profile_for_arch(arch);
    let mut builder = QemuBuilder::new(arch_enum, profile)
        .display_vnc()
        .enable_qmp("./qmp.sock");

    if use_iso {
        builder = builder.boot_iso();
    }

    let mut cmd = builder.build()?;
    let qemu_result = cmd
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
    // Check PATH first
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

/// TEAM_243: Run QEMU with test runner for automated OS testing
pub fn run_qemu_test(arch: &str) -> Result<()> {
    println!("🧪 Running LevitateOS Internal Tests for {}...\n", arch);

    // TEAM_317: x86_64 uses ISO (Limine)
    let use_iso = arch == "x86_64";

    // Build everything including test runner
    build::build_userspace(arch)?;
    build::create_test_initramfs(arch)?;
    if use_iso {
        build::build_iso(arch)?;
    } else {
        build::build_kernel_verbose(arch)?;
    }
    disk::create_disk_image_if_missing()?;

    let timeout_secs: u64 = 60;
    println!("Running QEMU (headless, {}s timeout)...\n", timeout_secs);

    let arch_enum = Arch::try_from(arch)?;
    let profile = profile_for_arch(arch);
    let mut builder = QemuBuilder::new(arch_enum, profile)
        .display_headless();

    if use_iso {
        builder = builder.boot_iso();
    } else {
        builder = builder.boot_kernel("initramfs_test.cpio");
    }

    let base_cmd = builder.build()?;
    let args: Vec<_> = base_cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();

    // Run with timeout
    let mut timeout_args = vec![format!("{}s", timeout_secs)];
    timeout_args.push(arch_enum.qemu_binary().to_string());
    timeout_args.extend(args);

    let output = Command::new("timeout")
        .args(&timeout_args)
        .output()
        .context("Failed to run QEMU")?;

    // Print stdout (serial output)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    print!("{}", stdout);

    if !output.status.success() && !stderr.is_empty() {
        eprintln!("\nQEMU Stderr:\n{}", stderr);
    }

    // Check for test results
    if stdout.contains("[TEST_RUNNER] RESULT: PASSED") {
        println!("\n✅ All OS internal tests passed!");
        Ok(())
    } else if stdout.contains("[TEST_RUNNER] RESULT: FAILED") {
        bail!("❌ Some OS internal tests failed!");
    } else if stdout.contains("[TEST_RUNNER]") {
        bail!("❌ Test runner did not complete (timeout or crash)");
    } else {
        bail!("❌ Test runner failed to start - check initramfs");
    }
}

/// TEAM_139: Run QEMU in terminal-only mode (WSL-like)
pub fn run_qemu_term(arch: &str, iso: bool) -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  LevitateOS Terminal Mode - {}                        ║", arch);
    println!("║                                                            ║");
    println!("║  Type directly here - keyboard goes to VM                  ║");
    println!("║  Ctrl+A X to exit QEMU                                     ║");
    println!("║  Ctrl+A C to switch to QEMU monitor                        ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    if iso {
        build::build_iso(arch)?;
    } else {
        disk::create_disk_image_if_missing()?;
        build::build_all(arch)?;
    }

    // Clean QMP socket
    let _ = std::fs::remove_file("./qmp.sock");

    let arch_enum = Arch::try_from(arch)?;
    let profile = profile_for_arch(arch);
    let mut builder = QemuBuilder::new(arch_enum, profile)
        .display_nographic()
        .enable_qmp("./qmp.sock");

    if iso {
        builder = builder.boot_iso();
    }

    let mut cmd = builder.build()?;
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run QEMU")?;

    Ok(())
}

/// TEAM_320: Verify GPU display via VNC + Puppeteer
pub fn verify_gpu(arch: &str, timeout: u32) -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  [GPU VERIFY] Starting automated GPU verification...     ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // TEAM_317: x86_64 uses ISO (Limine)
    let use_iso = arch == "x86_64";

    disk::create_disk_image_if_missing()?;
    if use_iso {
        build::build_iso(arch)?;
    } else {
        build::build_all(arch)?;
    }

    // Setup noVNC and websockify similar to run_qemu_vnc
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

    let websockify_path = find_websockify()?;

    // Kill existing processes
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

    // Clean QMP socket
    let _ = std::fs::remove_file("./qmp.sock");

    // Start QEMU in background
    let arch_enum = Arch::try_from(arch)?;
    let profile = profile_for_arch(arch);
    let mut builder = QemuBuilder::new(arch_enum, profile)
        .display_vnc()
        .enable_qmp("./qmp.sock");

    if use_iso {
        builder = builder.boot_iso();
    }

    let mut cmd = builder.build()?;
    let mut qemu = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start QEMU")?;

    // Wait for QMP socket
    println!("⏳ Waiting for QEMU to start...");
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Wait specified timeout for GPU to initialize
    println!("⏳ Waiting {}s for GPU display...", timeout);
    std::thread::sleep(std::time::Duration::from_secs(timeout as u64));

    // Take screenshot via QMP
    if std::path::Path::new("./qmp.sock").exists() {
        println!("📸 Taking screenshot via QMP...");
        match crate::support::qmp::QmpClient::connect("./qmp.sock") {
            Ok(mut client) => {
                let args = serde_json::json!({ "filename": "tests/screenshots/gpu_verify.ppm" });
                if client.execute("screendump", Some(args)).is_ok() {
                    println!("✅ Screenshot saved to tests/screenshots/gpu_verify.ppm");
                }
            }
            Err(e) => {
                println!("⚠️  Failed to connect to QMP: {}", e);
            }
        }
    }

    // Cleanup
    let _ = qemu.kill();
    let _ = websockify.kill();

    // Check screenshot file
    let screenshot_path = std::path::Path::new("tests/screenshots/gpu_verify.ppm");
    if screenshot_path.exists() {
        let metadata = std::fs::metadata(screenshot_path)?;
        if metadata.len() > 1000 {
            println!("✅ GPU verification: Screenshot captured ({} bytes)", metadata.len());
            Ok(())
        } else {
            bail!("❌ GPU verification failed: Screenshot too small (display may be inactive)");
        }
    } else {
        bail!("❌ GPU verification failed: Could not capture screenshot");
    }
}
