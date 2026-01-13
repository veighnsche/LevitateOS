//! Unit tests - runs cargo test on crates with std feature
//!
//! `TEAM_030`: Tests individual functions in isolation
//! `TEAM_425`: Run tests from kernel submodule directory (separate workspace)

use anyhow::{bail, Context, Result};
use std::process::Command;

/// Run a test for a kernel crate from the kernel submodule directory
fn run_kernel_test(crate_name: &str, features: Option<&str>) -> Result<()> {
    println!("Running {crate_name} unit tests...");

    let mut args = vec![
        "test",
        "-p",
        crate_name,
        "--target",
        "x86_64-unknown-linux-gnu",
    ];

    if let Some(feat) = features {
        args.push("--features");
        args.push(feat);
    }

    let status = Command::new("cargo")
        .current_dir("crates/kernel") // Run from kernel submodule
        .args(&args)
        .status()
        .with_context(|| format!("Failed to run {crate_name} tests"))?;

    if !status.success() {
        bail!("{crate_name} unit tests failed");
    }
    Ok(())
}

pub fn run() -> Result<()> {
    println!("=== Unit Tests ===\n");

    // Core library tests
    run_kernel_test("los_hal", Some("std"))?;
    run_kernel_test("los_utils", Some("std"))?;
    run_kernel_test("los_error", None)?;

    // Trait crate tests
    run_kernel_test("input-device", Some("std"))?;
    run_kernel_test("network-device", Some("std"))?;
    run_kernel_test("storage-device", Some("std"))?;

    // Driver tests
    run_kernel_test("virtio-transport", Some("std"))?;
    run_kernel_test("virtio-blk", Some("std"))?;
    run_kernel_test("virtio-input", Some("std"))?;
    run_kernel_test("virtio-net", Some("std"))?;
    run_kernel_test("los_pci", Some("std"))?;

    println!("\n✅ All unit tests passed\n");
    Ok(())
}
