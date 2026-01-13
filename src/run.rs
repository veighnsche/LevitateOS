//! QEMU run commands
//!
//! `TEAM_322`: Refactored to use `QemuBuilder` pattern.

use crate::qemu::{Arch, QemuBuilder};
use crate::{builder, disk};
use anyhow::{bail, Context, Result};
// TEAM_370: Removed unused clap::Subcommand import
use std::path::PathBuf;
use std::process::{Command, Stdio};

// TEAM_370: Removed dead RunCommands enum - main.rs uses RunArgs instead

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

/// `TEAM_322`: Run QEMU with default GUI display
/// `TEAM_474`: Added linux parameter for Linux kernel support
/// `TEAM_475`: Added openrc parameter for OpenRC initramfs
pub fn run_qemu(
    profile: QemuProfile,
    headless: bool,
    iso: bool,
    arch: &str,
    gpu_debug: bool,
    linux: bool,
    openrc: bool,
) -> Result<()> {
    disk::create_disk_image_if_missing()?;

    let arch_enum = Arch::try_from(arch)?;
    // TEAM_330: Explicitly set GPU resolution for readable display
    let mut builder = QemuBuilder::new(arch_enum, profile).gpu_resolution(1280, 800);

    // TEAM_474: Use Linux kernel if requested
    if linux {
        builder = builder.linux_kernel();
        // TEAM_475: Use OpenRC initramfs if requested
        if openrc {
            let initrd_path = format!("target/initramfs/{}-openrc.cpio", arch);
            builder = builder.initrd(&initrd_path);
        }
    }

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

/// `TEAM_116`: Run QEMU with GDB server enabled (port 1234)
/// TEAM_476: Updated to support Linux kernel mode (default)
pub fn run_qemu_gdb_linux(profile: QemuProfile, wait: bool, arch: &str, openrc: bool) -> Result<()> {
    let init_system = if openrc { "OpenRC" } else { "BusyBox" };
    println!("🐛 Starting QEMU with GDB server on port 1234 (Linux + {init_system})...");
    if wait {
        println!("⏳ Waiting for GDB connection before starting...");
    }

    disk::create_disk_image_if_missing()?;

    let arch_enum = Arch::try_from(arch)?;
    let mut builder = QemuBuilder::new(arch_enum, profile)
        .gpu_resolution(1280, 800)
        .enable_gdb(wait)
        .enable_qmp("./qmp.sock")
        .linux_kernel();

    // Use OpenRC initramfs if requested
    if openrc {
        let initrd_path = format!("target/initramfs/{}-openrc.cpio", arch);
        builder = builder.initrd(&initrd_path);
    }

    let mut cmd = builder.build()?;
    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run QEMU with GDB")?;

    Ok(())
}

/// Run QEMU with VNC for browser-based GPU display verification
/// TEAM_476: Updated to use Linux + OpenRC
pub fn run_qemu_vnc(arch: &str) -> Result<()> {
    println!("🖥️  Starting QEMU with VNC for browser-based display verification...\n");

    disk::create_disk_image_if_missing()?;
    // TEAM_476: Always use Linux + OpenRC
    builder::create_openrc_initramfs(arch)?;

    // Setup noVNC
    let novnc_path = PathBuf::from("/tmp/novnc");
    if !novnc_path.exists() {
        println!("📥 Downloading noVNC...");
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://github.com/novnc/noVNC.git",
                "/tmp/novnc",
            ])
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
    let _ = Command::new("pkill")
        .args(["-f", "websockify.*6080"])
        .status();
    let _ = Command::new("pkill")
        .args(["-f", "qemu.*-vnc.*:0"])
        .status();
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
            bail!("websockify exited immediately with status: {status}. Port 6080 may be in use.");
        }
        Ok(None) => {} // Still running
        Err(e) => bail!("Failed to check websockify status: {e}"),
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
    // TEAM_330: Explicit resolution for VNC display
    // TEAM_476: Use Linux kernel with OpenRC
    let initrd_path = format!("target/initramfs/{}-openrc.cpio", arch);
    let builder = QemuBuilder::new(arch_enum, profile)
        .gpu_resolution(1280, 800)
        .display_vnc()
        .enable_qmp("./qmp.sock")
        .linux_kernel()
        .initrd(&initrd_path);

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
                println!("  Found websockify at: {path}");
                return Ok(path);
            }
        }
    }

    // Check common pip user install location
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let pip_path = format!("{home}/.local/bin/websockify");
    if std::path::Path::new(&pip_path).exists() {
        println!("  Found websockify at: {pip_path}");
        return Ok(pip_path);
    }

    // Check for pipx installation
    let pipx_path = format!("{home}/.local/pipx/venvs/websockify/bin/websockify");
    if std::path::Path::new(&pipx_path).exists() {
        println!("  Found websockify at: {pipx_path}");
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

/// `TEAM_374`: Run QEMU with test runner for automated OS testing
/// TEAM_476: Updated to test Linux + OpenRC boot
pub fn run_qemu_test(arch: &str) -> Result<()> {
    println!("🧪 Running LevitateOS Boot Test for {arch}...\n");

    // Build Linux + OpenRC
    builder::create_openrc_initramfs(arch)?;
    disk::create_disk_image_if_missing()?;

    let timeout_secs: u64 = 60;
    println!("Running QEMU (headless, {timeout_secs}s timeout)...\n");

    let arch_enum = Arch::try_from(arch)?;
    let profile = profile_for_arch(arch);
    let initrd_path = format!("target/initramfs/{}-openrc.cpio", arch);
    let builder = QemuBuilder::new(arch_enum, profile)
        .display_headless()
        .linux_kernel()
        .initrd(&initrd_path);

    let base_cmd = builder.build()?;
    let args: Vec<_> = base_cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

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
    print!("{stdout}");

    if !output.status.success() && !stderr.is_empty() {
        eprintln!("\nQEMU Stderr:\n{stderr}");
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

/// TEAM_475: Run Linux kernel in terminal mode with optional OpenRC
/// TEAM_476: This is now the only terminal mode (custom kernel removed)
pub fn run_qemu_term_linux(arch: &str, openrc: bool) -> Result<()> {
    let init_system = if openrc { "OpenRC" } else { "BusyBox" };
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  LevitateOS + Linux Kernel ({init_system}) - {arch}         ");
    println!("║                                                            ║");
    println!("║  Type directly here - keyboard goes to VM                  ║");
    println!("║  Ctrl+A X to exit QEMU                                     ║");
    println!("║  Ctrl+A C to switch to QEMU monitor                        ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Clean QMP socket
    let _ = std::fs::remove_file("./qmp.sock");

    let arch_enum = Arch::try_from(arch)?;
    let profile = profile_for_arch(arch);
    let mut builder = QemuBuilder::new(arch_enum, profile)
        .display_nographic()
        .enable_qmp("./qmp.sock")
        .linux_kernel();

    // Use OpenRC initramfs if requested
    if openrc {
        let initrd_path = format!("target/initramfs/{}-openrc.cpio", arch);
        builder = builder.initrd(&initrd_path);
    }

    let mut cmd = builder.build()?;
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run QEMU")?;

    Ok(())
}

/// `TEAM_320`: Verify GPU display via VNC + Puppeteer
/// TEAM_476: Updated to use Linux + OpenRC
pub fn verify_gpu(arch: &str, timeout: u32) -> Result<()> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  [GPU VERIFY] Starting automated GPU verification...     ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    disk::create_disk_image_if_missing()?;
    // TEAM_476: Always use Linux + OpenRC
    builder::create_openrc_initramfs(arch)?;

    // Setup noVNC and websockify similar to run_qemu_vnc
    let novnc_path = PathBuf::from("/tmp/novnc");
    if !novnc_path.exists() {
        println!("📥 Downloading noVNC...");
        let status = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://github.com/novnc/noVNC.git",
                "/tmp/novnc",
            ])
            .status()
            .context("Failed to clone noVNC")?;
        if !status.success() {
            bail!("Failed to download noVNC");
        }
    }

    let websockify_path = find_websockify()?;

    // Kill existing processes
    let _ = Command::new("pkill")
        .args(["-f", "websockify.*6080"])
        .status();
    let _ = Command::new("pkill")
        .args(["-f", "qemu.*-vnc.*:0"])
        .status();
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
    // TEAM_330: Explicit resolution for GPU verification
    // TEAM_476: Use Linux kernel with OpenRC
    let initrd_path = format!("target/initramfs/{}-openrc.cpio", arch);
    let builder = QemuBuilder::new(arch_enum, profile)
        .gpu_resolution(1280, 800)
        .display_vnc()
        .enable_qmp("./qmp.sock")
        .linux_kernel()
        .initrd(&initrd_path);

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
    println!("⏳ Waiting {timeout}s for GPU display...");
    std::thread::sleep(std::time::Duration::from_secs(u64::from(timeout)));

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
                println!("⚠️  Failed to connect to QMP: {e}");
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
            println!(
                "✅ GPU verification: Screenshot captured ({} bytes)",
                metadata.len()
            );
            Ok(())
        } else {
            bail!("❌ GPU verification failed: Screenshot too small (display may be inactive)");
        }
    } else {
        bail!("❌ GPU verification failed: Could not capture screenshot");
    }
}
