//! VM interaction commands for background QEMU sessions.

pub mod commands;
pub mod qmp;
pub mod session;

use clap::Subcommand;

/// VM subcommands.
#[derive(Subcommand)]
pub enum VmCommands {
    /// Start VM in background
    Start {
        /// Start with debug shell (busybox) instead of login
        #[arg(long)]
        debug: bool,
    },
    /// Stop running VM
    Stop,
    /// Send text to VM serial
    Send {
        /// Text to send (followed by Enter)
        text: String,
    },
    /// Execute command in VM and capture output
    Exec {
        /// Command to run
        command: String,
        /// Timeout in seconds
        #[arg(long, default_value = "5")]
        timeout: u64,
    },
    /// Show VM status
    Status,
    /// View VM output log
    Log {
        /// Print once instead of following
        #[arg(long)]
        no_follow: bool,
    },
    /// Dump VM debug info (files, processes)
    Debug,
}
