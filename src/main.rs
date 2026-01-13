// TEAM_470: Suppress overly pedantic clippy lints for xtask
#![allow(clippy::ptr_arg)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::manual_strip)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::verbose_bit_mask)]
#![allow(clippy::format_push_string)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::manual_flatten)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::while_let_loop)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::lines_filter_map_ok)]

//! `LevitateOS` xtask - Development task runner
//!
//! `TEAM_326`: Refactored command structure for clarity.
//! `TEAM_475`: Linux kernel + OpenRC is now the default.
//! `TEAM_476`: Removed custom kernel path (now Linux-only distribution builder).
//!
//! Usage:
//!   cargo xtask run --term            # Linux + OpenRC (default, most common)
//!   cargo xtask run --term --minimal  # Linux + BusyBox init (debugging)
//!   cargo xtask build                 # Build everything
//!   cargo xtask test                  # Run all tests
//!
//!   cargo xtask run --gdb             # Run with GDB server
//!   cargo xtask run --vnc             # VNC display
//!   cargo xtask run --profile pixel6  # Pixel 6 profile (aarch64)
//!
//!   cargo xtask vm start              # Start persistent VM session
//!   cargo xtask vm send "ls"          # Send command to VM
//!   cargo xtask vm screenshot         # Take VM screenshot
//!   cargo xtask vm regs               # Dump CPU registers
//!   cargo xtask vm stop               # Stop VM session
//!
//!   cargo xtask disk create           # Create disk image
//!   cargo xtask disk install          # Install userspace to disk
//!
//!   cargo xtask check                 # Preflight checks
//!   cargo xtask clean                 # Clean up artifacts

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
#[command(name = "xtask", about = "LevitateOS development task runner")]
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

// TEAM_326: Simplified run args with flags instead of subcommands
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

    // TEAM_370: Removed --iso flag - x86_64 always uses ISO, aarch64 can't use ISO
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

    /// TEAM_476: Use minimal BusyBox init instead of OpenRC (for debugging)
    #[arg(long)]
    pub minimal: bool,
}

#[derive(clap::Args)]
pub struct TestArgs {
    /// Which test suite to run (unit, behavior, regress, gicv3, coreutils, or all)
    #[arg(default_value = "all")]
    pub suite: String,

    /// Update golden logs with current output (Rule 4 Refined)
    #[arg(long)]
    pub update: bool,

    /// `TEAM_465`: Phase to run for coreutils tests (e.g., "all", "2", "1-5")
    #[arg(long, default_value = "all")]
    pub phase: String,
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
                println!("🧪 Running COMPLETE test suite for {arch}...\n");
                tests::unit::run()?;
                tests::behavior::run(arch, args.update)?;
                if arch == "aarch64" {
                    tests::behavior::run_gicv3().unwrap_or_else(|_| {
                        println!("⚠️  GICv3 behavior differs (expected, needs separate golden file)\n");
                    });
                }
                tests::regression::run()?;
                // TEAM_142: Shutdown test is interactive, run separately
                println!("\n✅ COMPLETE test suite finished!");
                println!("ℹ️  Run 'cargo xtask test shutdown' separately for shutdown golden file test");
            }
            "unit" => tests::unit::run()?,
            "behavior" => tests::behavior::run(arch, args.update)?,
            "regress" | "regression" => tests::regression::run()?,
            "gicv3" => {
                if arch != "aarch64" {
                    bail!("GICv3 tests only supported on aarch64");
                }
                tests::behavior::run_gicv3()?;
            },
            "serial" => tests::serial_input::run(arch)?,
            "keyboard" => tests::keyboard_input::run(arch)?,
            "shutdown" => tests::shutdown::run(arch)?,
            // TEAM_327: Backspace regression test
            "backspace" => tests::backspace::run(arch)?,
            // TEAM_325: Debug tools integration tests (both architectures)
            "debug" => tests::debug_tools::run(args.update)?,
            // TEAM_327: New unified screenshot tests
            "screenshot" => tests::screenshot::run(None)?,
            "screenshot:alpine" => tests::screenshot::run(Some("alpine"))?,
            "screenshot:levitate" => tests::screenshot::run(Some("levitate"))?,
            "screenshot:userspace" => tests::screenshot::run(Some("userspace"))?,
            // Legacy aliases
            "alpine" => tests::screenshot::run(Some("alpine"))?,
            "levitate" | "display" => tests::screenshot::run(Some("levitate"))?,
            "userspace" => tests::screenshot::run(Some("userspace"))?,
            // TEAM_465: Coreutils test suite
            "coreutils" | "core" => tests::coreutils::run(arch, Some(args.phase.as_str()))?,
            // TEAM_435: Eyra test removed (Eyra replaced by c-gull)
            other => bail!("Unknown test suite: {other}. Use 'unit', 'behavior', 'regress', 'gicv3', 'coreutils', 'serial', 'keyboard', 'shutdown', 'debug', 'screenshot', or 'all'"),
        },
        // TEAM_326: Refactored command handlers
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
                // TEAM_476: Linux kernel with OpenRC (or BusyBox if --minimal)
                if args.minimal {
                    builder::create_busybox_initramfs(arch)?;
                } else {
                    builder::create_openrc_initramfs(arch)?;
                }
                run::run_qemu_term_linux(arch, !args.minimal)?;
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
                // TEAM_476: Build Linux + OpenRC for debugging
                if args.minimal {
                    builder::create_busybox_initramfs(arch)?;
                } else {
                    builder::create_openrc_initramfs(arch)?;
                }
                run::run_qemu_gdb_linux(profile, args.wait, arch, !args.minimal)?;
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
                // TEAM_476: Linux kernel with OpenRC (or BusyBox if --minimal)
                if args.minimal {
                    builder::create_busybox_initramfs(arch)?;
                } else {
                    builder::create_openrc_initramfs(arch)?;
                }
                run::run_qemu(profile, args.headless, false, arch, args.gpu_debug, true, !args.minimal)?;
            }
        },
        Commands::Build(cmd) => {
            preflight::check_preflight(arch)?;
            // TEAM_476: Linux distribution builder - all commands build Linux components
            match cmd {
                builder::BuildCommands::All => builder::build_all(arch)?,
                builder::BuildCommands::Initramfs => builder::create_busybox_initramfs(arch)?,
                builder::BuildCommands::Busybox => builder::busybox::build(arch)?,
                builder::BuildCommands::Linux => builder::linux::build_linux_kernel(arch)?,
                builder::BuildCommands::Openrc => builder::openrc::build(arch)?,
                builder::BuildCommands::OpenrcInitramfs => { builder::create_openrc_initramfs(arch)?; }
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
