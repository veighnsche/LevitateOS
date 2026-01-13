//! Shutdown Behavior Test
//!
//! TEAM_142: Tests graceful shutdown via shell command.
//! TEAM_476: Rewritten for Linux + OpenRC (tests poweroff command).

use anyhow::{bail, Context, Result};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::qemu::{Arch, QemuBuilder, QemuProfile};

/// Run the shutdown behavior test
pub fn run(arch: &str) -> Result<()> {
    println!("=== Shutdown Behavior Test (Linux + OpenRC) for {arch} ===\n");

    // Build Linux + OpenRC initramfs
    crate::builder::create_openrc_initramfs(arch)?;

    let arch_enum = Arch::try_from(arch)?;
    let profile = if arch == "x86_64" {
        QemuProfile::X86_64
    } else {
        QemuProfile::Default
    };

    // Kill any existing QEMU
    let _ = Command::new("pkill")
        .args(["-f", arch_enum.qemu_binary()])
        .status();

    let initrd_path = format!("target/initramfs/{}-openrc.cpio", arch);
    let builder = QemuBuilder::new(arch_enum, profile)
        .display_nographic()
        .linux_kernel()
        .initrd(&initrd_path);

    let base_cmd = builder.build()?;
    let args: Vec<_> = base_cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();

    println!("Starting QEMU...");
    let mut child = Command::new(arch_enum.qemu_binary())
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start QEMU")?;

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let mut stdout = child.stdout.take().expect("Failed to get stdout");

    // Set stdout to non-blocking
    use std::os::unix::io::AsRawFd;
    let fd = stdout.as_raw_fd();
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }

    let mut all_output = String::new();
    let mut buf = [0u8; 4096];

    // Wait for shell prompt
    println!("Waiting for shell prompt...");
    let start = Instant::now();
    let timeout = Duration::from_secs(30);

    while start.elapsed() < timeout {
        match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                all_output.push_str(&chunk);

                // Look for shell prompt (BusyBox ash prompt)
                if all_output.contains("~ #") || all_output.contains("/ #") {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => break,
        }
    }

    if !all_output.contains("#") {
        let _ = child.kill();
        let _ = child.wait();
        bail!("Shell prompt not found within timeout");
    }

    println!("Shell ready. Sending 'poweroff' command...");

    // Send poweroff command
    stdin.write_all(b"poweroff\n")?;
    stdin.flush()?;

    // Wait for QEMU to exit (poweroff should halt the VM)
    let shutdown_start = Instant::now();
    let shutdown_timeout = Duration::from_secs(15);

    while shutdown_start.elapsed() < shutdown_timeout {
        match child.try_wait() {
            Ok(Some(_status)) => {
                println!("QEMU exited after poweroff.");
                println!("\n✅ SUCCESS: Shutdown completed!");
                return Ok(());
            }
            Ok(None) => {
                // Still running, read any output
                if let Ok(n) = stdout.read(&mut buf) {
                    if n > 0 {
                        let chunk = String::from_utf8_lossy(&buf[..n]);
                        print!("{chunk}");
                    }
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                bail!("Error waiting for QEMU: {e}");
            }
        }
    }

    // Force kill if still running
    let _ = child.kill();
    let _ = child.wait();
    bail!("QEMU did not exit after poweroff command within timeout")
}
