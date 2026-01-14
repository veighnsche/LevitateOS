//! VM command implementations.

use super::{qmp, session};
use crate::builder::{initramfs, linux};
use anyhow::{bail, Context, Result};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::process::Command;

const OUTPUT_FILE: &str = "build/vm-output.log";
const QMP_SOCKET: &str = "/tmp/levitate-qemu-qmp.sock";
const SERIAL_SOCKET: &str = "/tmp/levitate-serial.sock";

/// Start VM in background.
pub fn start() -> Result<()> {
    // Check for existing session
    if session::exists() {
        let existing = session::load()?;
        if existing.is_alive() {
            bail!(
                "VM already running (PID {}). Use 'vm stop' first.",
                existing.pid
            );
        }
        // Stale session, clean up
        session::clear()?;
    }

    // Clean up stale sockets
    for socket in [QMP_SOCKET, SERIAL_SOCKET] {
        if std::path::Path::new(socket).exists() {
            std::fs::remove_file(socket)?;
        }
    }

    // Ensure build directory exists
    std::fs::create_dir_all("build")?;

    // Clear previous output
    std::fs::write(OUTPUT_FILE, "")?;

    // Start QEMU with socket-based serial that logs to file
    // The logfile option captures all I/O while still allowing us to connect
    let child = Command::new("qemu-system-x86_64")
        .args([
            "-kernel",
            linux::kernel_path(),
            "-initrd",
            initramfs::cpio_path(),
            "-append",
            "console=ttyS0 rw",
            "-m",
            "512M",
            "-no-reboot",
            "-display",
            "none",
            "-chardev",
            &format!(
                "socket,id=serial0,path={},server=on,wait=off,logfile={}",
                SERIAL_SOCKET, OUTPUT_FILE
            ),
            "-serial",
            "chardev:serial0",
            "-qmp",
            &format!("unix:{},server,nowait", QMP_SOCKET),
        ])
        .spawn()
        .context("Failed to start QEMU")?;

    let now = chrono::Utc::now().to_rfc3339();
    session::save(&session::Session {
        pid: child.id(),
        qmp_socket: QMP_SOCKET.to_string(),
        serial_socket: SERIAL_SOCKET.to_string(),
        output_file: OUTPUT_FILE.to_string(),
        started_at: now,
    })?;

    // Wait briefly for QEMU to create the sockets
    std::thread::sleep(std::time::Duration::from_millis(500));

    println!("VM started (PID {})", child.id());
    println!("Output: {}", OUTPUT_FILE);
    println!("\nCommands:");
    println!("  cargo run -- vm status    # Check status");
    println!("  cargo run -- vm log       # View output");
    println!("  cargo run -- vm send TEXT # Send text to VM");
    println!("  cargo run -- vm stop      # Stop VM");

    Ok(())
}

/// Stop running VM.
pub fn stop() -> Result<()> {
    let session = session::load()?;

    if !session.is_alive() {
        println!("VM not running (stale session).");
        session::clear()?;
        return Ok(());
    }

    let pid_str = session.pid.to_string();

    // Try graceful shutdown via QMP
    let mut qmp_succeeded = false;
    match qmp::QmpClient::connect(&session.qmp_socket) {
        Ok(mut client) => {
            if client.handshake().is_ok() {
                let _ = client.quit();
                qmp_succeeded = true;
                println!("Sent quit command to VM.");
            }
        }
        Err(_) => {
            println!("QMP unavailable, will force kill.");
        }
    }

    // Wait briefly for graceful shutdown
    if qmp_succeeded {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Check if process is still alive and force kill if needed
    if let Ok(alive) = Command::new("kill")
        .args(["-0", &pid_str])
        .status()
    {
        if alive.success() {
            // Process still exists, force kill it
            println!("VM still running, force killing...");
            let _ = Command::new("kill")
                .args(["-9", &pid_str])
                .status();

            // Wait for kill to take effect
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    session::clear()?;
    println!("VM stopped.");

    Ok(())
}

/// Send text to VM via serial console.
pub fn send(text: &str) -> Result<()> {
    let session = session::load()?;

    if !session.is_alive() {
        bail!("VM not running.");
    }

    // Connect to serial socket and send text
    let mut stream = UnixStream::connect(&session.serial_socket)
        .context("Failed to connect to serial socket")?;

    // Set timeouts
    stream.set_write_timeout(Some(std::time::Duration::from_secs(5)))?;

    // Send text followed by carriage return (terminal expects CR, not just LF)
    let data = format!("{}\r", text);
    stream
        .write_all(data.as_bytes())
        .context("Failed to write to serial")?;
    stream.flush()?;

    // Keep connection open briefly to ensure delivery
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("Sent: {}", text);
    Ok(())
}

/// Show VM status.
pub fn status() -> Result<()> {
    if !session::exists() {
        println!("No VM session.");
        return Ok(());
    }

    let session = session::load()?;
    let alive = session.is_alive();

    println!("VM Status:");
    println!("  PID: {} ({})", session.pid, if alive { "running" } else { "dead" });
    println!("  Started: {}", session.started_at);
    println!("  Output: {}", session.output_file);
    println!("  Serial: {}", session.serial_socket);
    println!("  QMP: {}", session.qmp_socket);

    if !alive {
        println!("\nNote: Session is stale. Run 'vm stop' to clean up.");
    }

    Ok(())
}

/// View VM output log.
pub fn log(follow: bool) -> Result<()> {
    let output_file = if session::exists() {
        let session = session::load()?;
        session.output_file
    } else {
        // No session, but maybe output file exists from previous run
        OUTPUT_FILE.to_string()
    };

    if !std::path::Path::new(&output_file).exists() {
        bail!("Output file not found: {}", output_file);
    }

    if follow {
        // Use tail -f
        let status = Command::new("tail")
            .args(["-f", &output_file])
            .status()
            .context("Failed to run tail")?;

        if !status.success() {
            bail!("tail exited with error");
        }
    } else {
        // Just print contents
        let contents = std::fs::read_to_string(&output_file)?;
        print!("{}", contents);
    }

    Ok(())
}

/// Execute QEMU monitor command via QMP.
pub fn qmp_command(cmd: &str) -> Result<()> {
    let session = session::load()?;

    if !session.is_alive() {
        bail!("VM not running");
    }

    let mut client = qmp::QmpClient::connect(&session.qmp_socket)
        .context("Failed to connect to QMP socket")?;
    client.handshake().context("QMP handshake failed")?;

    let output = client.human_monitor_command(cmd)?;
    println!("{}", output);

    Ok(())
}

/// Dump physical memory to file.
pub fn memory_dump(addr: u64, size: u64, output: &str) -> Result<()> {
    let session = session::load()?;

    if !session.is_alive() {
        bail!("VM not running");
    }

    let mut client = qmp::QmpClient::connect(&session.qmp_socket)
        .context("Failed to connect to QMP socket")?;
    client.handshake().context("QMP handshake failed")?;

    println!("Dumping {} bytes from 0x{:x} to {}...", size, addr, output);
    client.pmemsave(addr, size, output)?;
    println!("Memory dump complete");

    Ok(())
}

/// Take a screenshot of the VM display.
pub fn screenshot(output: &str) -> Result<()> {
    let session = session::load()?;

    if !session.is_alive() {
        bail!("VM not running");
    }

    let mut client = qmp::QmpClient::connect(&session.qmp_socket)
        .context("Failed to connect to QMP socket")?;
    client.handshake().context("QMP handshake failed")?;

    println!("Taking screenshot...");
    client.screendump(output)?;
    println!("Screenshot saved to {}", output);

    Ok(())
}

/// Reset the VM.
pub fn reset() -> Result<()> {
    let session = session::load()?;

    if !session.is_alive() {
        bail!("VM not running");
    }

    let mut client = qmp::QmpClient::connect(&session.qmp_socket)
        .context("Failed to connect to QMP socket")?;
    client.handshake().context("QMP handshake failed")?;

    println!("Resetting VM...");
    client.system_reset()?;
    println!("VM reset complete");

    Ok(())
}
