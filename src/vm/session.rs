//! VM session management
//!
//! `TEAM_324`: Persistent shell session using QMP sendkey.
//! `TEAM_326`: Moved to vm module for unified VM interaction.
//!
//! Commands:
//! - start: Start VM in background
//! - send: Send keystrokes via QMP
//! - screenshot: Take screenshot
//! - stop: Kill VM and cleanup

use crate::builder;
use crate::qemu::{Arch, QemuBuilder, QemuProfile};
use crate::support::qmp::QmpClient;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::{Command, Stdio};

const SESSION_FILE: &str = ".qemu-session.json";
const QMP_SOCKET: &str = "./qemu-session.sock";

/// Session state persisted to disk
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionState {
    pub pid: u32,
    pub qmp_socket: String,
    pub arch: String,
    pub started_at: String,
}

impl SessionState {
    /// Load session from disk
    pub fn load() -> Result<Option<Self>> {
        if !std::path::Path::new(SESSION_FILE).exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(SESSION_FILE)?;
        let state: SessionState = serde_json::from_str(&content)?;
        Ok(Some(state))
    }

    /// Save session to disk
    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(SESSION_FILE, content)?;
        Ok(())
    }

    /// Remove session file
    pub fn remove() -> Result<()> {
        if std::path::Path::new(SESSION_FILE).exists() {
            fs::remove_file(SESSION_FILE)?;
        }
        Ok(())
    }

    /// Check if the process is still running
    pub fn is_alive(&self) -> bool {
        std::path::Path::new(&format!("/proc/{}", self.pid)).exists()
    }
}

/// Start a new VM session
pub fn start(arch: &str) -> Result<()> {
    println!("🚀 Starting persistent VM session...");
    println!("   Arch: {arch}");

    // Check for existing session
    if let Some(existing) = SessionState::load()? {
        if existing.is_alive() {
            bail!(
                "Session already running (PID {}). Use 'vm stop' first.",
                existing.pid
            );
        }
        println!("⚠️  Cleaning up stale session...");
        SessionState::remove()?;
        let _ = fs::remove_file(&existing.qmp_socket);
    }

    // Clean up socket
    let _ = fs::remove_file(QMP_SOCKET);

    // TEAM_476: Build Linux + OpenRC
    builder::create_openrc_initramfs(arch)?;

    // Build QEMU command
    let arch_enum = Arch::try_from(arch)?;
    let profile = if arch == "x86_64" {
        QemuProfile::X86_64
    } else {
        QemuProfile::Default
    };

    let initrd_path = format!("target/initramfs/{}-openrc.cpio", arch);
    let builder = QemuBuilder::new(arch_enum, profile)
        .display_vnc()
        .enable_qmp(QMP_SOCKET)
        .linux_kernel()
        .initrd(&initrd_path);

    let mut cmd = builder.build()?;

    // Start in background
    let child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start QEMU")?;

    let pid = child.id();

    // Wait for QMP socket
    println!("⏳ Waiting for VM to start...");
    for _ in 0..40 {
        if std::path::Path::new(QMP_SOCKET).exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    if !std::path::Path::new(QMP_SOCKET).exists() {
        bail!("QMP socket not created - QEMU may have failed to start");
    }

    // Save session state
    let state = SessionState {
        pid,
        qmp_socket: QMP_SOCKET.to_string(),
        arch: arch.to_string(),
        started_at: format!("{:?}", std::time::SystemTime::now()),
    };
    state.save()?;

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  ✅ VM session started                                   ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  PID:      {pid}                                        ");
    println!("║  Arch:     {arch}                                       ");
    println!("║  QMP:      {QMP_SOCKET}                            ");
    println!("║  VNC:      localhost:5900                               ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  Commands:                                               ║");
    println!("║    cargo xtask vm send \"ls\"                             ║");
    println!("║    cargo xtask vm screenshot                             ║");
    println!("║    cargo xtask vm regs                                   ║");
    println!("║    cargo xtask vm stop                                   ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    Ok(())
}

/// Send text as keystrokes to running VM
pub fn send(text: &str) -> Result<()> {
    let state = SessionState::load()?
        .ok_or_else(|| anyhow::anyhow!("No session running. Use 'vm start' first."))?;

    if !state.is_alive() {
        SessionState::remove()?;
        bail!(
            "Session died (PID {} not running). Use 'vm start' to restart.",
            state.pid
        );
    }

    println!("📤 Sending: {text}");

    let mut client = QmpClient::connect(&state.qmp_socket)?;

    // Send each character
    for ch in text.chars() {
        send_char(&mut client, ch)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Send Enter
    send_key(&mut client, "ret")?;

    println!("✅ Sent {} characters + Enter", text.len());
    Ok(())
}

/// Take screenshot of running VM
pub fn screenshot(output: &str) -> Result<()> {
    let state = SessionState::load()?
        .ok_or_else(|| anyhow::anyhow!("No session running. Use 'vm start' first."))?;

    if !state.is_alive() {
        SessionState::remove()?;
        bail!("Session died. Use 'vm start' to restart.");
    }

    println!("📸 Taking screenshot...");

    let mut client = QmpClient::connect(&state.qmp_socket)?;

    // Determine output format
    let ppm_output = if output.ends_with(".png") {
        output.replace(".png", ".ppm")
    } else if output.ends_with(".ppm") {
        output.to_string()
    } else {
        format!("{output}.ppm")
    };

    let abs_path = std::env::current_dir()?.join(&ppm_output);
    let args = serde_json::json!({
        "filename": abs_path.to_string_lossy()
    });
    client.execute("screendump", Some(args))?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Convert to PNG if requested
    if output.ends_with(".png") && std::path::Path::new(&ppm_output).exists() {
        if Command::new("magick")
            .args([&ppm_output, output])
            .status()
            .is_ok()
        {
            let _ = fs::remove_file(&ppm_output);
            println!("✅ Screenshot saved: {output}");
        } else {
            println!("✅ Screenshot saved: {ppm_output} (PPM format)");
        }
    } else {
        println!("✅ Screenshot saved: {ppm_output}");
    }

    Ok(())
}

/// Stop the running VM session
pub fn stop() -> Result<()> {
    let state = SessionState::load()?.ok_or_else(|| anyhow::anyhow!("No session running."))?;

    println!("🛑 Stopping session (PID {})...", state.pid);

    // Kill the process
    if state.is_alive() {
        let _ = Command::new("kill")
            .args(["-9", &state.pid.to_string()])
            .status();
    }

    // Cleanup
    let _ = fs::remove_file(&state.qmp_socket);
    SessionState::remove()?;

    println!("✅ Session stopped");
    Ok(())
}

/// Send a single character as a keypress
fn send_char(client: &mut QmpClient, ch: char) -> Result<()> {
    let (key, needs_shift) = char_to_qcode(ch);

    if needs_shift {
        let args = serde_json::json!({
            "keys": [
                {"type": "qcode", "data": "shift"},
                {"type": "qcode", "data": key}
            ]
        });
        client.execute("sendkey", Some(args))?;
    } else {
        send_key(client, key)?;
    }

    Ok(())
}

/// Send a single key
fn send_key(client: &mut QmpClient, qcode: &str) -> Result<()> {
    let args = serde_json::json!({
        "keys": [{"type": "qcode", "data": qcode}]
    });
    client.execute("sendkey", Some(args))?;
    Ok(())
}

/// Convert a character to QEMU qcode
fn char_to_qcode(ch: char) -> (&'static str, bool) {
    match ch {
        'a'..='z' => {
            let idx = (ch as u8 - b'a') as usize;
            let keys = [
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            ];
            (keys[idx], false)
        }
        'A'..='Z' => {
            let idx = (ch as u8 - b'A') as usize;
            let keys = [
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            ];
            (keys[idx], true)
        }
        '0'..='9' => {
            let idx = (ch as u8 - b'0') as usize;
            let keys = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
            (keys[idx], false)
        }
        ' ' => ("spc", false),
        '\n' => ("ret", false),
        '-' => ("minus", false),
        '_' => ("minus", true),
        '=' => ("equal", false),
        '+' => ("equal", true),
        '.' => ("dot", false),
        '>' => ("dot", true),
        ',' => ("comma", false),
        '<' => ("comma", true),
        '/' => ("slash", false),
        '?' => ("slash", true),
        ';' => ("semicolon", false),
        ':' => ("semicolon", true),
        '\'' => ("apostrophe", false),
        '"' => ("apostrophe", true),
        '[' => ("bracket_left", false),
        '{' => ("bracket_left", true),
        ']' => ("bracket_right", false),
        '}' => ("bracket_right", true),
        '\\' => ("backslash", false),
        '|' => ("backslash", true),
        '`' => ("grave_accent", false),
        '~' => ("grave_accent", true),
        '!' => ("1", true),
        '@' => ("2", true),
        '#' => ("3", true),
        '$' => ("4", true),
        '%' => ("5", true),
        '^' => ("6", true),
        '&' => ("7", true),
        '*' => ("8", true),
        '(' => ("9", true),
        ')' => ("0", true),
        _ => ("spc", false),
    }
}
