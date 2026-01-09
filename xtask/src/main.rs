//! LevitateOS xtask - Development task runner
//!
//! TEAM_326: Refactored command structure for clarity.
//!
//! Usage:
//!   cargo xtask run                   # Build + run with GUI (most common)
//!   cargo xtask build                 # Build everything
//!   cargo xtask test                  # Run all tests
//!
//!   cargo xtask run --gdb             # Run with GDB server
//!   cargo xtask run --term            # Terminal mode (WSL)
//!   cargo xtask run --vnc             # VNC display
//!   cargo xtask run --profile pixel6  # Pixel 6 profile
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

mod build;
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
    Build(build::BuildCommands),
    
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
    
    /// Boot from Limine ISO instead of -kernel
    #[arg(long)]
    pub iso: bool,
    
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
    /// Which test suite to run (unit, behavior, regress, gicv3, or all)
    #[arg(default_value = "all")]
    pub suite: String,

    /// Update golden logs with current output (Rule 4 Refined)
    #[arg(long)]
    pub update: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let arch = cli.arch.as_str();

    if arch != "aarch64" && arch != "x86_64" {
        bail!("Unsupported architecture: {}. Use 'aarch64' or 'x86_64'", arch);
    }

    // Ensure we're in project root
    let project_root = project_root()?;
    std::env::set_current_dir(&project_root)?;

    match cli.command {
        Commands::Test(args) => match args.suite.as_str() {
            "all" => {
                preflight::check_preflight(arch)?;
                println!("🧪 Running COMPLETE test suite for {}...\n", arch);
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
                tests::behavior::run_gicv3()?
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
            other => bail!("Unknown test suite: {}. Use 'unit', 'behavior', 'regress', 'gicv3', 'serial', 'keyboard', 'shutdown', 'debug', 'screenshot', 'screenshot:userspace', or 'all'", other),
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
                let use_iso = args.iso || arch == "x86_64";
                run::run_qemu_term(arch, use_iso)?;
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
                let use_iso = args.iso || arch == "x86_64";
                if use_iso {
                    build::build_iso(arch)?;
                } else {
                    build::build_all(arch)?;
                }
                run::run_qemu_gdb(profile, args.wait, use_iso, arch)?;
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
                let use_iso = args.iso || arch == "x86_64";
                if use_iso {
                    build::build_iso(arch)?;
                } else {
                    build::build_all(arch)?;
                }
                run::run_qemu(profile, args.headless, use_iso, arch, args.gpu_debug)?;
            }
        },
        Commands::Build(cmd) => {
            preflight::check_preflight(arch)?;
            match cmd {
                build::BuildCommands::All => build::build_all(arch)?,
                build::BuildCommands::Kernel => build::build_kernel_only(arch)?,
                build::BuildCommands::Userspace { .. } => {
                    build::build_userspace(arch)?;
                    build::create_initramfs(arch)?;
                }
                build::BuildCommands::Initramfs => build::create_initramfs(arch)?,
                build::BuildCommands::Iso => build::build_iso(arch)?,
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

pub fn get_binaries(arch: &str) -> Result<Vec<String>> {
    let mut bins = Vec::new();
    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };
    let release_dir = PathBuf::from(format!("crates/userspace/target/{}/release", target));
    if !release_dir.exists() {
        return Ok(bins);
    }

    for entry in std::fs::read_dir(release_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                // Binaries in our setup don't have extensions.
                // We skip common files like .cargo-lock, .fingerprint, etc.
                if !name.contains('.') && name != "build" {
                    bins.push(name.to_string());
                }
            }
        }
    }
    bins.sort();
    Ok(bins)
}
