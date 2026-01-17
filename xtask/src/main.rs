//! Development tasks for LevitateOS.
//!
//! Usage: cargo xtask <command>

mod vm;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development tasks for LevitateOS")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// VM management for testing
    Vm {
        #[command(subcommand)]
        action: VmAction,
    },
}

#[derive(Subcommand)]
enum VmAction {
    /// Start the test VM
    Start {
        /// Run in background (detached)
        #[arg(short, long)]
        detach: bool,

        /// Enable GUI display (default: headless with serial)
        #[arg(short, long)]
        gui: bool,

        /// Memory in MB (default: 4096)
        #[arg(short, long, default_value = "4096")]
        memory: u32,

        /// Number of CPUs (default: 4)
        #[arg(short, long, default_value = "4")]
        cpus: u32,

        /// Boot from ISO/CDROM (for installation)
        #[arg(long)]
        cdrom: Option<String>,

        /// Use UEFI boot (requires OVMF)
        #[arg(long)]
        uefi: bool,
    },

    /// Stop the running VM
    Stop,

    /// Show VM status
    Status,

    /// Send a command to the VM via SSH
    Send {
        /// Command to execute
        command: Vec<String>,
    },

    /// Show VM serial console log
    Log {
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },

    /// SSH into the VM
    Ssh,

    /// Create/setup the base Arch Linux image
    Setup {
        /// Force recreation even if image exists
        #[arg(short, long)]
        force: bool,
    },

    /// Build recipe binary and prepare files for VM
    Prepare,

    /// Show the install script to run inside VM
    InstallScript,

    /// Copy recipe binary and recipes to running VM
    Copy,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Vm { action } => match action {
            VmAction::Start { detach, gui, memory, cpus, cdrom, uefi } => {
                vm::start(detach, gui, memory, cpus, cdrom, uefi)
            }
            VmAction::Stop => vm::stop(),
            VmAction::Status => vm::status(),
            VmAction::Send { command } => vm::send(&command.join(" ")),
            VmAction::Log { follow } => vm::log(follow),
            VmAction::Ssh => vm::ssh(),
            VmAction::Setup { force } => vm::setup(force),
            VmAction::Prepare => vm::prepare(),
            VmAction::InstallScript => vm::install_script(),
            VmAction::Copy => vm::copy_files(),
        },
    }
}
