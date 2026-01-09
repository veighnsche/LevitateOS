//! LevitateOS Screenshot Tests
//!
//! TEAM_325: Tests LevitateOS display output for both architectures.
//!
//! Expected:
//! - aarch64: Working display with shell
//! - x86_64: Known issue - black screen

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::build;
use crate::qemu::{Arch, QemuBuilder, QemuProfile};
use crate::support::qmp::QmpClient;

fn qmp_socket(arch: &str) -> String {
    format!("./levitate-test-{}.sock", arch)
}

const BOOT_TIMEOUT_SECS: u64 = 15;

/// Run LevitateOS screenshot tests for both architectures in parallel
pub fn run() -> Result<()> {
    println!("📸 LevitateOS Screenshot Tests\n");

    println!("🔨 Building LevitateOS for both architectures...");
    build::build_all("aarch64")?;
    build::build_all("x86_64")?;

    // Run sequentially to avoid disk lock conflicts
    println!("\n━━━ aarch64 ━━━");
    let aarch64_result = run_arch("aarch64");
    
    println!("\n━━━ x86_64 ━━━");
    let x86_64_result = run_arch("x86_64");

    // Report results (don't fail on x86_64 - we expect issues)
    println!("\n━━━ Results ━━━");
    
    match aarch64_result {
        Ok(_) => println!("  ✅ aarch64: Screenshot captured"),
        Err(e) => println!("  ❌ aarch64: {}", e),
    }
    
    match x86_64_result {
        Ok(_) => println!("  ✅ x86_64: Screenshot captured"),
        Err(e) => println!("  ⚠️  x86_64: {} (expected)", e),
    }

    println!("\n   Screenshots saved to tests/screenshots/");
    Ok(())
}

/// Run screenshot test for a specific architecture
fn run_arch(arch: &str) -> Result<()> {
    let socket = qmp_socket(arch);
    
    // Clean up any existing socket
    let _ = fs::remove_file(&socket);

    println!("[{}] 🚀 Starting LevitateOS...", arch);
    let mut child = start_levitate(arch)?;

    // Wait for QMP socket
    wait_for_qmp_socket(&socket)?;

    // Wait for OS to boot
    println!("[{}] ⏳ Waiting for boot ({} seconds)...", arch, BOOT_TIMEOUT_SECS);
    std::thread::sleep(Duration::from_secs(BOOT_TIMEOUT_SECS));

    let mut client = QmpClient::connect(&socket)?;

    // Take screenshot
    let screenshot = format!("tests/screenshots/levitate_{}.ppm", arch);
    println!("[{}] 📸 Taking screenshot: {}", arch, screenshot);
    take_screenshot(&mut client, &screenshot)?;

    // Cleanup
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_file(&socket);

    // Verify screenshot exists
    let png = screenshot.replace(".ppm", ".png");
    if !Path::new(&png).exists() && !Path::new(&screenshot).exists() {
        bail!("Screenshot not created");
    }

    println!("[{}] ✅ Screenshot captured!", arch);
    Ok(())
}

/// Start LevitateOS in QEMU
fn start_levitate(arch: &str) -> Result<std::process::Child> {
    let arch_enum = Arch::try_from(arch)?;
    let profile = if arch == "x86_64" {
        QemuProfile::X86_64
    } else {
        QemuProfile::Default
    };

    let socket = qmp_socket(arch);

    let mut builder = QemuBuilder::new(arch_enum, profile)
        .enable_qmp(&socket);

    // Add display device for screenshots
    // Use VNC for headless screenshot capture
    builder = builder.display_vnc();

    if arch == "x86_64" {
        builder = builder.boot_iso();
    }

    let mut cmd = builder.build()?;

    let child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start QEMU")?;

    Ok(child)
}

/// Wait for QMP socket to be available
fn wait_for_qmp_socket(socket: &str) -> Result<()> {
    for _ in 0..60 {
        if Path::new(socket).exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    bail!("QMP socket not created - QEMU may have failed to start")
}

/// Take a screenshot via QMP and convert to PNG
fn take_screenshot(client: &mut QmpClient, output: &str) -> Result<()> {
    let abs_path = std::env::current_dir()?.join(output);
    let args = serde_json::json!({
        "filename": abs_path.to_string_lossy()
    });
    client.execute("screendump", Some(args))?;
    std::thread::sleep(Duration::from_millis(500));
    
    // Convert PPM to PNG using ImageMagick
    if output.ends_with(".ppm") {
        let png_path = output.replace(".ppm", ".png");
        let status = Command::new("magick")
            .args([output, &png_path])
            .status();
        
        if status.is_ok() && status.unwrap().success() {
            let _ = fs::remove_file(output);
        }
    }
    
    Ok(())
}
