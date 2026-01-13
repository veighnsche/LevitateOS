#![allow(dead_code)]
//! Alpine Linux Screenshot Integration Tests
//!
//! `TEAM_325`: Tests screenshot functionality using Alpine Linux.
//!
//! For each architecture:
//! 1. Boot Alpine Linux in live mode
//! 2. Wait for shell prompt
//! 3. Show time and take screenshot
//! 4. Run `ls` and take second screenshot
//! 5. Cleanup

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::support::qmp::QmpClient;

fn qmp_socket(arch: &str) -> String {
    format!("./alpine-test-{arch}.sock")
}
const ALPINE_VERSION: &str = "3.20.0";
const BOOT_TIMEOUT_SECS: u64 = 30;

/// Run Alpine screenshot tests for both architectures in parallel
pub fn run() -> Result<()> {
    println!("📸 Alpine Linux Screenshot Tests\n");

    // Check for Alpine images
    check_alpine_images()?;

    println!("🚀 Running both architectures in parallel...\n");

    // Run both architectures in parallel using threads
    let aarch64_handle = std::thread::spawn(|| run_arch("aarch64"));
    let x86_64_handle = std::thread::spawn(|| run_arch("x86_64"));

    // Wait for both to complete
    let aarch64_result = aarch64_handle
        .join()
        .map_err(|_| anyhow::anyhow!("aarch64 thread panicked"))?;
    let x86_64_result = x86_64_handle
        .join()
        .map_err(|_| anyhow::anyhow!("x86_64 thread panicked"))?;

    // Check results
    aarch64_result?;
    x86_64_result?;

    println!("\n✅ All Alpine screenshot tests passed!");
    println!("   Screenshots saved to tests/screenshots/");
    Ok(())
}

/// Check that Alpine images exist
fn check_alpine_images() -> Result<()> {
    let x86_iso = format!("tests/images/alpine-virt-{ALPINE_VERSION}-x86_64.iso");
    let arm_iso = format!("tests/images/alpine-virt-{ALPINE_VERSION}-aarch64.iso");

    if !Path::new(&x86_iso).exists() || !Path::new(&arm_iso).exists() {
        bail!(
            "Alpine images not found. Run:\n  \
             ./tests/images/download.sh"
        );
    }
    Ok(())
}

/// Run screenshot test for a specific architecture
fn run_arch(arch: &str) -> Result<()> {
    let socket = qmp_socket(arch);

    // Clean up any existing socket
    let _ = fs::remove_file(&socket);

    // Start Alpine
    println!("[{arch}] 🚀 Starting Alpine Linux...");
    let mut child = start_alpine(arch)?;

    // Wait for QMP socket
    wait_for_qmp_socket(&socket)?;

    // Wait for QEMU to initialize
    std::thread::sleep(Duration::from_secs(3));

    let mut client = QmpClient::connect(&socket)?;

    // x86_64 Alpine uses ISOLINUX which waits at "boot:" prompt
    if arch == "x86_64" {
        println!("  ⏳ Waiting for ISOLINUX boot prompt (5s)...");
        std::thread::sleep(Duration::from_secs(5));
        println!("  ⏎ Pressing Enter at ISOLINUX boot prompt...");
        send_key(&mut client, "ret")?;
    }

    // Wait for Alpine to boot and show login prompt
    println!("  ⏳ Waiting for boot ({BOOT_TIMEOUT_SECS} seconds)...");
    std::thread::sleep(Duration::from_secs(BOOT_TIMEOUT_SECS));

    // Login as root (no password in live mode)
    println!("  🔑 Logging in as root...");
    send_keys(&mut client, "root")?;
    send_key(&mut client, "ret")?;
    std::thread::sleep(Duration::from_secs(3));

    // Show date/time
    println!("  ⏰ Showing time...");
    send_keys(&mut client, "date")?;
    send_key(&mut client, "ret")?;
    std::thread::sleep(Duration::from_secs(2));

    // Take first screenshot (shell with time)
    let screenshot1 = format!("tests/screenshots/alpine_{arch}_shell.ppm");
    println!("  📸 Taking screenshot 1: {screenshot1}");
    take_screenshot(&mut client, &screenshot1)?;

    // Run ls command
    println!("  📂 Running ls...");
    send_keys(&mut client, "ls -la /")?;
    send_key(&mut client, "ret")?;
    std::thread::sleep(Duration::from_secs(2));

    // Take second screenshot (ls output)
    let screenshot2 = format!("tests/screenshots/alpine_{arch}_ls.ppm");
    println!("  📸 Taking screenshot 2: {screenshot2}");
    take_screenshot(&mut client, &screenshot2)?;

    // Cleanup
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_file(&socket);

    // Verify screenshots exist (check PNG first, then PPM fallback)
    let png1 = screenshot1.replace(".ppm", ".png");
    let png2 = screenshot2.replace(".ppm", ".png");
    if !Path::new(&png1).exists() && !Path::new(&screenshot1).exists() {
        bail!("Screenshot 1 not created: {screenshot1}");
    }
    if !Path::new(&png2).exists() && !Path::new(&screenshot2).exists() {
        bail!("Screenshot 2 not created: {screenshot2}");
    }

    println!("  ✅ {arch} screenshots captured!");
    Ok(())
}

/// Start Alpine Linux in QEMU
fn start_alpine(arch: &str) -> Result<std::process::Child> {
    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => bail!("Unsupported architecture: {arch}"),
    };

    let iso_path = format!("tests/images/alpine-virt-{ALPINE_VERSION}-{arch}.iso");
    let qmp_arg = format!("unix:{},server,nowait", qmp_socket(arch));

    let mut args: Vec<String> = vec![
        "-m".into(),
        "512M".into(),
        "-display".into(),
        "none".into(), // No window, but still have framebuffer
        "-serial".into(),
        "mon:stdio".into(),
        "-qmp".into(),
        qmp_arg,
        "-cdrom".into(),
        iso_path,
        "-boot".into(),
        "d".into(),
    ];

    // Architecture-specific args
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
            "std".into(), // Use standard VGA for BIOS compatibility
        ]);
    }

    let child = Command::new(qemu_bin)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit()) // Show errors for debugging
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
        let status = Command::new("magick").args([output, &png_path]).status();

        if status.is_ok() && status.unwrap().success() {
            let _ = fs::remove_file(output); // Remove PPM after successful conversion
        }
    }

    Ok(())
}

/// Send a string as keystrokes
fn send_keys(client: &mut QmpClient, text: &str) -> Result<()> {
    for ch in text.chars() {
        send_char(client, ch)?;
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}

/// Send a single key via human-monitor-command
fn send_key(client: &mut QmpClient, qcode: &str) -> Result<()> {
    let cmd = format!("sendkey {qcode}");
    let args = serde_json::json!({
        "command-line": cmd
    });
    client.execute("human-monitor-command", Some(args))?;
    Ok(())
}

/// Send a character as a keypress
fn send_char(client: &mut QmpClient, ch: char) -> Result<()> {
    let (key, needs_shift) = char_to_qcode(ch);

    if needs_shift {
        // Send shift+key
        let cmd = format!("sendkey shift-{key}");
        let args = serde_json::json!({
            "command-line": cmd
        });
        client.execute("human-monitor-command", Some(args))?;
    } else {
        send_key(client, key)?;
    }

    Ok(())
}

/// Convert a character to QEMU qcode
fn char_to_qcode(ch: char) -> (&'static str, bool) {
    match ch {
        'a'..='z' => {
            let idx = (ch as u8 - b'a') as usize;
            let keys = [
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            ];
            (keys[idx], false)
        }
        'A'..='Z' => {
            let idx = (ch as u8 - b'A') as usize;
            let keys = [
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            ];
            (keys[idx], true)
        }
        '0'..='9' => {
            let idx = (ch as u8 - b'0') as usize;
            let keys = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
            (keys[idx], false)
        }
        ' ' => ("spc", false),
        '\n' => ("ret", false),
        '-' => ("minus", false),
        '/' => ("slash", false),
        '.' => ("dot", false),
        _ => ("spc", false),
    }
}
