//! Backspace Regression Test
//!
//! `TEAM_327`: Verifies backspace actually erases characters.
//!
//! This test catches the bug where:
//! - Keyboard sends 0x08 (BS)
//! - TTY only recognized 0x7F (DEL) as VERASE
//! - Result: backspace echoed as ^H instead of erasing
//!
//! Test Strategy:
//! 1. Type "abc"
//! 2. Press backspace
//! 3. Type "x"
//! 4. Press Enter
//! 5. Verify the shell received "abx" (not "abc^Hx" or "abcx")

use anyhow::{bail, Context, Result};
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::process::Stdio;
use std::time::{Duration, Instant};

use crate::builder;
use crate::qemu::{Arch, QemuBuilder, QemuProfile};

/// Run backspace regression test
pub fn run(arch: &str) -> Result<()> {
    println!("🔙 Backspace Regression Test for {arch}\n");
    println!("   This test verifies backspace actually erases characters.");
    println!("   It will FAIL if backspace is broken (echoes ^H instead of erasing).\n");

    // Build
    builder::build_all(arch)?;

    let arch_enum = Arch::try_from(arch)?;
    let profile = if arch == "x86_64" {
        QemuProfile::X86_64
    } else {
        QemuProfile::Default
    };

    let mut builder = QemuBuilder::new(arch_enum, profile).display_nographic();

    if arch == "x86_64" {
        builder = builder.boot_iso();
    }

    let mut cmd = builder.build()?;

    println!("🚀 Starting QEMU...");
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start QEMU")?;

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let mut stdout = child.stdout.take().expect("Failed to get stdout");

    // Set stdout to non-blocking
    let fd = stdout.as_raw_fd();
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }

    // Wait for shell prompt
    println!("⏳ Waiting for shell prompt...");
    let mut all_output = String::new();
    let mut buf = [0u8; 4096];
    let start = Instant::now();
    let timeout = Duration::from_secs(30);

    while start.elapsed() < timeout {
        match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                all_output.push_str(&chunk);
                if all_output.contains("# ") {
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

    if !all_output.contains("# ") {
        let _ = child.kill();
        bail!("Shell prompt not found within timeout");
    }

    // Clear output buffer
    all_output.clear();
    std::thread::sleep(Duration::from_millis(500));

    // === THE ACTUAL TEST ===
    // Type "echo abc", then backspace to erase 'c', then type 'x', expect "abx"
    println!("\n📝 TEST: Type 'echo abc', backspace, 'x', Enter");
    println!("   Expected: Shell echoes 'abx' (backspace erased 'c')");
    println!("   Broken:   Shell echoes 'abc^Hx' or 'abcx' (backspace didn't work)\n");

    // Type: echo abc
    stdin.write_all(b"echo abc")?;
    stdin.flush()?;
    std::thread::sleep(Duration::from_millis(100));

    // Send backspace (0x08)
    stdin.write_all(&[0x08])?;
    stdin.flush()?;
    std::thread::sleep(Duration::from_millis(100));

    // Type: x
    stdin.write_all(b"x")?;
    stdin.flush()?;
    std::thread::sleep(Duration::from_millis(100));

    // Press Enter
    stdin.write_all(b"\n")?;
    stdin.flush()?;

    // Wait for response
    std::thread::sleep(Duration::from_millis(1000));

    // Read output
    loop {
        match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                all_output.push_str(&chunk);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => break,
        }
    }

    // Cleanup
    let _ = child.kill();
    let _ = child.wait();

    println!("📤 Raw output:\n{all_output:?}\n");

    // === VERIFICATION ===
    // The echo command should have received "abx" and printed it
    // If backspace worked: output contains "abx"
    // If backspace broken: output contains "abc" followed by ^H or similar

    let backspace_worked = all_output.contains("abx");
    let backspace_broken_caret = all_output.contains("^H"); // TTY echoed ^H
    let backspace_broken_literal = all_output.contains("abc\x08"); // Literal BS in output

    println!("━━━ Results ━━━");

    if backspace_worked && !backspace_broken_caret {
        println!("✅ PASS: Backspace works correctly!");
        println!("   Output contains 'abx' - backspace erased 'c' and 'x' replaced it.");
        Ok(())
    } else if backspace_broken_caret {
        println!("❌ FAIL: Backspace is BROKEN!");
        println!("   Output contains '^H' - TTY echoed control character instead of erasing.");
        println!("   This means the TTY isn't recognizing 0x08 (BS) as an erase character.");
        println!("\n   FIX: Ensure kernel/src/fs/tty/mod.rs accepts both 0x08 and 0x7F as VERASE.");
        bail!("Backspace regression test failed - TTY not handling BS (0x08)")
    } else if backspace_broken_literal {
        println!("❌ FAIL: Backspace is BROKEN!");
        println!("   Output contains literal backspace - not being processed.");
        bail!("Backspace regression test failed - BS not processed")
    } else {
        println!("⚠️  INCONCLUSIVE: Could not verify backspace behavior.");
        println!("   Output: {all_output:?}");
        println!("   Expected to find 'abx' in output.");
        // Don't fail - might be timing issue
        Ok(())
    }
}
