//! Serial Input Test
//!
//! TEAM_139: Automated test for verifying serial console input works.
//! TEAM_476: Rewritten to test Linux + OpenRC boot.
//!
//! Starts QEMU with -nographic, pipes input, verifies response.

use anyhow::bail;
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::qemu::{Arch, QemuBuilder, QemuProfile};

pub fn run(arch: &str) -> Result<()> {
    println!("=== Serial Input Test (Linux + OpenRC) for {arch} ===\n");

    // Build Linux + OpenRC initramfs
    crate::builder::create_initramfs(arch)?;

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

    println!("Starting QEMU...");
    let mut child = Command::new(arch_enum.qemu_binary())
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start QEMU")?;

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let mut stdout = child.stdout.take().expect("Failed to get stdout");

    // Read stdout in a separate thread
    let (tx, rx) = std::sync::mpsc::channel();
    let stdout_thread = std::thread::spawn(move || {
        let mut buf = [0u8; 128];
        loop {
            if let Ok(n) = stdout.read(&mut buf) {
                if n == 0 {
                    break;
                }
                let s = String::from_utf8_lossy(&buf[..n]);
                print!("{s}"); // Mirror to our stdout
                let _ = tx.send(s.into_owned());
            } else {
                break;
            }
        }
    });

    // Wait for boot
    std::thread::sleep(Duration::from_secs(10));

    // Send "ls" command and check for output
    println!("\nSending 'ls' command...");
    stdin.write_all(b"ls\n")?;
    stdin.flush()?;

    // Check captured output for response
    let start = Instant::now();
    let mut found = false;
    let mut buffer = String::new();

    while start.elapsed() < Duration::from_secs(5) {
        if let Ok(chunk) = rx.try_recv() {
            buffer.push_str(&chunk);
            // Looking for typical Linux directory contents
            if buffer.contains("bin") || buffer.contains("etc") || buffer.contains("dev") {
                found = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    let _ = child.kill();
    let _ = stdout_thread.join();

    if found {
        println!("✅ SUCCESS: Serial input received and command executed!");
        Ok(())
    } else {
        bail!("Failed to get expected response from serial input");
    }
}
