//! VM Debug commands
//!
//! `TEAM_325`: Implements debug tools for inspecting running VMs via QMP.
//! `TEAM_326`: Moved to vm module for unified VM interaction.
//!
//! Commands:
//! - regs: Dump CPU registers via QMP human-monitor-command
//! - mem: Dump memory via QMP memsave

use crate::support::qmp::QmpClient;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const DEFAULT_QMP_SOCKET: &str = "./qmp.sock";
const SESSION_QMP_SOCKET: &str = "./qemu-session.sock";

/// Find an available QMP socket
fn find_qmp_socket(override_path: Option<&str>) -> Result<String> {
    if let Some(path) = override_path {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        }
        anyhow::bail!("QMP socket not found: {path}");
    }

    // Try session socket first, then default
    if Path::new(SESSION_QMP_SOCKET).exists() {
        Ok(SESSION_QMP_SOCKET.to_string())
    } else if Path::new(DEFAULT_QMP_SOCKET).exists() {
        Ok(DEFAULT_QMP_SOCKET.to_string())
    } else {
        anyhow::bail!(
            "No QMP socket found. Start a VM with:\n  \
             cargo xtask vm start\n  \
             or\n  \
             cargo xtask run (with QMP enabled)"
        )
    }
}

/// Dump CPU registers via QMP human-monitor-command
pub fn regs(qmp_socket: Option<String>) -> Result<()> {
    let socket = find_qmp_socket(qmp_socket.as_deref())?;
    println!("🔍 Connecting to QMP socket: {socket}");

    let mut client = QmpClient::connect(&socket)?;

    let args = serde_json::json!({
        "command-line": "info registers"
    });

    let result = client.execute("human-monitor-command", Some(args))?;

    if let Some(output) = result.as_str() {
        println!();
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║  CPU Registers                                           ║");
        println!("╚══════════════════════════════════════════════════════════╝");
        println!();
        println!("{output}");
    } else {
        println!("📋 Register dump:");
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

/// Dump memory via QMP memsave
pub fn mem(addr: u64, len: usize, qmp_socket: Option<String>) -> Result<()> {
    let socket = find_qmp_socket(qmp_socket.as_deref())?;
    println!("🔍 Connecting to QMP socket: {socket}");
    println!("📍 Address: 0x{addr:x}, Length: {len} bytes");

    let mut client = QmpClient::connect(&socket)?;

    let tmp_file = format!("/tmp/levitate_memdump_{}.bin", std::process::id());

    let args = serde_json::json!({
        "val": addr,
        "size": len,
        "filename": &tmp_file
    });

    client
        .execute("memsave", Some(args))
        .context("memsave command failed")?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    let data = fs::read(&tmp_file).context("Failed to read memory dump file")?;
    let _ = fs::remove_file(&tmp_file);

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  Memory Dump                                             ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    hexdump(&data, addr);

    Ok(())
}

/// Display data as a hex dump with ASCII sidebar
fn hexdump(data: &[u8], base_addr: u64) {
    const BYTES_PER_LINE: usize = 16;

    for (i, chunk) in data.chunks(BYTES_PER_LINE).enumerate() {
        let offset = i * BYTES_PER_LINE;
        let addr = base_addr + offset as u64;

        print!("0x{addr:08x}: ");

        for (j, byte) in chunk.iter().enumerate() {
            print!("{byte:02x} ");
            if j == 7 {
                print!(" ");
            }
        }

        if chunk.len() < BYTES_PER_LINE {
            for j in chunk.len()..BYTES_PER_LINE {
                print!("   ");
                if j == 7 {
                    print!(" ");
                }
            }
        }

        print!(" |");
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                print!("{}", *byte as char);
            } else {
                print!(".");
            }
        }
        println!("|");
    }
}
