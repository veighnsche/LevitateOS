//! LevitateOS xtask - Development task runner
//!
//! Usage:
//!   cargo xtask test          # Run all tests (behavior + regression)
//!   cargo xtask test behavior # Run behavior test only
//!   cargo xtask test regress  # Run regression tests only
//!   cargo xtask build         # Build kernel (release)
//!   cargo xtask run           # Build and run in QEMU

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::{Command, Stdio};

mod tests;

#[derive(Parser)]
#[command(name = "xtask", about = "LevitateOS development task runner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run tests
    Test {
        /// Which test suite to run (unit, behavior, regress, or all)
        #[arg(default_value = "all")]
        suite: String,
    },
    /// Build the kernel
    Build,
    /// Build and run in QEMU
    Run,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure we're in project root
    let project_root = project_root()?;
    std::env::set_current_dir(&project_root)?;

    match cli.command {
        Commands::Test { suite } => match suite.as_str() {
            "all" => {
                tests::unit::run()?;
                tests::behavior::run()?;
                tests::regression::run()?;
            }
            "unit" => tests::unit::run()?,
            "behavior" => tests::behavior::run()?,
            "regress" | "regression" => tests::regression::run()?,
            other => bail!("Unknown test suite: {}. Use 'unit', 'behavior', 'regress', or 'all'", other),
        },
        Commands::Build => {
            build_kernel()?;
        }
        Commands::Run => {
            build_kernel()?;
            run_qemu(false)?;
        }
    }

    Ok(())
}

fn project_root() -> Result<PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap());

    // If we're in xtask/, go up one level
    if manifest_dir.ends_with("xtask") {
        Ok(manifest_dir.parent().unwrap().to_path_buf())
    } else {
        Ok(manifest_dir)
    }
}

pub fn build_kernel() -> Result<()> {
    build_kernel_with_features(&[])
}

/// Build kernel with verbose feature for behavior testing (Rule 4: Silence is Golden)
pub fn build_kernel_verbose() -> Result<()> {
    build_kernel_with_features(&["verbose"])
}

fn build_kernel_with_features(features: &[&str]) -> Result<()> {
    println!("Building kernel...");
    let mut args = vec![
        "build".to_string(),
        "--release".to_string(),
        "--target".to_string(), "aarch64-unknown-none".to_string(),
        "-p".to_string(), "levitate-kernel".to_string(),
    ];
    
    if !features.is_empty() {
        args.push("--features".to_string());
        args.push(features.join(","));
    }
    
    let status = Command::new("cargo")
        .args(&args)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        bail!("Kernel build failed");
    }
    Ok(())
}

pub fn run_qemu(headless: bool) -> Result<std::process::Output> {
    let kernel_elf = "target/aarch64-unknown-none/release/levitate-kernel";

    let mut args = vec![
        "-M", "virt",
        "-cpu", "cortex-a53",
        "-m", "512M",
        "-kernel", kernel_elf,
        "-device", "virtio-gpu-device",
        "-device", "virtio-keyboard-device",
        "-device", "virtio-tablet-device",
        "-device", "virtio-net-device,netdev=net0",
        "-netdev", "user,id=net0",
        "-drive", "file=tinyos_disk.img,format=raw,if=none,id=hd0",
        "-device", "virtio-blk-device,drive=hd0",
        "-no-reboot",
    ];

    if headless {
        args.extend(["-display", "none", "-serial", "stdio"]);
    } else {
        args.extend(["-serial", "stdio"]);
    }

    Command::new("qemu-system-aarch64")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run QEMU")
}
