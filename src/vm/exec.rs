//! VM exec command
//!
//! TEAM_323: Run arbitrary commands inside the VM shell from the host.
//! TEAM_326: Moved to vm module for unified VM interaction.
//! TEAM_476: Updated to use Linux + OpenRC boot.
//!
//! This works by:
//! 1. Starting QEMU headless with stdin/stdout piped
//! 2. Waiting for shell prompt
//! 3. Sending the command
//! 4. Capturing output until next prompt
//! 5. Returning the output

use crate::builder;
use crate::qemu::{Arch, QemuBuilder, QemuProfile};
use anyhow::{bail, Context, Result};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Execute a command in the VM shell and return the output
pub fn exec(cmd: &str, timeout_secs: u32, arch: &str) -> Result<String> {
    println!("🐚 Executing in VM: {cmd}");
    println!("   Arch: {arch}, Timeout: {timeout_secs}s");
    println!();

    // TEAM_476: Build Linux + OpenRC
    builder::create_initramfs(arch)?;

    // TEAM_476: Use QemuBuilder for Linux + OpenRC
    let arch_enum = Arch::try_from(arch)?;
    let profile = if arch == "x86_64" {
        QemuProfile::X86_64
    } else {
        QemuProfile::Default
    };

    let initrd_path = format!("target/initramfs/{}-openrc.cpio", arch);
    let builder = QemuBuilder::new(arch_enum, profile)
        .display_nographic()
        
        .initrd(&initrd_path);

    let base_cmd = builder.build()?;
    let args: Vec<_> = base_cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

    println!("🚀 Starting QEMU...");

    let mut child = Command::new(arch_enum.qemu_binary())
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start QEMU")?;

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let mut stdout = child.stdout.take().expect("Failed to get stdout");

    // Set stdout to non-blocking
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fd = stdout.as_raw_fd();
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFL);
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
    }

    let mut all_output = String::new();
    let mut buf = [0u8; 4096];
    let timeout = Duration::from_secs(u64::from(timeout_secs));
    let start = Instant::now();

    // Wait for shell prompt
    println!("⏳ Waiting for shell prompt...");
    while start.elapsed() < timeout {
        match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                all_output.push_str(&chunk);
                if all_output.contains("# ") || all_output.contains("$ ") {
                    println!("✅ Shell prompt found");
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => break,
        }
    }

    if !all_output.contains("# ") && !all_output.contains("$ ") {
        let _ = child.kill();
        bail!("Shell prompt not found within timeout. Partial output:\n{all_output}");
    }

    all_output.clear();
    std::thread::sleep(Duration::from_millis(200));

    // Send the command
    println!("📤 Sending command: {cmd}");
    stdin.write_all(cmd.as_bytes())?;
    stdin.write_all(b"\n")?;
    stdin.flush()?;

    // Capture output until we see another prompt
    let cmd_start = Instant::now();
    let cmd_timeout = Duration::from_secs(u64::from((timeout_secs / 2).max(5)));

    while cmd_start.elapsed() < cmd_timeout {
        match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                all_output.push_str(&chunk);
                if all_output.lines().count() > 1
                    && (all_output.ends_with("# ")
                        || all_output.ends_with("$ ")
                        || all_output.contains("\n# ")
                        || all_output.contains("\n$ "))
                {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => break,
        }
    }

    // Cleanup
    let _ = child.kill();
    let _ = child.wait();

    let clean_output = clean_shell_output(&all_output, cmd);

    println!();
    println!("📥 Output:");
    println!("────────────────────────────────────────");
    println!("{clean_output}");
    println!("────────────────────────────────────────");

    Ok(clean_output)
}

/// Clean shell output by removing echoed command, prompts, and ANSI codes
fn clean_shell_output(output: &str, cmd: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();

    let mut result = Vec::new();
    let mut skip_first = true;

    for line in lines {
        if skip_first && line.contains(cmd) {
            skip_first = false;
            continue;
        }
        skip_first = false;

        if line.ends_with("# ") || line.ends_with("$ ") || line.trim() == "#" || line.trim() == "$"
        {
            continue;
        }

        let clean = strip_ansi(line);
        if !clean.is_empty() {
            result.push(clean);
        }
    }

    result.join("\n")
}

/// Strip ANSI escape codes from a string
fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            result.push(c);
        }
    }

    result
}
