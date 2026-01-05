//! Behavior test - verifies kernel boot output matches golden log
//!
//! TEAM_030: Migrated from scripts/test_behavior.sh
//! TEAM_042: Added Pixel 6 profile support
//!
//! Note: Uses --features verbose to enable boot messages.
//! Production builds are silent (Rule 4: Silence is Golden).

use anyhow::{bail, Context, Result};
use std::fs;
use std::process::Command;

use crate::QemuProfile;

const GOLDEN_FILE: &str = "tests/golden_boot.txt";
const ACTUAL_FILE: &str = "tests/actual_boot.txt";
const KERNEL_BIN: &str = "kernel64_rust.bin";
const TIMEOUT_SECS: u64 = 5;

pub fn run() -> Result<()> {
    run_with_profile(QemuProfile::Default)
}

/// Run behavior test with Pixel 6 profile (8GB, 8 cores)
#[allow(dead_code)]
pub fn run_pixel6() -> Result<()> {
    run_with_profile(QemuProfile::Pixel6)
}

/// Run behavior test with GICv3 profile (TEAM_055)
pub fn run_gicv3() -> Result<()> {
    run_with_profile(QemuProfile::GicV3)
}

fn run_with_profile(profile: QemuProfile) -> Result<()> {
    let profile_name = match profile {
        QemuProfile::Default => "Default (512MB)",
        QemuProfile::Pixel6 => "Pixel 6 (8GB, 8 cores)",
        QemuProfile::GicV3 => "GICv3 Test (512MB)",
    };
    println!("=== Behavior Test [{}] ===\n", profile_name);

    // Build kernel with verbose feature for golden file comparison
    crate::build_kernel_verbose()?;

    // Kill any existing QEMU
    let _ = Command::new("pkill")
        .args(["-f", "qemu-system-aarch64"])
        .status();

    // Clean up previous run
    let _ = fs::remove_file(ACTUAL_FILE);

    println!("Running QEMU (headless, {}s timeout)...", TIMEOUT_SECS);

    // Build QEMU args using profile
    let mut args = vec![
        format!("{}s", TIMEOUT_SECS),
        "qemu-system-aarch64".to_string(),
        "-M".to_string(), profile.machine().to_string(),
        "-cpu".to_string(), profile.cpu().to_string(),
        "-m".to_string(), profile.memory().to_string(),
        "-kernel".to_string(), KERNEL_BIN.to_string(),
        "-display".to_string(), "none".to_string(),
        "-serial".to_string(), format!("file:{}", ACTUAL_FILE),
        "-device".to_string(), "virtio-gpu-device".to_string(),
        "-device".to_string(), "virtio-keyboard-device".to_string(),
        "-device".to_string(), "virtio-tablet-device".to_string(),
        "-device".to_string(), "virtio-net-device,netdev=net0".to_string(),
        "-netdev".to_string(), "user,id=net0".to_string(),
        "-drive".to_string(), "file=tinyos_disk.img,format=raw,if=none,id=hd0".to_string(),
        "-device".to_string(), "virtio-blk-device,drive=hd0".to_string(),
        "-initrd".to_string(), "initramfs.cpio".to_string(),
        "-no-reboot".to_string(),
    ];

    if let Some(smp) = profile.smp() {
        args.push("-smp".to_string());
        args.push(smp.to_string());
    }

    // Run QEMU with timeout, capturing serial output to file
    let status = Command::new("timeout")
        .args(&args)
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

    // TEAM_065: Normalize GIC version differences (environment-dependent)
    // GIC version (V2 vs V3) depends on QEMU version/config, not code
    let golden = golden.replace("Detected GIC version: V2", "Detected GIC version: V*");
    let golden = golden.replace("Detected GIC version: V3", "Detected GIC version: V*");
    let actual = actual.replace("Detected GIC version: V2", "Detected GIC version: V*");
    let actual = actual.replace("Detected GIC version: V3", "Detected GIC version: V*");

    // TEAM_111: Filter out [TICK] lines as they are timing-dependent and flaky
    let golden = golden.lines()
        .filter(|l| !l.starts_with("[TICK]"))
        .collect::<Vec<_>>()
        .join("\n");
        
    let actual = actual.lines()
        .filter(|l| !l.starts_with("[TICK]"))
        .collect::<Vec<_>>()
        .join("\n");

    // Compare
    if golden.trim() == actual.trim() {
        println!("✅ SUCCESS: Current behavior matches Golden Log.\n");
        
        // TEAM_111: Additional verification for DESIRED BEHAVIORS
        // We MUST verify that we actually reached the shell, ensuring meaningful boot.
        if !actual.contains("LevitateOS Shell") || !actual.contains("# ") {
            bail!("❌ FAILURE: Boot did not reach shell prompt! (Golden log might be incomplete?)");
        } else {
            println!("✅ VERIFIED: Shell prompt reached.");
        }

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
