//! Behavior test - verifies kernel boot output matches golden log
//!
//! TEAM_030: Migrated from scripts/test_behavior.sh
//! TEAM_042: Added Pixel 6 profile support
//! TEAM_142: Added graceful shutdown testing with exit --verbose
//!
//! Note: Uses --features verbose to enable boot messages.
//! Production builds are silent (Rule 4: Silence is Golden).

use anyhow::{bail, Context, Result};
use std::fs;
use std::process::Command;

use crate::config::{GoldenRating, XtaskConfig};
use crate::qemu::QemuProfile;

// TEAM_287: Arch-specific golden files
fn golden_file(arch: &str) -> &'static str {
    match arch {
        "x86_64" => "tests/golden_boot_x86_64.txt",
        _ => "tests/golden_boot.txt",
    }
}
const ACTUAL_FILE: &str = "tests/actual_boot.txt";
const TIMEOUT_SECS: u64 = 15;

pub fn run(arch: &str, update: bool) -> Result<()> {
    let profile = if arch == "x86_64" {
        QemuProfile::X86_64
    } else {
        QemuProfile::Default
    };
    run_with_profile(profile, arch, update)
}

/// Run behavior test with Pixel 6 profile (8GB, 8 cores)
#[allow(dead_code)]
pub fn run_pixel6(arch: &str) -> Result<()> {
    if arch != "aarch64" {
        bail!("Pixel 6 profile only supported on aarch64");
    }
    run_with_profile(QemuProfile::Pixel6, arch, false)
}

/// Run behavior test with GICv3 profile (TEAM_055)
pub fn run_gicv3() -> Result<()> {
    run_with_profile(QemuProfile::GicV3, "aarch64", false)
}

fn run_with_profile(profile: QemuProfile, arch: &str, update: bool) -> Result<()> {
    let profile_name = match profile {
        QemuProfile::Default => "Default",
        QemuProfile::Pixel6 => "Pixel 6 (8GB, 8 cores)",
        QemuProfile::GicV3 => "GICv3 Test",
        QemuProfile::X86_64 => "x86_64 (ISO)",
    };
    println!("=== Behavior Test [{} on {}] ===\n", profile_name, arch);

    // Load config to check golden file rating
    let config = XtaskConfig::load()?;
    let golden_path = golden_file(arch);
    let rating = config.golden_rating(golden_path);

    // TEAM_286: x86_64 requires Limine ISO boot (SeaBIOS doesn't support multiboot)
    let use_iso = arch == "x86_64";

    if use_iso {
        // Build ISO with verbose features
        crate::build::build_iso_verbose(arch)?;
    } else {
        // Build kernel with verbose feature for golden file comparison
        crate::build::build_kernel_verbose(arch)?;
    }

    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    // Kill any existing QEMU
    let _ = Command::new("pkill").args(["-f", qemu_bin]).status();

    // Clean up previous run
    let _ = fs::remove_file(ACTUAL_FILE);

    println!("Running QEMU (headless, {}s timeout)...", TIMEOUT_SECS);

    let kernel_bin = if arch == "aarch64" {
        "kernel64_rust.bin"
    } else {
        // x86_64 uses ISO boot, kernel path not used directly
        "levitate.iso"
    };

    // Build QEMU args using profile
    let mut args = vec![
        format!("{}s", TIMEOUT_SECS),
        qemu_bin.to_string(),
        "-M".to_string(),
        profile.machine().to_string(),
        "-cpu".to_string(),
        profile.cpu().to_string(),
        "-m".to_string(),
        profile.memory().to_string(),
    ];

    // TEAM_286: x86_64 uses ISO boot, aarch64 uses -kernel
    if use_iso {
        args.extend([
            "-cdrom".to_string(),
            kernel_bin.to_string(),
            "-boot".to_string(),
            "d".to_string(),
        ]);
    } else {
        // TEAM_327: Use arch-specific initramfs
        args.extend([
            "-kernel".to_string(),
            kernel_bin.to_string(),
            "-initrd".to_string(),
            format!("initramfs_{}.cpio", arch),
        ]);
    }

    args.extend([
        "-display".to_string(),
        "none".to_string(),
        "-serial".to_string(),
        format!("file:{}", ACTUAL_FILE),
        "-device".to_string(),
        "virtio-gpu-pci".to_string(), // TEAM_114: PCI transport
        "-device".to_string(),
        format!(
            "virtio-keyboard-{}",
            if arch == "x86_64" { "pci" } else { "device" }
        ),
        "-device".to_string(),
        format!(
            "virtio-tablet-{}",
            if arch == "x86_64" { "pci" } else { "device" }
        ),
        "-device".to_string(),
        format!(
            "virtio-net-{},netdev=net0",
            if arch == "x86_64" { "pci" } else { "device" }
        ),
        "-netdev".to_string(),
        "user,id=net0".to_string(),
        "-drive".to_string(),
        "file=tinyos_disk.img,format=raw,if=none,id=hd0".to_string(),
        "-device".to_string(),
        format!(
            "virtio-blk-{},drive=hd0",
            if arch == "x86_64" { "pci" } else { "device" }
        ),
        "-no-reboot".to_string(),
    ]);

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
    // TEAM_287: Use arch-specific golden file
    let golden_path = golden_file(arch);
    let golden = fs::read_to_string(golden_path)
        .context(format!("Failed to read golden boot log: {}", golden_path))?;
    let actual = fs::read_to_string(ACTUAL_FILE).context("Failed to read actual boot output")?;

    // Normalize line endings
    let golden = golden.replace("\r\n", "\n");
    let actual = actual.replace("\r\n", "\n");

    // TEAM_129: Save raw actual for GPU regression assertions (before filtering)
    let actual_raw = actual.clone();

    // TEAM_065: Normalize GIC version differences (environment-dependent)
    // GIC version (V2 vs V3) depends on QEMU version/config, not code
    let golden = golden.replace("Detected GIC version: V2", "Detected GIC version: V*");
    let golden = golden.replace("Detected GIC version: V3", "Detected GIC version: V*");
    let actual = actual.replace("Detected GIC version: V2", "Detected GIC version: V*");
    let actual = actual.replace("Detected GIC version: V3", "Detected GIC version: V*");

    // TEAM_111: Filter out [TICK] lines as they are timing-dependent and flaky
    // TEAM_129: Filter out [GPU_TEST] lines - they are verified separately with assertions
    let golden = golden
        .lines()
        .filter(|l| !l.starts_with("[TICK]") && !l.starts_with("[GPU_TEST]"))
        .collect::<Vec<_>>()
        .join("\n");

    let actual = actual
        .lines()
        .filter(|l| !l.starts_with("[TICK]") && !l.starts_with("[GPU_TEST]"))
        .collect::<Vec<_>>()
        .join("\n");

    // TEAM_115/129: Normalization logic removed in favor of kernel-side masking (Rule 18)
    // The kernel strict masking ensures "way better" golden file stability.

    // TEAM_143: Check for USER EXCEPTION FIRST - crashes are always a bug
    // This runs before golden file comparison so we catch crashes even if output differs
    if actual_raw.contains("*** USER EXCEPTION ***") {
        // Extract exception details for debugging
        let exception_lines: Vec<&str> = actual_raw
            .lines()
            .skip_while(|l| !l.contains("*** USER EXCEPTION ***"))
            .take(6)
            .collect();
        println!("❌ FAILURE: User process crashed with exception!\n");
        println!("--- Exception Details ---");
        for line in &exception_lines {
            println!("{}", line);
        }
        println!();
        bail!("Userspace crashed - this is a bug that needs to be fixed! See TODO.md for debugging steps.");
    }

    // Compare - behavior depends on golden file rating (gold vs silver)
    let matches = golden.trim() == actual.trim();

    let should_verify = match rating {
        GoldenRating::Silver => {
            // Silver files: Always auto-update and show diff
            if !matches {
                println!("🔄 SILVER MODE: Auto-updating golden file...\n");
                fs::write(golden_path, &actual_raw).context("Failed to update golden boot log")?;
                println!("--- Changes Detected (Golden file updated) ---");
                print_colored_diff(&golden, &actual);
                println!();
            } else {
                println!("✅ SILVER MODE: No changes detected.\n");
            }
            true // Continue to verification
        }
        GoldenRating::Gold => {
            if matches {
                println!("✅ SUCCESS: Current behavior matches Golden Log.\n");
                true // Continue to verification
            } else if update {
                println!("🔄 UPDATING Golden Log (Rule 4 Refined)...\n");
                fs::write(golden_path, &actual_raw).context("Failed to update golden boot log")?;
                println!("✅ Golden Log updated successfully. Re-run tests to verify.");
                return Ok(()); // Skip verification after manual update
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
    };

    if should_verify {
        // TEAM_111: Additional verification for DESIRED BEHAVIORS
        // TEAM_129: Verify shell was spawned AND actually ran (catches scheduling bugs)
        // TEAM_287: Skip userspace checks for x86_64 (currently serial-only, no initramfs in ISO)

        if arch != "x86_64" {
            // Check 1: Shell was spawned by init
            if !actual_raw.contains("[INIT] Shell spawned as PID 2") {
                bail!("❌ FAILURE: Shell was not spawned!");
            }
            println!("✅ VERIFIED: Shell spawned successfully.");

            // Check 2: Shell task was scheduled (catches yield/scheduling bugs)
            if !actual_raw.contains("[TASK] Entering user task PID=2") {
                bail!("❌ FAILURE: Shell was spawned but never scheduled! (scheduling bug)");
            }
            println!("✅ VERIFIED: Shell was scheduled.");

            // Check 3: Shell's _start() executed and printed banner
            if !actual_raw.contains("LevitateOS Shell") {
                bail!(
                    "❌ FAILURE: Shell started but didn't print banner! (userspace execution bug)"
                );
            }
            println!("✅ VERIFIED: Shell executed successfully.");

            // TEAM_129: GPU regression test verification (use raw output before filtering)
            // Check that GPU flush was called (prevents black screen regression)
            if actual_raw.contains("[GPU_TEST] WARNING: GPU flush count is 0") {
                bail!("❌ FAILURE: GPU flush count is 0 - display would be black!");
            }

            // TEAM_129: Check flush count is high enough (indicates flushes during shell execution)
            // If flush only happens during boot (not after scheduler), count would be ~1
            // With proper per-write flushing, count should be much higher (50+)
            if let Some(count_str) = actual_raw.split("[GPU_TEST] Flush count: ").nth(1) {
                if let Some(count) = count_str
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<u32>().ok())
                {
                    if count < 10 {
                        bail!("❌ FAILURE: GPU flush count is {} (too low) - shell output may not be visible!", count);
                    }
                    println!("✅ VERIFIED: GPU flush count is {} (flushes happening during shell execution).", count);
                }
            } else if actual_raw.contains("[GPU_TEST] Flush count:") {
                println!("✅ VERIFIED: GPU flush is being called.");
            }

            // Check that framebuffer has content (terminal rendered something)
            if actual_raw.contains("[GPU_TEST] WARNING: Framebuffer is entirely black") {
                bail!("❌ FAILURE: Framebuffer is entirely black - no content rendered!");
            }
            // Note: Check for warning absence, not "0 non-black" substring (would match "400 non-black")
            if actual_raw.contains("[GPU_TEST] Framebuffer:")
                && !actual_raw.contains("WARNING: Framebuffer is entirely black")
            {
                println!("✅ VERIFIED: Framebuffer has rendered content.");
            }
        } else {
            // x86_64: Just verify basic boot progress (serial-only mode)
            if actual_raw.contains("[BOOT] Stage 4:") {
                println!("✅ VERIFIED: x86_64 boot reached Stage 4.");
            }
        }

        // TEAM_143: USER EXCEPTION check is now done BEFORE golden file comparison
        // so it runs unconditionally and reports crashes even if output differs
        println!("✅ VERIFIED: No userspace crashes detected.");
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
                println!("- {}", e);
                println!("+ {}", a);
                changed += 1;
            }
            (Some(e), None) => {
                // Line removed
                println!("- {}", e);
                removed += 1;
            }
            (None, Some(a)) => {
                // Line added
                println!("+ {}", a);
                added += 1;
            }
            (None, None) => unreachable!(),
        }
    }

    // Summary
    if added > 0 || removed > 0 || changed > 0 {
        println!(
            "\nSummary: {} added, {} removed, {} changed",
            added, removed, changed
        );
    }
}
