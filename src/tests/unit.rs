//! Unit tests - runs cargo test on the main crate
//!
//! `TEAM_030`: Tests individual functions in isolation

use anyhow::{bail, Context, Result};
use std::process::Command;

pub fn run() -> Result<()> {
    println!("=== Unit Tests ===\n");

    println!("Running levitate unit tests...");

    let status = Command::new("cargo")
        .args(["test"])
        .status()
        .context("Failed to run unit tests")?;

    if !status.success() {
        bail!("Unit tests failed");
    }

    println!("\n✅ All unit tests passed\n");
    Ok(())
}
