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
    /// Build and run in QEMU (default profile: 512MB)
    Run,
    /// Build and run in QEMU with Pixel 6 profile (8GB, 8 cores)
    RunPixel6,
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
            run_qemu(QemuProfile::Default, false)?;
        }
        Commands::RunPixel6 => {
            println!("🎯 Running with Pixel 6 profile (8GB RAM, 8 cores)");
            build_kernel()?;
            run_qemu(QemuProfile::Pixel6, false)?;
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
        "-Z".to_string(), "build-std=core,alloc".to_string(),
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

/// QEMU hardware profiles
/// TEAM_042: Added Pixel 6 profile for target hardware testing
///
/// Pixel 6 Tensor SoC has:
/// - 2x Cortex-X1 @ 2.80 GHz (big)
/// - 2x Cortex-A76 @ 2.25 GHz (medium)  
/// - 4x Cortex-A55 @ 1.80 GHz (little)
///
/// QEMU Mitigations (verified QEMU 10.1+):
/// - cortex-a76 is available (exact match for medium cores, close to X1)
/// - Cluster topology supported via -smp clusters=N
/// - 8GB RAM supported via highmem
///
/// Limitations:
/// - QEMU cannot mix CPU types (no true big.LITTLE) → use clusters
/// - GICv3 not supported by kernel yet → use GICv2 (limits to 8 CPUs)
#[derive(Clone, Copy)]
pub enum QemuProfile {
    /// Default: 512MB RAM, 1 core, cortex-a53
    Default,
    /// Pixel 6: 8GB RAM, 8 cores, cortex-a76, GICv3
    Pixel6,
    /// Test: GICv3 on default machine
    GicV3,
}

impl QemuProfile {
    pub fn machine(&self) -> String {
        match self {
            QemuProfile::Default => "virt".to_string(),
            QemuProfile::Pixel6 => "virt,gic-version=3".to_string(),
            QemuProfile::GicV3 => "virt,gic-version=3".to_string(),
        }
    }

    pub fn cpu(&self) -> &'static str {
        match self {
            QemuProfile::Default => "cortex-a53",
            QemuProfile::Pixel6 => "cortex-a76",
            QemuProfile::GicV3 => "cortex-a53",
        }
    }

    pub fn memory(&self) -> &'static str {
        match self {
            QemuProfile::Default => "512M",
            QemuProfile::Pixel6 => "8G",
            QemuProfile::GicV3 => "512M",
        }
    }

    /// Returns SMP topology string
    pub fn smp(&self) -> Option<&'static str> {
        match self {
            QemuProfile::Default => None,
            QemuProfile::Pixel6 => Some("8"),
            QemuProfile::GicV3 => None,
        }
    }
}

pub fn run_qemu(profile: QemuProfile, headless: bool) -> Result<std::process::Output> {
    let kernel_elf = "target/aarch64-unknown-none/release/levitate-kernel";

    let machine = profile.machine();
    let mut args = vec![
        "-M", machine.as_str(),
        "-cpu", profile.cpu(),
        "-m", profile.memory(),
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

    if let Some(smp) = profile.smp() {
        args.extend(["-smp", smp]);
    }

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
