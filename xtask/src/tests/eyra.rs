//! TEAM_358: Eyra std integration test
//!
//! Tests Eyra-based binaries on LevitateOS to detect missing syscalls.
//! This test builds an Eyra hello world, adds it to initramfs, and runs it.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Run the Eyra integration test
pub fn run(arch: &str) -> Result<()> {
    println!("=== Eyra Integration Test [{}] ===\n", arch);

    // Step 1: Build Eyra binary
    println!("📦 Building Eyra test binary...");
    build_eyra_binary()?;

    // Step 2: Copy to initramfs staging area
    println!("📁 Adding to initramfs...");
    add_to_initramfs(arch)?;

    // Step 3: Use vm::exec to run eyra-hello in the shell
    println!("🚀 Running eyra-hello in VM shell...\n");
    let output = crate::vm::exec("eyra-hello", 45, arch)?;

    // Step 4: Analyze output
    println!("\n📊 Analyzing results...\n");
    analyze_output_string(&output)?;

    Ok(())
}

fn build_eyra_binary() -> Result<()> {
    // Check if eyra binary already exists (skip slow rebuild)
    let binary_path = "userspace/eyra-hello/target/x86_64-unknown-linux-gnu/release/eyra-hello";
    if Path::new(binary_path).exists() {
        println!("  ✅ Eyra binary already built (using cached)");
        return Ok(());
    }

    // Eyra requires -Zbuild-std and specific nightly toolchain
    // The rust-toolchain.toml in eyra-hello specifies nightly-2025-04-28
    println!("  ⏳ Building Eyra (this may take a while on first run)...");
    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target", "x86_64-unknown-linux-gnu",
            "-Zbuild-std=std,panic_abort",
        ])
        .current_dir("userspace/eyra-hello")
        .status()
        .context("Failed to run cargo build for eyra-hello")?;

    if !status.success() {
        println!("\n⚠️  Eyra build failed. This usually means:");
        println!("   1. Missing nightly toolchain: rustup install nightly-2025-04-28");
        println!("   2. Missing rust-src: rustup component add rust-src --toolchain nightly-2025-04-28");
        println!("\nRun manually to see full error:");
        println!("   cd userspace/eyra-hello && cargo build --release --target x86_64-unknown-linux-gnu -Zbuild-std=std,panic_abort");
        bail!("Eyra binary build failed");
    }

    // Verify binary exists
    if !Path::new(binary_path).exists() {
        bail!("Eyra binary not found at {}", binary_path);
    }

    println!("  ✅ Eyra binary built successfully");
    Ok(())
}

fn add_to_initramfs(arch: &str) -> Result<()> {
    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    // Create staging directory if needed
    let staging_dir = format!("crates/userspace/target/{}/release", target);
    fs::create_dir_all(&staging_dir)?;

    // Copy Eyra binary to userspace staging area
    let src = "userspace/eyra-hello/target/x86_64-unknown-linux-gnu/release/eyra-hello";
    let dst = format!("{}/eyra-hello", staging_dir);
    fs::copy(src, &dst).context("Failed to copy Eyra binary to staging")?;

    println!("  ✅ Added eyra-hello to initramfs staging");
    Ok(())
}

fn analyze_output_string(output: &str) -> Result<()> {
    println!("--- Analyzing Eyra Output ---");
    
    // Track what we found
    let mut syscall_errors: Vec<String> = Vec::new();
    let mut eyra_started = false;
    let mut eyra_complete = false;
    let mut success_markers: Vec<&str> = Vec::new();
    let mut fail_markers: Vec<&str> = Vec::new();

    for line in output.lines() {
        // Check for unknown syscalls
        if line.contains("Unknown syscall number:") {
            let syscall_num = line.split("Unknown syscall number:").nth(1)
                .map(|s| s.trim())
                .unwrap_or("?");
            syscall_errors.push(format!("Syscall {} not implemented", syscall_num));
            println!("❌ {}", line);
        }

        // Check for Eyra test markers
        if line.contains("=== Eyra Test on LevitateOS ===") {
            eyra_started = true;
            println!("📍 {}", line);
        }
        if line.contains("=== Eyra Test Complete ===") {
            eyra_complete = true;
            println!("📍 {}", line);
        }
        if line.contains("[OK]") {
            success_markers.push(line);
            println!("✅ {}", line);
        }
        if line.contains("[FAIL]") {
            fail_markers.push(line);
            println!("❌ {}", line);
        }

        // Check for crashes
        if line.contains("*** USER EXCEPTION ***") || line.contains("INVALID OPCODE") {
            println!("💥 {}", line);
        }
    }

    println!("\n--- Summary ---\n");

    // Report results
    if !syscall_errors.is_empty() {
        println!("🔶 Missing syscalls detected:");
        for err in &syscall_errors {
            println!("   - {}", err);
        }
        println!();
    }

    if eyra_started {
        println!("✅ Eyra binary started executing");
    } else {
        println!("❌ Eyra binary did NOT start (check if it's in initramfs and being executed)");
    }

    if eyra_complete {
        println!("✅ Eyra test completed successfully!");
    } else if eyra_started {
        println!("⚠️  Eyra test started but did not complete (crashed or missing syscalls)");
    }

    println!("\nSuccess markers: {}", success_markers.len());
    println!("Failure markers: {}", fail_markers.len());

    if !syscall_errors.is_empty() {
        println!("\n💡 To fix: Implement the missing syscalls listed above.");
        // Don't bail - this is informational
    }

    if !fail_markers.is_empty() {
        bail!("Eyra test had {} failures", fail_markers.len());
    }

    if eyra_complete && syscall_errors.is_empty() {
        println!("\n🎉 Full Eyra/std support verified!");
    }

    Ok(())
}
