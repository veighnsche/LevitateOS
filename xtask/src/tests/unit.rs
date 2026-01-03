//! Unit tests - runs cargo test on crates with std feature
//!
//! TEAM_030: Tests individual functions in isolation

use anyhow::{bail, Context, Result};
use std::process::Command;

pub fn run() -> Result<()> {
    println!("=== Unit Tests ===\n");

    // Run unit tests for levitate-hal (has most tests)
    println!("Running levitate-hal unit tests...");
    let hal_status = Command::new("cargo")
        .args([
            "test",
            "-p", "levitate-hal",
            "--features", "std",
            "--target", "x86_64-unknown-linux-gnu",
        ])
        .status()
        .context("Failed to run levitate-hal tests")?;

    if !hal_status.success() {
        bail!("levitate-hal unit tests failed");
    }

    // Run unit tests for levitate-utils
    println!("\nRunning levitate-utils unit tests...");
    let utils_status = Command::new("cargo")
        .args([
            "test",
            "-p", "levitate-utils",
            "--features", "std",
            "--target", "x86_64-unknown-linux-gnu",
        ])
        .status()
        .context("Failed to run levitate-utils tests")?;

    if !utils_status.success() {
        bail!("levitate-utils unit tests failed");
    }

    println!("\n✅ All unit tests passed\n");
    Ok(())
}
