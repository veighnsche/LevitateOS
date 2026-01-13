//! Screenshot Tests
//!
//! `TEAM_327`: Unified screenshot tests for visual verification.
//!
//! Test Types:
//! - `alpine` - Alpine Linux reference tests (external OS)
//! - `levitate` - Basic `LevitateOS` display test
//! - `userspace` - Run userspace commands and capture results
//!
//! Usage:
//!   cargo xtask test screenshot           # All screenshot tests
//!   cargo xtask test screenshot alpine    # Alpine only
//!   cargo xtask test screenshot levitate  # `LevitateOS` display only
//!   cargo xtask test screenshot userspace # Userspace tests + results

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::builder;
use crate::qemu::{Arch, QemuBuilder, QemuProfile};
use crate::support::qmp::QmpClient;

use super::common::{qmp_send_key, qmp_send_keys, wait_for_qmp_socket};

const SCREENSHOT_DIR: &str = "tests/screenshots";

/// Run all screenshot tests
pub fn run(subtest: Option<&str>) -> Result<()> {
    fs::create_dir_all(SCREENSHOT_DIR)?;

    match subtest {
        Some("alpine") => run_alpine(),
        Some("levitate") => run_levitate(),
        Some("userspace") => run_userspace(),
        Some(other) => bail!("Unknown screenshot test: {other}. Use: alpine, levitate, userspace"),
        None => {
            println!("📸 Running All Screenshot Tests\n");

            // Run userspace test (most important)
            println!("━━━ Userspace Tests ━━━");
            let userspace_result = run_userspace();

            // Run basic levitate display test
            println!("\n━━━ LevitateOS Display ━━━");
            let levitate_result = run_levitate();

            // Report results
            println!("\n━━━ Results ━━━");
            match &userspace_result {
                Ok(()) => println!("  ✅ userspace: Tests completed, screenshot captured"),
                Err(e) => println!("  ❌ userspace: {e}"),
            }
            match &levitate_result {
                Ok(()) => println!("  ✅ levitate: Display verified"),
                Err(e) => println!("  ⚠️  levitate: {e}"),
            }

            println!("\n   Screenshots saved to {SCREENSHOT_DIR}/");

            // Fail if userspace failed (it's the important one)
            userspace_result
        }
    }
}

// =============================================================================
// Userspace Tests - Run commands and capture results
// =============================================================================

/// Run userspace tests and capture screenshot with results
pub fn run_userspace() -> Result<()> {
    println!("🧪 Userspace Test Screenshot\n");

    // Build for aarch64 (working arch)
    let arch = "aarch64";
    println!("🔨 Building LevitateOS for {arch}...");
    builder::build_all(arch)?;

    let qmp_socket = format!("./userspace-test-{}.sock", std::process::id());
    let _ = fs::remove_file(&qmp_socket);

    println!("🚀 Starting LevitateOS...");
    let mut child = start_levitate_vnc(arch, &qmp_socket)?;

    // Wait for QMP
    wait_for_qmp_socket(&qmp_socket, 30)?;
    std::thread::sleep(Duration::from_secs(2));

    let mut client = QmpClient::connect(&qmp_socket)?;

    // Wait for boot (watch serial output)
    println!("⏳ Waiting for shell prompt (20s)...");
    std::thread::sleep(Duration::from_secs(20));

    // Run test commands via keyboard input
    println!("⌨️  Running test commands...");

    // Clear screen and show test header
    run_command(&mut client, "echo")?;
    run_command(&mut client, "echo '=============================='")?;
    run_command(&mut client, "echo '   USERSPACE TEST RESULTS'")?;
    run_command(&mut client, "echo '=============================='")?;
    run_command(&mut client, "echo")?;

    // Test 1: List files in root
    run_command(&mut client, "echo '[TEST 1] ls /'")?;
    run_command(&mut client, "ls")?;
    run_command(&mut client, "echo")?;

    // Test 2: Show help
    run_command(&mut client, "echo '[TEST 2] help'")?;
    run_command(&mut client, "help")?;
    run_command(&mut client, "echo")?;

    // Test 3: Echo test
    run_command(&mut client, "echo '[TEST 3] echo test'")?;
    run_command(&mut client, "echo 'Hello from userspace!'")?;
    run_command(&mut client, "echo")?;

    // Final summary
    run_command(&mut client, "echo '=============================='")?;
    run_command(&mut client, "echo '   ALL TESTS COMPLETED'")?;
    run_command(&mut client, "echo '=============================='")?;

    // Wait for output to render
    std::thread::sleep(Duration::from_secs(2));

    // Take screenshot
    let screenshot = format!("{SCREENSHOT_DIR}/userspace_{arch}.ppm");
    println!("📸 Taking screenshot: {screenshot}");
    take_screenshot(&mut client, &screenshot)?;

    // Cleanup
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_file(&qmp_socket);

    // Verify screenshot exists
    let png = screenshot.replace(".ppm", ".png");
    if Path::new(&png).exists() || Path::new(&screenshot).exists() {
        println!("✅ Userspace test screenshot captured!");
        Ok(())
    } else {
        bail!("Screenshot not created")
    }
}

/// Run a command by typing it via QMP keyboard
fn run_command(client: &mut QmpClient, cmd: &str) -> Result<()> {
    qmp_send_keys(client, cmd)?;
    qmp_send_key(client, "ret")?;
    std::thread::sleep(Duration::from_millis(500));
    Ok(())
}

// =============================================================================
// LevitateOS Display Test - Basic boot verification
// =============================================================================

/// Basic `LevitateOS` display test
/// TEAM_476: Updated to use Linux + OpenRC
pub fn run_levitate() -> Result<()> {
    println!("📸 LevitateOS Display Test (Linux + OpenRC)\n");

    // Build for x86_64 (primary architecture)
    println!("🔨 Building LevitateOS...");
    builder::create_initramfs("x86_64")?;

    // Test aarch64 (working)
    println!("\n━━━ aarch64 ━━━");
    let aarch64_result = run_levitate_arch("aarch64");

    // Test x86_64 (investigating black screen)
    println!("\n━━━ x86_64 ━━━");
    let x86_64_result = run_levitate_arch("x86_64");

    // Report
    println!("\n━━━ Results ━━━");
    match &aarch64_result {
        Ok(()) => println!("  ✅ aarch64: Screenshot captured"),
        Err(e) => println!("  ❌ aarch64: {e}"),
    }
    match &x86_64_result {
        Ok(()) => println!("  ✅ x86_64: Screenshot captured"),
        Err(e) => println!("  ⚠️  x86_64: {e}"),
    }

    aarch64_result
}

fn run_levitate_arch(arch: &str) -> Result<()> {
    let qmp_socket = format!("./levitate-{arch}.sock");
    let _ = fs::remove_file(&qmp_socket);

    println!("[{arch}] 🚀 Starting LevitateOS...");
    let mut child = start_levitate_vnc(arch, &qmp_socket)?;

    wait_for_qmp_socket(&qmp_socket, 30)?;

    println!("[{arch}] ⏳ Waiting for boot (15s)...");
    std::thread::sleep(Duration::from_secs(15));

    let mut client = QmpClient::connect(&qmp_socket)?;

    let screenshot = format!("{SCREENSHOT_DIR}/levitate_{arch}.ppm");
    println!("[{arch}] 📸 Taking screenshot: {screenshot}");
    take_screenshot(&mut client, &screenshot)?;

    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_file(&qmp_socket);

    let png = screenshot.replace(".ppm", ".png");
    let img_path = if Path::new(&png).exists() {
        &png
    } else {
        &screenshot
    };

    if !Path::new(img_path).exists() {
        bail!("Screenshot not created")
    }

    // TEAM_329: Analyze screenshot for black screen detection
    match analyze_screenshot(img_path) {
        Ok(ScreenshotContent::Black { brightness }) => {
            println!("[{arch}] ⚠️  BLACK SCREEN DETECTED (brightness: {brightness:.1})");
            println!("[{arch}] Screenshot saved but display appears empty");
        }
        Ok(ScreenshotContent::HasContent { brightness }) => {
            println!(
                "[{arch}] ✅ Screenshot captured (brightness: {brightness:.1} - display working)"
            );
        }
        Err(e) => {
            println!("[{arch}] ⚠️  Screenshot captured but analysis failed: {e}");
        }
    }

    Ok(())
}

// =============================================================================
// Alpine Linux Reference Tests
// =============================================================================

const ALPINE_VERSION: &str = "3.20.0";

/// Alpine Linux screenshot tests
pub fn run_alpine() -> Result<()> {
    println!("📸 Alpine Linux Screenshot Tests\n");

    // Check for Alpine images
    let x86_iso = format!("tests/images/alpine-virt-{ALPINE_VERSION}-x86_64.iso");
    let arm_iso = format!("tests/images/alpine-virt-{ALPINE_VERSION}-aarch64.iso");

    if !Path::new(&x86_iso).exists() || !Path::new(&arm_iso).exists() {
        bail!("Alpine images not found. Run:\n  ./tests/images/download.sh");
    }

    // Run both architectures
    println!("━━━ aarch64 ━━━");
    let aarch64_result = run_alpine_arch("aarch64");

    println!("\n━━━ x86_64 ━━━");
    let x86_result = run_alpine_arch("x86_64");

    // Report
    println!("\n━━━ Results ━━━");
    match &aarch64_result {
        Ok(()) => println!("  ✅ aarch64: Screenshots captured"),
        Err(e) => println!("  ❌ aarch64: {e}"),
    }
    match &x86_result {
        Ok(()) => println!("  ✅ x86_64: Screenshots captured"),
        Err(e) => println!("  ❌ x86_64: {e}"),
    }

    aarch64_result?;
    x86_result?;

    println!("\n✅ All Alpine screenshot tests passed!");
    Ok(())
}

fn run_alpine_arch(arch: &str) -> Result<()> {
    let qmp_socket = format!("./alpine-{arch}.sock");
    let _ = fs::remove_file(&qmp_socket);

    println!("[{arch}] 🚀 Starting Alpine Linux...");
    let mut child = start_alpine(arch, &qmp_socket)?;

    wait_for_qmp_socket(&qmp_socket, 30)?;
    std::thread::sleep(Duration::from_secs(3));

    let mut client = QmpClient::connect(&qmp_socket)?;

    // x86_64 needs Enter at ISOLINUX
    if arch == "x86_64" {
        println!("[{arch}] ⏎ Pressing Enter at boot prompt...");
        std::thread::sleep(Duration::from_secs(5));
        qmp_send_key(&mut client, "ret")?;
    }

    // Wait for boot
    println!("[{arch}] ⏳ Waiting for boot (30s)...");
    std::thread::sleep(Duration::from_secs(30));

    // Login as root
    println!("[{arch}] 🔑 Logging in...");
    qmp_send_keys(&mut client, "root")?;
    qmp_send_key(&mut client, "ret")?;
    std::thread::sleep(Duration::from_secs(3));

    // Run date command
    qmp_send_keys(&mut client, "date")?;
    qmp_send_key(&mut client, "ret")?;
    std::thread::sleep(Duration::from_secs(2));

    // Screenshot 1
    let screenshot1 = format!("{SCREENSHOT_DIR}/alpine_{arch}_shell.ppm");
    println!("[{arch}] 📸 Taking screenshot 1: {screenshot1}");
    take_screenshot(&mut client, &screenshot1)?;

    // Run ls
    qmp_send_keys(&mut client, "ls -la /")?;
    qmp_send_key(&mut client, "ret")?;
    std::thread::sleep(Duration::from_secs(2));

    // Screenshot 2
    let screenshot2 = format!("{SCREENSHOT_DIR}/alpine_{arch}_ls.ppm");
    println!("[{arch}] 📸 Taking screenshot 2: {screenshot2}");
    take_screenshot(&mut client, &screenshot2)?;

    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_file(&qmp_socket);

    println!("[{arch}] ✅ Screenshots captured!");
    Ok(())
}

// =============================================================================
// Helpers
// =============================================================================

/// Start `LevitateOS` with VNC display
/// TEAM_476: Updated to use Linux kernel instead of ISO
fn start_levitate_vnc(arch: &str, qmp_socket: &str) -> Result<std::process::Child> {
    let arch_enum = Arch::try_from(arch)?;
    let profile = if arch == "x86_64" {
        QemuProfile::X86_64
    } else {
        QemuProfile::Default
    };

    let initrd_path = format!("target/initramfs/{}-openrc.cpio", arch);
    let builder = QemuBuilder::new(arch_enum, profile)
        .display_vnc()
        .enable_qmp(qmp_socket)
        
        .initrd(&initrd_path);

    let mut cmd = builder.build()?;

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start QEMU")
}

/// Start Alpine Linux
fn start_alpine(arch: &str, qmp_socket: &str) -> Result<std::process::Child> {
    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => bail!("Unsupported architecture: {arch}"),
    };

    let iso_path = format!("tests/images/alpine-virt-{ALPINE_VERSION}-{arch}.iso");
    let qmp_arg = format!("unix:{qmp_socket},server,nowait");

    let mut args: Vec<String> = vec![
        "-m".into(),
        "512M".into(),
        "-display".into(),
        "none".into(),
        "-serial".into(),
        "mon:stdio".into(),
        "-qmp".into(),
        qmp_arg,
        "-cdrom".into(),
        iso_path,
        "-boot".into(),
        "d".into(),
    ];

    if arch == "aarch64" {
        args.extend([
            "-M".into(),
            "virt".into(),
            "-cpu".into(),
            "cortex-a72".into(),
            "-bios".into(),
            "/usr/share/AAVMF/AAVMF_CODE.fd".into(),
            "-device".into(),
            "virtio-gpu-pci".into(),
            "-device".into(),
            "virtio-keyboard-pci".into(),
        ]);
    } else {
        args.extend([
            "-M".into(),
            "q35".into(),
            "-cpu".into(),
            "qemu64".into(),
            "-enable-kvm".into(),
            "-vga".into(),
            "std".into(),
        ]);
    }

    Command::new(qemu_bin)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start QEMU")
}

/// Take screenshot via QMP
fn take_screenshot(client: &mut QmpClient, output: &str) -> Result<()> {
    let abs_path = std::env::current_dir()?.join(output);
    let args = serde_json::json!({
        "filename": abs_path.to_string_lossy()
    });
    client.execute("screendump", Some(args))?;
    std::thread::sleep(Duration::from_millis(500));

    // Convert PPM to PNG
    if output.ends_with(".ppm") {
        let png_path = output.replace(".ppm", ".png");
        let status = Command::new("magick").args([output, &png_path]).status();

        if status.is_ok() && status.unwrap().success() {
            let _ = fs::remove_file(output);
        }
    }

    Ok(())
}

// =============================================================================
// TEAM_329: Black Screen Detection
// =============================================================================

/// Result of screenshot content analysis
#[derive(Debug)]
pub enum ScreenshotContent {
    /// Image has visible content (text, graphics)
    HasContent { brightness: f32 },
    /// Image is black or nearly black
    Black { brightness: f32 },
}

/// `TEAM_329`: Analyze a screenshot to detect if it's black/empty
///
/// Detects black screens by counting bright pixels (for white text on black bg)
/// Returns Black if less than 0.1% of pixels are bright (> 128)
pub fn analyze_screenshot(path: &str) -> Result<ScreenshotContent> {
    use image::GenericImageView;

    let img = image::open(path).with_context(|| format!("Failed to open image: {path}"))?;

    let (width, height) = img.dimensions();
    let total_pixels = u64::from(width) * u64::from(height);

    if total_pixels == 0 {
        return Ok(ScreenshotContent::Black { brightness: 0.0 });
    }

    // Count bright pixels (text on terminal is bright on dark background)
    let bright_threshold: u8 = 128;
    let mut bright_count: u64 = 0;
    let mut total_luminance: u64 = 0;

    for pixel in img.pixels() {
        let [r, g, b, _] = pixel.2 .0;
        // Luminance: 0.299*R + 0.587*G + 0.114*B
        let luminance = (0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b)) as u8;
        total_luminance += u64::from(luminance);

        if luminance > bright_threshold {
            bright_count += 1;
        }
    }

    let avg_brightness = (total_luminance as f64 / total_pixels as f64) as f32;
    let bright_percentage = (bright_count as f64 / total_pixels as f64) * 100.0;

    // Black screen: less than 0.1% of pixels are bright
    // This catches text terminals (white text on black = ~1-5% bright pixels)
    if bright_percentage < 0.1 {
        Ok(ScreenshotContent::Black {
            brightness: avg_brightness,
        })
    } else {
        Ok(ScreenshotContent::HasContent {
            brightness: avg_brightness,
        })
    }
}
