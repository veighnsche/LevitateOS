//! # LevitateOS - Rust-native Linux distribution builder
//!
//! Build minimal Linux systems from source with type-safe, fast Rust tooling.
//!
//! ## Usage
//!
//! ```bash
//! levitate build all           # Build Linux + BusyBox + OpenRC + initramfs
//! levitate run --term          # Boot in QEMU with serial console
//! levitate run                 # Boot with GUI
//! levitate run --gdb           # Boot with GDB server on port 1234
//! levitate clean               # Clean build artifacts
//! ```
//!
//! ## Components Built
//!
//! - **Linux kernel** (6.19-rc5 from submodule)
//! - **BusyBox** (shell + 300 utilities, static musl)
//! - **OpenRC** (init system, static musl)
//! - **initramfs** (CPIO archive)

// Clippy configuration for CLI tool
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::doc_lazy_continuation)]

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod builder;
mod calc;
mod config;
mod disk;
mod qemu;
mod run;
mod support;
mod tests;
mod vm;

// Re-exports for convenience
use support::{clean, preflight};

#[derive(Parser)]
#[command(name = "levitate", about = "Rust-native Linux distribution builder")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Target architecture
    #[arg(long, global = true, default_value = "x86_64")]
    arch: String,
}

// TEAM_326: Refactored command structure for clarity
#[derive(Subcommand)]
enum Commands {
    // === Most Common ===
    /// Run QEMU (builds first if needed)
    Run(RunArgs),

    /// Build components
    #[command(subcommand)]
    Build(builder::BuildCommands),

    /// Run tests
    Test(TestArgs),

    // === VM Interaction ===
    /// Interact with running VM (session, debug, exec)
    #[command(subcommand)]
    Vm(vm::VmCommands),

    // === Disk Management ===
    /// Manage disk images
    #[command(subcommand)]
    Disk(disk::DiskCommands),

    // === Utilities ===
    /// Run preflight checks
    Check,

    /// Clean up artifacts and QEMU locks
    Clean,

    /// Kill any running QEMU instances
    Kill,

    /// Debug calculator for memory/address/bit math
    #[command(subcommand)]
    Calc(calc::CalcCommands),
}

#[derive(clap::Args)]
pub struct RunArgs {
    /// Run with GDB server enabled (port 1234)
    #[arg(long)]
    pub gdb: bool,

    /// Wait for GDB connection on startup (requires --gdb)
    #[arg(long)]
    pub wait: bool,

    /// Run in terminal-only mode (no GUI window)
    #[arg(long)]
    pub term: bool,

    /// Run with VNC display for browser verification
    #[arg(long)]
    pub vnc: bool,

    /// Run headless (no display)
    #[arg(long)]
    pub headless: bool,

    /// Enable GPU debug tracing
    #[arg(long)]
    pub gpu_debug: bool,

    /// QEMU profile (default, pixel6)
    #[arg(long, default_value = "default")]
    pub profile: String,

    /// Run internal OS tests
    #[arg(long)]
    pub test: bool,

    /// Verify GPU display via VNC
    #[arg(long)]
    pub verify_gpu: bool,

    /// Timeout for verify-gpu (seconds)
    #[arg(long, default_value = "30")]
    pub timeout: u32,
}

#[derive(clap::Args)]
pub struct TestArgs {
    /// Which test suite to run (unit, behavior, serial, keyboard, shutdown, debug, screenshot, or all)
    #[arg(default_value = "all")]
    pub suite: String,

    /// Update golden logs with current output
    #[arg(long)]
    pub update: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let arch = cli.arch.as_str();

    if arch != "aarch64" && arch != "x86_64" {
        bail!("Unsupported architecture: {arch}. Use 'aarch64' or 'x86_64'");
    }

    // Ensure we're in project root
    let project_root = project_root()?;
    std::env::set_current_dir(&project_root)?;

    match cli.command {
        Commands::Test(args) => match args.suite.as_str() {
            "all" => {
                preflight::check_preflight(arch)?;
                println!("🧪 Running test suite for {arch}...\n");
                tests::unit::run()?;
                tests::behavior::run(arch, args.update)?;
                println!("\n✅ Test suite finished!");
            }
            "unit" => tests::unit::run()?,
            "behavior" => tests::behavior::run(arch, args.update)?,
            "serial" => tests::serial_input::run(arch)?,
            "keyboard" => tests::keyboard_input::run(arch)?,
            "shutdown" => tests::shutdown::run(arch)?,
            "debug" => tests::debug_tools::run(args.update)?,
            "screenshot" => tests::screenshot::run(None)?,
            "screenshot:alpine" => tests::screenshot::run(Some("alpine"))?,
            "screenshot:levitate" => tests::screenshot::run(Some("levitate"))?,
            other => bail!("Unknown test suite: {other}. Use 'unit', 'behavior', 'serial', 'keyboard', 'shutdown', 'debug', 'screenshot', or 'all'"),
        },
        Commands::Run(args) => {
            preflight::check_preflight(arch)?;

            // Handle special modes first
            if args.test {
                run::run_qemu_test(arch)?;
            } else if args.verify_gpu {
                run::verify_gpu(arch, args.timeout)?;
            } else if args.vnc {
                run::run_qemu_vnc(arch)?;
            } else if args.term {
                builder::create_initramfs(arch)?;
                run::run_qemu_term_linux(arch)?;
            } else if args.gdb {
                let profile = match args.profile.as_str() {
                    "pixel6" => {
                        if arch != "aarch64" {
                            bail!("Pixel 6 profile only supported on aarch64");
                        }
                        qemu::QemuProfile::Pixel6
                    }
                    _ => if arch == "x86_64" { qemu::QemuProfile::X86_64 } else { qemu::QemuProfile::Default }
                };
                builder::create_initramfs(arch)?;
                run::run_qemu_gdb_linux(profile, args.wait, arch)?;
            } else {
                // Default: run with GUI
                let profile = match args.profile.as_str() {
                    "pixel6" => {
                        if arch != "aarch64" {
                            bail!("Pixel 6 profile only supported on aarch64");
                        }
                        println!("🎯 Running with Pixel 6 profile (8GB RAM, 8 cores)");
                        qemu::QemuProfile::Pixel6
                    }
                    _ => if arch == "x86_64" { qemu::QemuProfile::X86_64 } else { qemu::QemuProfile::Default }
                };
                builder::create_initramfs(arch)?;
                run::run_qemu(profile, args.headless, arch, args.gpu_debug)?;
            }
        },
        Commands::Build(cmd) => {
            preflight::check_preflight(arch)?;
            // TEAM_476: Linux distribution builder - all commands build Linux components
            match cmd {
                builder::BuildCommands::All => builder::build_all(arch)?,
                builder::BuildCommands::Initramfs => builder::create_initramfs(arch)?,
                builder::BuildCommands::Busybox => builder::busybox::build(arch)?,
                builder::BuildCommands::Linux => builder::linux::build_linux_kernel(arch)?,
                builder::BuildCommands::Openrc => builder::openrc::build(arch)?
            }
        },
        Commands::Vm(cmd) => match cmd {
            vm::VmCommands::Start => vm::start(arch)?,
            vm::VmCommands::Stop => vm::stop()?,
            vm::VmCommands::Send { text } => vm::send(&text)?,
            vm::VmCommands::Exec { command, timeout } => { vm::exec(&command, timeout, arch)?; },
            vm::VmCommands::Screenshot { output } => vm::screenshot(&output)?,
            vm::VmCommands::Regs { qmp_socket } => vm::regs(qmp_socket)?,
            vm::VmCommands::Mem { addr, len, qmp_socket } => vm::mem(addr, len, qmp_socket)?,
        },
        Commands::Disk(cmd) => match cmd {
            disk::DiskCommands::Create => disk::create_disk_image_if_missing()?,
            disk::DiskCommands::Install => disk::install_userspace_to_disk(arch)?,
            disk::DiskCommands::Status => disk::show_disk_status()?,
        },
        Commands::Check => {
            preflight::check_preflight(arch)?;
        },
        Commands::Clean => {
            clean::clean(arch)?;
        },
        Commands::Kill => {
            clean::kill_qemu(arch)?;
        },
        Commands::Calc(cmd) => {
            calc::run(cmd)?;
        },
    }

    Ok(())
}

fn project_root() -> Result<PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir())?;

    // If we're in xtask/, go up one level
    if manifest_dir.ends_with("xtask") {
        manifest_dir
            .parent()
            .map(std::path::Path::to_path_buf)
            .ok_or_else(|| anyhow::anyhow!("manifest_dir has no parent"))
    } else {
        Ok(manifest_dir)
    }
}
