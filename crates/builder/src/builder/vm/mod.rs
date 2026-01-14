//! VM interaction commands for background QEMU sessions.

pub mod commands;
pub mod qmp;
pub mod session;

use anyhow::Result;
use clap::Subcommand;

/// Parse hexadecimal number (with or without 0x prefix).
pub fn parse_hex(s: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(s.trim_start_matches("0x"), 16)
}

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
    /// Execute QEMU monitor command
    Qmp {
        /// HMP command to execute
        command: String,
    },
    /// Dump physical memory region
    MemDump {
        /// Physical address (hex, with or without 0x prefix)
        #[arg(value_parser = parse_hex)]
        addr: u64,
        /// Size in bytes
        size: u64,
        /// Output file path
        #[arg(short, long, default_value = "memory.bin")]
        output: String,
    },
    /// Take a screenshot (requires GUI mode)
    Screenshot {
        /// Output file path
        #[arg(short, long, default_value = "screenshot.ppm")]
        output: String,
    },
    /// Reset the VM
    Reset,
}
