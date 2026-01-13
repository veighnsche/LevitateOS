//! Behavior test - verifies Linux boot output matches golden log
//!
//! TEAM_030: Migrated from scripts/test_behavior.sh
//! TEAM_476: Rewritten to test Linux + OpenRC boot (custom kernel removed)
//!
//! Compares boot output against golden file to detect regressions.

use anyhow::{bail, Context, Result};
use std::fs;
use std::process::Command;

use crate::config::{GoldenRating, XtaskConfig};
use crate::qemu::{Arch, QemuBuilder, QemuProfile};

// TEAM_476: Use Linux + OpenRC golden file
fn golden_file(_arch: &str) -> &'static str {
    "tests/golden_boot_linux_openrc.txt"
}

const ACTUAL_FILE: &str = "tests/actual_boot.txt";
const TIMEOUT_SECS: u64 = 30;

pub fn run(arch: &str, update: bool) -> Result<()> {
    let profile = if arch == "x86_64" {
        QemuProfile::X86_64
    } else {
        QemuProfile::Default
    };
    run_with_profile(profile, arch, update)
}

/// Run behavior test with GICv3 profile (TEAM_055)
#[allow(dead_code)]
pub fn run_gicv3() -> Result<()> {
    run_with_profile(QemuProfile::GicV3, "aarch64", false)
}

fn run_with_profile(profile: QemuProfile, arch: &str, update: bool) -> Result<()> {
    let profile_name = match profile {
        QemuProfile::Default => "Default",
        QemuProfile::Pixel6 => "Pixel 6 (8GB, 8 cores)",
        QemuProfile::GicV3 => "GICv3 Test",
        QemuProfile::X86_64 => "x86_64",
    };
    println!("=== Behavior Test [Linux + OpenRC, {profile_name} on {arch}] ===\n");

    // Load config to check golden file rating
    let config = XtaskConfig::load()?;
    let golden_path = golden_file(arch);
    let rating = config.golden_rating(golden_path);

    // TEAM_476: Build Linux + OpenRC initramfs
    crate::builder::create_initramfs(arch)?;

    // Kill any existing QEMU
    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => bail!("Unsupported architecture: {arch}"),
    };
    let _ = Command::new("pkill").args(["-f", qemu_bin]).status();

    // Clean up previous run
    let _ = fs::remove_file(ACTUAL_FILE);

    println!("Running QEMU (headless, {TIMEOUT_SECS}s timeout)...");

    // Build QEMU command using QemuBuilder
    let arch_enum = Arch::try_from(arch)?;
    let initrd_path = format!("target/initramfs/{}-openrc.cpio", arch);

    let builder = QemuBuilder::new(arch_enum, profile)
        .display_headless()
        .serial_file(ACTUAL_FILE)
        
        .initrd(&initrd_path);

    let base_cmd = builder.build()?;
    let args: Vec<_> = base_cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

    // Run with timeout
    let mut timeout_args = vec![format!("{}s", TIMEOUT_SECS)];
    timeout_args.push(arch_enum.qemu_binary().to_string());
    timeout_args.extend(args);

    let status = Command::new("timeout")
        .args(&timeout_args)
        .status()
        .context("Failed to run QEMU")?;

    // timeout returns 124 on timeout, which is expected
    if !status.success() && status.code() != Some(124) {
        bail!("QEMU failed unexpectedly");
    }

    // Read files
    let golden = fs::read_to_string(golden_path)
        .context(format!("Failed to read golden boot log: {golden_path}"))?;
    let actual = fs::read_to_string(ACTUAL_FILE).context("Failed to read actual boot output")?;

    // Normalize line endings
    let golden = golden.replace("\r\n", "\n");
    let actual = actual.replace("\r\n", "\n");

    // Compare - behavior depends on golden file rating (gold vs silver)
    let matches = golden.trim() == actual.trim();

    match rating {
        GoldenRating::Silver => {
            if matches {
                println!("✅ SILVER MODE: No changes detected.\n");
            } else {
                println!("🔄 SILVER MODE: Auto-updating golden file...\n");
                fs::write(golden_path, &actual).context("Failed to update golden boot log")?;
                println!("--- Changes Detected (Golden file updated) ---");
                print_colored_diff(&golden, &actual);
                println!();
            }
        }
        GoldenRating::Gold => {
            if matches {
                println!("✅ SUCCESS: Current behavior matches Golden Log.\n");
            } else if update {
                println!("🔄 UPDATING Golden Log...\n");
                fs::write(golden_path, &actual).context("Failed to update golden boot log")?;
                println!("✅ Golden Log updated successfully. Re-run tests to verify.");
                return Ok(());
            } else {
                println!("❌ FAILURE: Behavior REGRESSION detected!\n");
                println!("--- Diff ---");
                print_colored_diff(&golden, &actual);
                println!(
                    "\n💡 TIP: If this change is intentional, run with --update to refresh the golden log."
                );
                return Err(anyhow::anyhow!("Behavior test failed"));
            }
        }
    }

    // TEAM_476: Verify Linux boot markers
    if actual.contains("Linux version") {
        println!("✅ VERIFIED: Linux kernel booted.");
    } else {
        bail!("❌ FAILURE: Linux kernel did not boot!");
    }

    if actual.contains("OpenRC") {
        println!("✅ VERIFIED: OpenRC init started.");
    }

    if actual.contains("~ #") || actual.contains("/ #") {
        println!("✅ VERIFIED: Shell prompt reached.");
    }

    Ok(())
}

/// Print a colored diff showing additions and removals
fn print_colored_diff(expected: &str, actual: &str) {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    let max_len = expected_lines.len().max(actual_lines.len());
    let mut added = 0;
    let mut removed = 0;
    let mut changed = 0;

    for i in 0..max_len {
        match (expected_lines.get(i), actual_lines.get(i)) {
            (Some(e), Some(a)) if e == a => {
                // Lines match - no output
            }
            (Some(e), Some(a)) => {
                // Lines differ
                println!("- {e}");
                println!("+ {a}");
                changed += 1;
            }
            (Some(e), None) => {
                // Line removed
                println!("- {e}");
                removed += 1;
            }
            (None, Some(a)) => {
                // Line added
                println!("+ {a}");
                added += 1;
            }
            (None, None) => unreachable!(),
        }
    }

    // Summary
    if added > 0 || removed > 0 || changed > 0 {
        println!("\nSummary: {added} added, {removed} removed, {changed} changed");
    }
}
