//! Keyboard Input Regression Test
//!
//! `TEAM_156`: Tests that keyboard input is correctly received WITHOUT dropping characters.
//! This test MUST FAIL if any characters are dropped.

use anyhow::bail;
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Test keyboard/serial input by sending characters and verifying EXACT echo
pub fn run(arch: &str) -> Result<()> {
    println!("⌨️  Testing keyboard input for {arch} (strict, no drops allowed)...\n");

    // First build everything
    crate::builder::build_all(arch)?;
    crate::disk::create_disk_image_if_missing()?;

    // Clean up
    let _ = std::fs::remove_file("./qmp.sock");

    let kernel_bin = if arch == "aarch64" {
        "kernel64_rust.bin"
    } else {
        "crates/kernel/target/x86_64-unknown-none/release/levitate-kernel"
    };

    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => bail!("Unsupported architecture: {arch}"),
    };

    let args = vec![
        "-M",
        if arch == "aarch64" { "virt" } else { "q35" },
        "-cpu",
        if arch == "aarch64" {
            "cortex-a72"
        } else {
            "qemu64"
        },
        "-m",
        "1G",
        "-kernel",
        kernel_bin,
        "-nographic",
        "-device",
        "virtio-gpu-pci,xres=1280,yres=800",
        "-device",
        "virtio-keyboard-device",
        "-device",
        "virtio-tablet-device",
        "-device",
        "virtio-net-device,netdev=net0",
        "-netdev",
        "user,id=net0",
        "-drive",
        "file=tinyos_disk.img,format=raw,if=none,id=hd0",
        "-device",
        "virtio-blk-device,drive=hd0",
        // TEAM_327: Use arch-specific initramfs
        "-initrd",
        "initramfs_aarch64.cpio",
        "-serial",
        "mon:stdio",
        "-qmp",
        "unix:./qmp.sock,server,nowait",
        "-no-reboot",
    ];

    println!("  Starting QEMU...");
    let mut child = Command::new(qemu_bin)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start QEMU")?;

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let mut stdout = child.stdout.take().expect("Failed to get stdout");

    // Set stdout to non-blocking
    use std::os::unix::io::AsRawFd;
    let fd = stdout.as_raw_fd();
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }

    let mut all_output = String::new();
    let mut buf = [0u8; 4096];

    // Wait for shell prompt
    println!("  Waiting for shell prompt...");
    let start = Instant::now();
    let timeout = Duration::from_secs(30);

    while start.elapsed() < timeout {
        match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                all_output.push_str(&chunk);
                if all_output.contains("# ") {
                    println!("  [OK] Shell prompt found");
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => break,
        }
    }

    if !all_output.contains("# ") {
        let _ = child.kill();
        anyhow::bail!("Shell prompt not found");
    }

    // Clear output buffer for test
    all_output.clear();

    // Give shell time to be ready
    std::thread::sleep(Duration::from_millis(200));

    // TEST 1: Single character test
    println!("\n  TEST 1: Single character input");
    let test_chars = "abcdefghij";
    for ch in test_chars.chars() {
        stdin.write_all(&[ch as u8])?;
        stdin.flush()?;
        std::thread::sleep(Duration::from_millis(50)); // Slow, one at a time
    }
    stdin.write_all(b"\n")?;
    stdin.flush()?;

    // Wait for response
    std::thread::sleep(Duration::from_millis(500));

    // Read output
    loop {
        match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                all_output.push_str(&chunk);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => break,
        }
    }

    // Check if ALL characters were echoed
    let test1_passed = all_output.contains(test_chars);
    println!("    Sent: {test_chars:?}");
    println!("    Looking for exact match in output...");
    if test1_passed {
        println!("    ✅ TEST 1 PASSED");
    } else {
        println!("    ❌ TEST 1 FAILED - characters dropped!");
        println!("    Output: {all_output:?}");
    }

    // TEST 2: Rapid burst input (the actual failure case)
    println!("\n  TEST 2: Rapid burst input (stress test)");
    all_output.clear();
    std::thread::sleep(Duration::from_millis(200));

    let burst = "QWERTYUIOP";
    // Send all at once - no delays
    stdin.write_all(burst.as_bytes())?;
    stdin.write_all(b"\n")?;
    stdin.flush()?;

    // Wait for response
    std::thread::sleep(Duration::from_millis(500));

    // Read output
    loop {
        match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                all_output.push_str(&chunk);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => break,
        }
    }

    let test2_passed = all_output.contains(burst);
    println!("    Sent: {burst:?}");
    println!("    Looking for exact match in output...");
    if test2_passed {
        println!("    ✅ TEST 2 PASSED");
    } else {
        println!("    ❌ TEST 2 FAILED - characters dropped!");
        println!("    Output: {all_output:?}");
    }

    // TEST 3: Very rapid repeated input
    println!("\n  TEST 3: Very rapid repeated characters");
    all_output.clear();
    std::thread::sleep(Duration::from_millis(200));

    let repeated = "aaaaaaaaaa"; // 10 a's
    stdin.write_all(repeated.as_bytes())?;
    stdin.write_all(b"\n")?;
    stdin.flush()?;

    std::thread::sleep(Duration::from_millis(500));

    loop {
        match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                all_output.push_str(&chunk);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => break,
        }
    }

    let test3_passed = all_output.contains(repeated);
    println!("    Sent: {repeated:?}");
    if test3_passed {
        println!("    ✅ TEST 3 PASSED");
    } else {
        println!("    ❌ TEST 3 FAILED - characters dropped!");
        println!("    Output: {all_output:?}");
    }

    // Cleanup
    println!("\n  Cleaning up...");
    let _ = child.kill();
    let _ = child.wait();

    // Final verdict
    println!("\n========================================");
    if test1_passed && test2_passed && test3_passed {
        println!("✅ ALL KEYBOARD INPUT TESTS PASSED");
        Ok(())
    } else {
        let mut failures = Vec::new();
        if !test1_passed {
            failures.push("TEST 1 (single chars)");
        }
        if !test2_passed {
            failures.push("TEST 2 (burst)");
        }
        if !test3_passed {
            failures.push("TEST 3 (repeated)");
        }
        println!("❌ KEYBOARD INPUT TESTS FAILED: {}", failures.join(", "));
        anyhow::bail!("Keyboard input test failed - characters are being dropped")
    }
}
