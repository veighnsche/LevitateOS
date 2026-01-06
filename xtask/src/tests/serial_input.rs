//! Serial Input Test
//!
//! TEAM_139: Automated test for verifying serial console input works.
//! Starts QEMU with -nographic, pipes input, verifies echo.

use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Test serial input by piping to QEMU and checking echo
pub fn run() -> Result<()> {
    println!("🔌 Testing serial input...\n");

    // First build everything
    crate::build::build_all()?;
    crate::image::create_disk_image_if_missing()?;

    // Clean QMP socket
    let _ = std::fs::remove_file("./qmp.sock");

    let kernel_bin = "kernel64_rust.bin";
    let args = vec![
        "-M", "virt",
        "-cpu", "cortex-a72",
        "-m", "1G",
        "-kernel", kernel_bin,
        "-nographic",
        "-device", "virtio-gpu-pci,xres=1280,yres=800",
        "-device", "virtio-keyboard-device",
        "-device", "virtio-tablet-device",
        "-device", "virtio-net-device,netdev=net0",
        "-netdev", "user,id=net0",
        "-drive", "file=tinyos_disk.img,format=raw,if=none,id=hd0",
        "-device", "virtio-blk-device,drive=hd0",
        "-initrd", "initramfs.cpio",
        "-serial", "mon:stdio",
        "-qmp", "unix:./qmp.sock,server,nowait",
        "-no-reboot",
    ];

    println!("  Starting QEMU in nographic mode...");
    let mut child = Command::new("qemu-system-aarch64")
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
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

    // Accumulate output
    let mut all_output = String::new();
    let mut buf = [0u8; 4096];

    // Wait for shell to be ready (look for "# " prompt or "help" text)
    println!("  Waiting for shell prompt...");
    let start = Instant::now();
    let timeout = Duration::from_secs(30);

    while start.elapsed() < timeout {
        match stdout.read(&mut buf) {
            Ok(0) => break, // EOF
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                print!("{}", chunk);  // Echo to our stdout
                all_output.push_str(&chunk);
                
                // Check if we have the shell prompt
                if all_output.contains("# ") || all_output.ends_with("# ") {
                    println!("\n  [DETECTED] Shell prompt found!");
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                eprintln!("  Read error: {}", e);
                break;
            }
        }
    }

    if !all_output.contains("# ") {
        let _ = child.kill();
        anyhow::bail!("Shell prompt '# ' not found within timeout.\nOutput:\n{}", all_output);
    }

    // Send test command
    let test_input = "echo SERIAL_TEST_MARKER_12345\n";
    println!("\n  Sending: {:?}", test_input.trim());
    stdin.write_all(test_input.as_bytes())?;
    stdin.flush()?;

    // Wait for echo
    println!("  Waiting for echo...");
    let echo_start = Instant::now();
    let echo_timeout = Duration::from_secs(10);

    while echo_start.elapsed() < echo_timeout {
        match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                print!("{}", chunk);
                all_output.push_str(&chunk);
                
                if all_output.contains("SERIAL_TEST_MARKER_12345") {
                    println!("\n  [DETECTED] Marker found in output!");
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
    println!("\n  Killing QEMU...");
    let _ = child.kill();
    let _ = child.wait();

    if all_output.contains("SERIAL_TEST_MARKER_12345") {
        println!("\n✅ Serial input test PASSED - input was echoed back!");
        Ok(())
    } else {
        println!("\n❌ Serial input test FAILED");
        println!("  Full output:\n{}", all_output);
        anyhow::bail!("Serial input test FAILED - marker not found in output")
    }
}

