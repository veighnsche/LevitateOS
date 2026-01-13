#![allow(dead_code)]
//! Common test utilities
//!
//! `TEAM_327`: Shared infrastructure for all integration tests.
//! Eliminates code duplication across test files.

use anyhow::{bail, Context, Result};
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::qemu::{Arch, QemuBuilder, QemuProfile};
use crate::support::qmp::QmpClient;

/// A running QEMU session with stdin/stdout access
pub struct QemuSession {
    pub child: Child,
    pub stdin: std::process::ChildStdin,
    pub stdout: std::process::ChildStdout,
    pub arch: String,
    qmp_socket: Option<String>,
}

impl QemuSession {
    /// Start a new QEMU session for `LevitateOS`
    pub fn start(arch: &str, with_qmp: bool) -> Result<Self> {
        let arch_enum = Arch::try_from(arch)?;
        let profile = if arch == "x86_64" {
            QemuProfile::X86_64
        } else {
            QemuProfile::Default
        };

        let qmp_socket = if with_qmp {
            Some(format!("./test-session-{}.sock", std::process::id()))
        } else {
            None
        };

        // Clean up any existing socket
        if let Some(ref socket) = qmp_socket {
            let _ = std::fs::remove_file(socket);
        }

        let mut builder = QemuBuilder::new(arch_enum, profile).display_nographic();

        if let Some(ref socket) = qmp_socket {
            builder = builder.enable_qmp(socket);
        }

        if arch == "x86_64" {
            builder = builder.boot_iso();
        }

        let mut cmd = builder.build()?;

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to start QEMU")?;

        let stdin = child.stdin.take().expect("Failed to get stdin");
        let stdout = child.stdout.take().expect("Failed to get stdout");

        // Set stdout to non-blocking
        let fd = stdout.as_raw_fd();
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFL);
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        Ok(Self {
            child,
            stdin,
            stdout,
            arch: arch.to_string(),
            qmp_socket,
        })
    }

    /// Start a VNC-enabled session for screenshot capture
    pub fn start_vnc(arch: &str) -> Result<(Self, String)> {
        let arch_enum = Arch::try_from(arch)?;
        let profile = if arch == "x86_64" {
            QemuProfile::X86_64
        } else {
            QemuProfile::Default
        };

        let qmp_socket = format!("./test-vnc-{}.sock", std::process::id());
        let _ = std::fs::remove_file(&qmp_socket);

        let mut builder = QemuBuilder::new(arch_enum, profile)
            .display_vnc()
            .enable_qmp(&qmp_socket);

        if arch == "x86_64" {
            builder = builder.boot_iso();
        }

        let mut cmd = builder.build()?;

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to start QEMU")?;

        let stdin = child.stdin.take().expect("Failed to get stdin");
        let stdout = child.stdout.take().expect("Failed to get stdout");

        // Set stdout to non-blocking
        let fd = stdout.as_raw_fd();
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFL);
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        let session = Self {
            child,
            stdin,
            stdout,
            arch: arch.to_string(),
            qmp_socket: Some(qmp_socket.clone()),
        };

        Ok((session, qmp_socket))
    }

    /// Wait for shell prompt, returns all boot output
    pub fn wait_for_prompt(&mut self, timeout_secs: u64) -> Result<String> {
        let mut output = String::new();
        let mut buf = [0u8; 4096];
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            match self.stdout.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    output.push_str(&chunk);
                    if output.contains("# ") || output.contains("$ ") {
                        return Ok(output);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }

        if output.contains("# ") || output.contains("$ ") {
            Ok(output)
        } else {
            bail!("Shell prompt not found within {timeout_secs}s. Output:\n{output}")
        }
    }

    /// Send a command and wait for response
    pub fn send_command(&mut self, cmd: &str, wait_ms: u64) -> Result<String> {
        self.stdin.write_all(cmd.as_bytes())?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()?;

        std::thread::sleep(Duration::from_millis(wait_ms));

        let mut output = String::new();
        let mut buf = [0u8; 4096];

        loop {
            match self.stdout.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    output.push_str(&String::from_utf8_lossy(&buf[..n]));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        Ok(output)
    }

    /// Send raw bytes
    pub fn send_bytes(&mut self, data: &[u8]) -> Result<()> {
        self.stdin.write_all(data)?;
        self.stdin.flush()?;
        Ok(())
    }

    /// Read available output (non-blocking)
    pub fn read_output(&mut self) -> String {
        let mut output = String::new();
        let mut buf = [0u8; 4096];

        loop {
            match self.stdout.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    output.push_str(&String::from_utf8_lossy(&buf[..n]));
                }
                Err(_) => break,
            }
        }

        output
    }

    /// Get QMP client if QMP is enabled
    pub fn qmp_client(&self) -> Result<QmpClient> {
        let socket = self
            .qmp_socket
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("QMP not enabled for this session"))?;
        QmpClient::connect(socket)
    }

    /// Take a screenshot via QMP (requires VNC mode)
    pub fn screenshot(&self, output_path: &str) -> Result<()> {
        let mut client = self.qmp_client()?;
        let abs_path = std::env::current_dir()?.join(output_path);
        let args = serde_json::json!({
            "filename": abs_path.to_string_lossy()
        });
        client.execute("screendump", Some(args))?;
        std::thread::sleep(Duration::from_millis(500));

        // Convert PPM to PNG if ImageMagick available
        if output_path.ends_with(".ppm") {
            let png_path = output_path.replace(".ppm", ".png");
            let status = Command::new("magick")
                .args([output_path, &png_path])
                .status();

            if status.is_ok() && status.unwrap().success() {
                let _ = std::fs::remove_file(output_path);
            }
        }

        Ok(())
    }

    /// Clean up and terminate
    pub fn cleanup(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(ref socket) = self.qmp_socket {
            let _ = std::fs::remove_file(socket);
        }
    }
}

impl Drop for QemuSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(ref socket) = self.qmp_socket {
            let _ = std::fs::remove_file(socket);
        }
    }
}

/// Wait for a QMP socket file to appear
pub fn wait_for_qmp_socket(socket: &str, timeout_secs: u64) -> Result<()> {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if std::path::Path::new(socket).exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(250));
    }

    bail!("QMP socket not created within {timeout_secs}s")
}

/// Send keys via QMP human-monitor-command
pub fn qmp_send_keys(client: &mut QmpClient, text: &str) -> Result<()> {
    for ch in text.chars() {
        let (key, needs_shift) = char_to_qcode(ch);
        let cmd = if needs_shift {
            format!("sendkey shift-{key}")
        } else {
            format!("sendkey {key}")
        };
        let args = serde_json::json!({ "command-line": cmd });
        client.execute("human-monitor-command", Some(args))?;
        std::thread::sleep(Duration::from_millis(30));
    }
    Ok(())
}

/// Send a single key via QMP
pub fn qmp_send_key(client: &mut QmpClient, qcode: &str) -> Result<()> {
    let cmd = format!("sendkey {qcode}");
    let args = serde_json::json!({ "command-line": cmd });
    client.execute("human-monitor-command", Some(args))?;
    Ok(())
}

/// Convert character to QEMU qcode
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
        '/' => ("slash", false),
        '.' => ("dot", false),
        '=' => ("equal", false),
        '+' => ("equal", true),
        '[' => ("bracket_left", false),
        ']' => ("bracket_right", false),
        ';' => ("semicolon", false),
        ':' => ("semicolon", true),
        '\'' => ("apostrophe", false),
        '"' => ("apostrophe", true),
        ',' => ("comma", false),
        '<' => ("comma", true),
        '>' => ("dot", true),
        '?' => ("slash", true),
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
