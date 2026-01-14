//! VM control commands.

mod commands;
pub mod qmp;
mod session;

use anyhow::Result;
use clap::Subcommand;

/// Parse hexadecimal number (with or without 0x prefix).
fn parse_hex(s: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(s.trim_start_matches("0x"), 16)
}

#[derive(Subcommand)]
pub enum VmCommand {
    /// Start VM in background
    Start,

    /// Stop running VM
    Stop,

    /// Send text to VM serial console
    Send {
        /// Text to send
        text: String,
    },

    /// Show VM status
    Status,

    /// View VM output log
    Log {
        /// Print once instead of following
        #[clap(long)]
        no_follow: bool,
    },

    /// Execute QEMU monitor command
    Qmp {
        /// HMP command to execute
        command: String,
    },

    /// Dump physical memory region
    MemDump {
        /// Physical address (hex, with or without 0x prefix)
        #[clap(value_parser = parse_hex)]
        addr: u64,
        /// Size in bytes
        size: u64,
        /// Output file path
        #[clap(short, long, default_value = "memory.bin")]
        output: String,
    },

    /// Take a screenshot (requires GUI mode)
    Screenshot {
        /// Output file path
        #[clap(short, long, default_value = "screenshot.ppm")]
        output: String,
    },

    /// Reset the VM
    Reset,
}

pub fn run(cmd: &VmCommand) -> Result<()> {
    match cmd {
        VmCommand::Start => commands::start(),
        VmCommand::Stop => commands::stop(),
        VmCommand::Send { text } => commands::send(text),
        VmCommand::Status => commands::status(),
        VmCommand::Log { no_follow } => commands::log(!no_follow),
        VmCommand::Qmp { command } => commands::qmp_command(command),
        VmCommand::MemDump { addr, size, output } => commands::memory_dump(*addr, *size, output),
        VmCommand::Screenshot { output } => commands::screenshot(output),
        VmCommand::Reset => commands::reset(),
    }
}
