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

use crate::run::QemuProfile;

const GOLDEN_FILE: &str = "tests/golden_boot.txt";
const ACTUAL_FILE: &str = "tests/actual_boot.txt";
const TIMEOUT_SECS: u64 = 15;

pub fn run(arch: &str) -> Result<()> {
    let profile = if arch == "x86_64" {
        QemuProfile::X86_64
    } else {
        QemuProfile::Default
    };
    run_with_profile(profile, arch)
}

/// Run behavior test with Pixel 6 profile (8GB, 8 cores)
#[allow(dead_code)]
pub fn run_pixel6(arch: &str) -> Result<()> {
    if arch != "aarch64" {
        bail!("Pixel 6 profile only supported on aarch64");
    }
    run_with_profile(QemuProfile::Pixel6, arch)
}

/// Run behavior test with GICv3 profile (TEAM_055)
pub fn run_gicv3() -> Result<()> {
    run_with_profile(QemuProfile::GicV3, "aarch64")
}

fn run_with_profile(profile: QemuProfile, arch: &str) -> Result<()> {
    let profile_name = match profile {
        QemuProfile::Default => "Default",
        QemuProfile::Pixel6 => "Pixel 6 (8GB, 8 cores)",
        QemuProfile::GicV3 => "GICv3 Test",
        QemuProfile::X86_64 => "x86_64",
    };
    println!("=== Behavior Test [{} on {}] ===\n", profile_name, arch);

    // Build kernel with verbose feature for golden file comparison
    crate::build::build_kernel_verbose(arch)?;

    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    // Kill any existing QEMU
    let _ = Command::new("pkill")
        .args(["-f", qemu_bin])
        .status();

    // Clean up previous run
    let _ = fs::remove_file(ACTUAL_FILE);

    println!("Running QEMU (headless, {}s timeout)...", TIMEOUT_SECS);

    let kernel_bin = if arch == "aarch64" {
        "kernel64_rust.bin"
    } else {
        "target/x86_64-unknown-none/release/levitate-kernel"
    };

    // Build QEMU args using profile
    let mut args = vec![
        format!("{}s", TIMEOUT_SECS),
        qemu_bin.to_string(),
        "-M".to_string(), profile.machine().to_string(),
        "-cpu".to_string(), profile.cpu().to_string(),
        "-m".to_string(), profile.memory().to_string(),
        "-kernel".to_string(), kernel_bin.to_string(),
        "-display".to_string(), "none".to_string(),
        "-serial".to_string(), format!("file:{}", ACTUAL_FILE),
        "-device".to_string(), "virtio-gpu-pci".to_string(), // TEAM_114: PCI transport
        "-device".to_string(), format!("virtio-keyboard-{}", if arch == "x86_64" { "pci" } else { "device" }),
        "-device".to_string(), format!("virtio-tablet-{}", if arch == "x86_64" { "pci" } else { "device" }),
        "-device".to_string(), format!("virtio-net-{},netdev=net0", if arch == "x86_64" { "pci" } else { "device" }),
        "-netdev".to_string(), "user,id=net0".to_string(),
        "-drive".to_string(), "file=tinyos_disk.img,format=raw,if=none,id=hd0".to_string(),
        "-device".to_string(), format!("virtio-blk-{},drive=hd0", if arch == "x86_64" { "pci" } else { "device" }),
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
    let golden = golden.lines()
        .filter(|l| !l.starts_with("[TICK]") && !l.starts_with("[GPU_TEST]"))
        .collect::<Vec<_>>()
        .join("\n");
        
    let actual = actual.lines()
        .filter(|l| !l.starts_with("[TICK]") && !l.starts_with("[GPU_TEST]"))
        .collect::<Vec<_>>()
        .join("\n");

    // TEAM_115/129: Normalization logic removed in favor of kernel-side masking (Rule 18)
    // The kernel strict masking ensures "way better" golden file stability.

    // TEAM_143: Check for USER EXCEPTION FIRST - crashes are always a bug
    // This runs before golden file comparison so we catch crashes even if output differs
    if actual_raw.contains("*** USER EXCEPTION ***") {
        // Extract exception details for debugging
        let exception_lines: Vec<&str> = actual_raw.lines()
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

    // Compare
    if golden.trim() == actual.trim() {
        println!("✅ SUCCESS: Current behavior matches Golden Log.\n");
        
        // TEAM_111: Additional verification for DESIRED BEHAVIORS
        // TEAM_129: Verify shell was spawned AND actually ran (catches scheduling bugs)
        
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
            bail!("❌ FAILURE: Shell started but didn't print banner! (userspace execution bug)");
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
            if let Some(count) = count_str.split_whitespace().next().and_then(|s| s.parse::<u32>().ok()) {
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
        if actual_raw.contains("[GPU_TEST] Framebuffer:") && 
           !actual_raw.contains("WARNING: Framebuffer is entirely black") {
            println!("✅ VERIFIED: Framebuffer has rendered content.");
        }

        // TEAM_143: USER EXCEPTION check is now done BEFORE golden file comparison
        // so it runs unconditionally and reports crashes even if output differs
        println!("✅ VERIFIED: No userspace crashes detected.");

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
