//! VM interaction module
//!
//! `TEAM_326`: Merged shell + debug commands into unified vm module.
//!
//! Commands:
//! - start: Start persistent VM session
//! - stop: Stop VM session
//! - send: Send keystrokes to running VM
//! - exec: One-shot command execution (ephemeral)
//! - screenshot: Take screenshot of running VM
//! - regs: Dump CPU registers
//! - mem: Dump memory

mod debug;
mod exec;
mod session;

pub use debug::{mem, regs};
pub use exec::exec;
pub use session::{screenshot, send, start, stop};

use clap::Subcommand;

#[derive(Subcommand)]
pub enum VmCommands {
    /// Start a persistent VM session
    Start,

    /// Stop the running VM session
    Stop,

    /// Send text as keystrokes to running VM (Enter auto-appended)
    Send {
        /// Text to send
        text: String,
    },

    /// Execute a command in a fresh VM (slow, one-shot)
    Exec {
        /// Command to run inside the VM
        command: String,

        /// Timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u32,
    },

    /// Take screenshot of running VM
    Screenshot {
        /// Output filename
        #[arg(default_value = "tests/screenshots/vm_screenshot.png")]
        output: String,
    },

    /// Dump CPU registers from running VM
    Regs {
        /// QMP socket path (auto-detected if not specified)
        #[arg(long)]
        qmp_socket: Option<String>,
    },

    /// Dump memory from running VM
    Mem {
        /// Memory address to dump (hex, e.g., 0xffff8000)
        #[arg(value_parser = parse_hex_addr)]
        addr: u64,

        /// Number of bytes to dump
        #[arg(long, default_value = "256")]
        len: usize,

        /// QMP socket path (auto-detected if not specified)
        #[arg(long)]
        qmp_socket: Option<String>,
    },
}

/// Parse hex address from string (supports 0x prefix)
fn parse_hex_addr(s: &str) -> Result<u64, String> {
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    u64::from_str_radix(s, 16).map_err(|e| format!("Invalid hex address: {e}"))
}
