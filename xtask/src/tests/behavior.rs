//! Behavior test - verifies kernel boot output matches golden log
//!
//! TEAM_030: Migrated from scripts/test_behavior.sh
//!
//! Note: Uses --features verbose to enable boot messages.
//! Production builds are silent (Rule 4: Silence is Golden).

use anyhow::{bail, Context, Result};
use std::fs;
use std::process::Command;

const GOLDEN_FILE: &str = "tests/golden_boot.txt";
const ACTUAL_FILE: &str = "tests/actual_boot.txt";
const KERNEL_ELF: &str = "target/aarch64-unknown-none/release/levitate-kernel";
const TIMEOUT_SECS: u64 = 5;

pub fn run() -> Result<()> {
    println!("=== Behavior Test ===\n");

    // Build kernel with verbose feature for golden file comparison
    crate::build_kernel_verbose()?;

    // Kill any existing QEMU
    let _ = Command::new("pkill")
        .args(["-f", "qemu-system-aarch64"])
        .status();

    // Clean up previous run
    let _ = fs::remove_file(ACTUAL_FILE);

    println!("Running QEMU (headless, {}s timeout)...", TIMEOUT_SECS);

    // Run QEMU with timeout, capturing serial output to file
    let status = Command::new("timeout")
        .args([
            &format!("{}s", TIMEOUT_SECS),
            "qemu-system-aarch64",
            "-M", "virt",
            "-cpu", "cortex-a53",
            "-m", "512M",
            "-kernel", KERNEL_ELF,
            "-display", "none",
            "-serial", &format!("file:{}", ACTUAL_FILE),
            "-device", "virtio-gpu-device",
            "-device", "virtio-keyboard-device",
            "-device", "virtio-tablet-device",
            "-device", "virtio-net-device,netdev=net0",
            "-netdev", "user,id=net0",
            "-drive", "file=tinyos_disk.img,format=raw,if=none,id=hd0",
            "-device", "virtio-blk-device,drive=hd0",
            "-no-reboot",
        ])
        .status()
        .context("Failed to run QEMU")?;

    // timeout returns 124 on timeout, which is expected
    if !status.success() && status.code() != Some(124) {
        bail!("QEMU failed unexpectedly");
    }

    // Read files
    let golden = fs::read_to_string(GOLDEN_FILE)
        .context("Failed to read golden boot log")?;
    let actual = fs::read_to_string(ACTUAL_FILE)
        .context("Failed to read actual boot output")?;

    // Normalize line endings
    let golden = golden.replace("\r\n", "\n");
    let actual = actual.replace("\r\n", "\n");

    // Compare
    if golden.trim() == actual.trim() {
        println!("✅ SUCCESS: Current behavior matches Golden Log.\n");
        Ok(())
    } else {
        println!("❌ FAILURE: Behavior REGRESSION detected!\n");
        println!("--- Diff ---");
        print_diff(&golden, &actual);
        bail!("Behavior test failed");
    }
}

fn print_diff(expected: &str, actual: &str) {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    for (i, (e, a)) in expected_lines.iter().zip(actual_lines.iter()).enumerate() {
        if e != a {
            println!("Line {}: expected '{}', got '{}'", i + 1, e, a);
        }
    }

    if expected_lines.len() != actual_lines.len() {
        println!(
            "Line count differs: expected {}, got {}",
            expected_lines.len(),
            actual_lines.len()
        );
    }
}
