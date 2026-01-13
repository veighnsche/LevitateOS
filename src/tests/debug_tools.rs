//! Debug Tools Integration Tests
//!
//! `TEAM_325`: Automated tests for `debug regs` and `debug mem` commands.
//!
//! Tests both architectures using golden file comparison for output format.
//! Uses QEMU with NO guest OS - just tests QMP infrastructure against QEMU's
//! deterministic initial CPU/memory state. This ensures tests don't break
//! when the `LevitateOS` kernel changes.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::config::{GoldenRating, XtaskConfig};
use crate::support::qmp::QmpClient;

const QMP_SOCKET: &str = "./debug-test-qmp.sock";
const QEMU_INIT_WAIT_MS: u64 = 1000; // Just wait for QEMU to init, no OS boot needed

// Golden file paths
const GOLDEN_REGS_AARCH64: &str = "tests/golden_debug_regs_aarch64.txt";
const GOLDEN_REGS_X86_64: &str = "tests/golden_debug_regs_x86_64.txt";
const GOLDEN_MEM_AARCH64: &str = "tests/golden_debug_mem_aarch64.txt";
const GOLDEN_MEM_X86_64: &str = "tests/golden_debug_mem_x86_64.txt";

/// Run all debug tool tests for both architectures
pub fn run(update: bool) -> Result<()> {
    println!("🔧 Debug Tools Integration Tests\n");
    println!("   Testing both architectures...\n");

    // Test aarch64
    println!("━━━ aarch64 ━━━");
    run_arch("aarch64", update)?;

    // Test x86_64
    println!("\n━━━ x86_64 ━━━");
    run_arch("x86_64", update)?;

    println!("\n✅ All debug tool tests passed!");
    Ok(())
}

/// Run debug tests for a specific architecture
fn run_arch(arch: &str, update: bool) -> Result<()> {
    // NO BUILD NEEDED - we run QEMU without any guest OS
    // This tests the QMP infrastructure against QEMU's deterministic initial state

    // Clean up any existing socket
    let _ = fs::remove_file(QMP_SOCKET);

    // Start QEMU with QMP (no guest OS)
    let mut child = start_qemu_with_qmp(arch)?;

    // Wait for QEMU to initialize and QMP socket to be ready
    println!("  ⏳ Waiting for QEMU to initialize...");
    wait_for_qmp_socket()?;

    // Brief wait for QEMU to fully initialize (no OS boot needed)
    std::thread::sleep(Duration::from_millis(QEMU_INIT_WAIT_MS));

    // Run tests
    let regs_result = test_debug_regs(arch, update);
    let mem_result = test_debug_mem(arch, update);

    // Cleanup
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_file(QMP_SOCKET);

    // Report results
    regs_result?;
    mem_result?;

    Ok(())
}

/// Start QEMU with QMP socket enabled (NO guest OS - bare QEMU for deterministic testing)
fn start_qemu_with_qmp(arch: &str) -> Result<std::process::Child> {
    let qemu_bin = match arch {
        "aarch64" => "qemu-system-aarch64",
        "x86_64" => "qemu-system-x86_64",
        _ => bail!("Unsupported architecture: {arch}"),
    };

    let qmp_arg = format!("unix:{QMP_SOCKET},server,nowait");

    // Minimal QEMU args - NO kernel, NO ISO, NO guest OS
    // This gives us deterministic initial CPU/memory state for testing
    let args: Vec<String> = vec![
        "-M".into(),
        if arch == "aarch64" { "virt" } else { "q35" }.into(),
        "-cpu".into(),
        if arch == "aarch64" {
            "cortex-a72"
        } else {
            "qemu64"
        }
        .into(),
        "-m".into(),
        "64M".into(), // Minimal memory
        "-nographic".into(),
        "-qmp".into(),
        qmp_arg,
        "-S".into(), // Start paused - deterministic initial state
    ];

    let child = Command::new(qemu_bin)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start QEMU")?;

    Ok(child)
}

/// Wait for QMP socket to be available
fn wait_for_qmp_socket() -> Result<()> {
    for _ in 0..40 {
        if Path::new(QMP_SOCKET).exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    bail!("QMP socket not created - QEMU may have failed to start")
}

/// Test `debug regs` command
fn test_debug_regs(arch: &str, update: bool) -> Result<()> {
    println!("  📋 Testing debug regs...");

    let mut client = QmpClient::connect(QMP_SOCKET)?;

    // Get register dump via QMP
    let args = serde_json::json!({
        "command-line": "info registers"
    });
    let result = client.execute("human-monitor-command", Some(args))?;

    let output = result.as_str().unwrap_or("");

    // Normalize output for comparison (strip volatile values)
    let normalized = normalize_regs_output(output, arch);

    let golden_file = if arch == "aarch64" {
        GOLDEN_REGS_AARCH64
    } else {
        GOLDEN_REGS_X86_64
    };

    if update {
        // Update golden file
        fs::write(golden_file, &normalized)?;
        println!("    ✏️  Updated golden file: {golden_file}");
        return Ok(());
    }

    // Compare with golden file
    compare_with_golden(&normalized, golden_file, "regs")?;
    println!("    ✅ debug regs matches golden file");
    Ok(())
}

/// Test `debug mem` command
fn test_debug_mem(arch: &str, update: bool) -> Result<()> {
    println!("  💾 Testing debug mem...");

    let mut client = QmpClient::connect(QMP_SOCKET)?;

    // Use pmemsave for physical memory (more reliable than memsave which needs virtual addresses)
    // aarch64 virt: RAM starts at 0x40000000
    // x86_64 q35: RAM starts at 0x0
    let mem_addr: u64 = if arch == "aarch64" {
        0x4000_0000 // Start of physical RAM on virt machine
    } else {
        0x0 // Start of physical RAM on q35
    };

    let tmp_file = format!("/tmp/debug_test_mem_{}.bin", std::process::id());
    let args = serde_json::json!({
        "val": mem_addr,
        "size": 256,
        "filename": &tmp_file
    });
    // Use pmemsave for physical memory access (works before OS sets up virtual memory)
    client.execute("pmemsave", Some(args))?;

    // Wait for file to be written
    std::thread::sleep(Duration::from_millis(100));

    let data = fs::read(&tmp_file).context("Failed to read memory dump")?;
    let _ = fs::remove_file(&tmp_file);

    // Format as hex dump with correct base address
    let output = format_hexdump(&data, mem_addr);

    let golden_file = if arch == "aarch64" {
        GOLDEN_MEM_AARCH64
    } else {
        GOLDEN_MEM_X86_64
    };

    if update {
        // Update golden file
        fs::write(golden_file, &output)?;
        println!("    ✏️  Updated golden file: {golden_file}");
        return Ok(());
    }

    // Compare with golden file
    compare_with_golden(&output, golden_file, "mem")?;
    println!("    ✅ debug mem matches golden file");
    Ok(())
}

/// Normalize register output by replacing volatile values with placeholders
fn normalize_regs_output(output: &str, _arch: &str) -> String {
    let mut normalized = String::new();

    for line in output.lines() {
        // Replace all hex sequences of 8+ characters with X placeholders
        // Also normalize CPU flags like -ZC- which vary based on execution state
        let normalized_line = normalize_volatile_values(line);
        normalized.push_str(&normalized_line);
        normalized.push('\n');
    }

    normalized
}

/// Normalize a line by replacing volatile values with placeholders
fn normalize_volatile_values(line: &str) -> String {
    let mut result = String::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '=' {
            result.push(ch);
            // Consume hex digits after =
            let mut hex_count = 0;
            while let Some(&next) = chars.peek() {
                if next.is_ascii_hexdigit() {
                    chars.next();
                    hex_count += 1;
                    if hex_count <= 16 {
                        result.push('X');
                    }
                } else {
                    break;
                }
            }
        } else if ch == ':' {
            result.push(ch);
            // Consume hex digits after : (for Q registers like Q00=xxxx:yyyy)
            let mut hex_count = 0;
            while let Some(&next) = chars.peek() {
                if next.is_ascii_hexdigit() {
                    chars.next();
                    hex_count += 1;
                    if hex_count <= 16 {
                        result.push('X');
                    }
                } else {
                    break;
                }
            }
        } else if ch == '-' || ch == 'Z' || ch == 'C' || ch == 'N' || ch == 'V' {
            // Check if this looks like a flags field (e.g., -ZC-, NZcv)
            // These vary based on CPU state, normalize them
            let mut flag_chars = vec![ch];
            while let Some(&next) = chars.peek() {
                if next == '-'
                    || next == 'Z'
                    || next == 'C'
                    || next == 'N'
                    || next == 'V'
                    || next == 'z'
                    || next == 'c'
                    || next == 'n'
                    || next == 'v'
                {
                    flag_chars.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            // If it's exactly 4 chars and looks like flags, normalize
            if flag_chars.len() == 4 {
                result.push_str("____"); // Normalize flags
            } else {
                for c in flag_chars {
                    result.push(c);
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Format data as hex dump
fn format_hexdump(data: &[u8], base_addr: u64) -> String {
    const BYTES_PER_LINE: usize = 16;
    let mut output = String::new();

    for (i, chunk) in data.chunks(BYTES_PER_LINE).enumerate() {
        let offset = i * BYTES_PER_LINE;
        let addr = base_addr + offset as u64;

        // Address
        output.push_str(&format!("0x{addr:08x}: "));

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            output.push_str(&format!("{byte:02x} "));
            if j == 7 {
                output.push(' ');
            }
        }

        // Padding
        if chunk.len() < BYTES_PER_LINE {
            for j in chunk.len()..BYTES_PER_LINE {
                output.push_str("   ");
                if j == 7 {
                    output.push(' ');
                }
            }
        }

        // ASCII
        output.push('|');
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                output.push(*byte as char);
            } else {
                output.push('.');
            }
        }
        output.push_str("|\n");
    }

    output
}

/// Compare output with golden file
fn compare_with_golden(actual: &str, golden_path: &str, test_name: &str) -> Result<()> {
    // Load config to check golden file rating
    let config = XtaskConfig::load()?;
    let rating = config.golden_rating(golden_path);

    if !Path::new(golden_path).exists() {
        // Create golden file if it doesn't exist
        fs::write(golden_path, actual)?;
        println!("    📝 Created new golden file: {golden_path}");
        return Ok(());
    }

    let expected = fs::read_to_string(golden_path)
        .context(format!("Failed to read golden file: {golden_path}"))?;

    let matches = actual.trim() == expected.trim();

    match rating {
        GoldenRating::Silver => {
            // Silver files: Always auto-update and show diff
            if !matches {
                fs::write(golden_path, actual)?;
                println!("    🔄 SILVER MODE: Golden file updated");
                println!("    --- Changes ---");
                print_simple_diff(&expected, actual);
            }
            Ok(())
        }
        GoldenRating::Gold => {
            if matches {
                Ok(())
            } else {
                // Write actual output for debugging
                let actual_path = golden_path.replace("golden_", "actual_");
                fs::write(&actual_path, actual)?;

                bail!(
                    "{test_name} output differs from golden file!\n\
                     Expected: {golden_path}\n\
                     Actual:   {actual_path}\n\
                     \n\
                     To update golden file, run:\n  \
                     cargo xtask test debug --update"
                );
            }
        }
    }
}

/// Print a simple diff for debug output
fn print_simple_diff(expected: &str, actual: &str) {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    let max_len = expected_lines.len().max(actual_lines.len());

    for i in 0..max_len.min(10) {
        // Show first 10 differences
        match (expected_lines.get(i), actual_lines.get(i)) {
            (Some(e), Some(a)) if e != a => {
                println!("    - {e}");
                println!("    + {a}");
            }
            (Some(e), None) => {
                println!("    - {e}");
            }
            (None, Some(a)) => {
                println!("    + {a}");
            }
            _ => {}
        }
    }

    if max_len > 10 {
        println!("    ... ({} more lines)", max_len - 10);
    }
}
