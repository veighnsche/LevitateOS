// Integration test for Eyra build process
// Tests the complete build pipeline from source to binary

use std::fs;
use std::path::Path;
use std::process::Command;

/// Helper to check if a command exists
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Tests: [EY5] cross-compilation uses correct sysroot
#[test]
#[ignore] // Expensive test - run manually with: cargo test --test eyra_integration_test -- --ignored
fn test_sysroot_configuration() {
    // [EY5] Verify sysroot is accessible
    let sysroot_path = "/usr/aarch64-redhat-linux/sys-root/fc43";

    if !Path::new(sysroot_path).exists() {
        eprintln!(
            "WARNING: [EY5] Sysroot not found at {}. Install with:",
            sysroot_path
        );
        eprintln!("  sudo dnf install sysroot-aarch64-fc43-glibc");
        // Don't fail the test - just warn
        return;
    }

    // Verify key files exist in sysroot
    assert!(
        Path::new(&format!("{}/lib", sysroot_path)).exists(),
        "[EY5] sysroot must have lib directory"
    );
    assert!(
        Path::new(&format!("{}/usr/lib", sysroot_path)).exists(),
        "[EY5] sysroot must have usr/lib directory"
    );
}

/// Tests: [EY11] libgcc_eh stub is created
#[test]
#[ignore]
fn test_libgcc_eh_stub_exists() {
    // [EY11] Build and verify stub creation
    if !command_exists("ar") {
        eprintln!("WARNING: [EY11] 'ar' command not found, skipping");
        return;
    }

    let output = Command::new("cargo")
        .args(&[
            "clean",
            "--manifest-path",
            "crates/userspace/eyra/libsyscall-tests/Cargo.toml",
        ])
        .output()
        .expect("Failed to run cargo clean");

    assert!(output.status.success(), "[EY11] cargo clean failed");

    let output = Command::new("cargo")
        .args(&[
            "build",
            "--release",
            "--target",
            "aarch64-unknown-linux-gnu",
            "--manifest-path",
            "crates/userspace/eyra/libsyscall-tests/Cargo.toml",
        ])
        .output()
        .expect("Failed to run cargo build");

    if !output.status.success() {
        eprintln!(
            "[EY11] Build output:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
}

    // Check that build succeeded (stub was created successfully)
    assert!(
        output.status.success(),
        "[EY11] Build must succeed with libgcc_eh stub"
    );
}

/// Tests: [EY12] getauxval stub is linked
#[test]
#[ignore]
fn test_getauxval_stub_linked() {
    // [EY12] Verify getauxval stub compiles and links
    if !command_exists("aarch64-linux-gnu-gcc") {
        eprintln!("WARNING: [EY12] aarch64-linux-gnu-gcc not found, skipping");
        return;
    }

    let output = Command::new("cargo")
        .args(&[
            "build",
            "--release",
            "--target",
            "aarch64-unknown-linux-gnu",
            "--manifest-path",
            "crates/userspace/eyra/libsyscall-tests/Cargo.toml",
        ])
        .output()
        .expect("Failed to run cargo build");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should not have "undefined reference to `getauxval`" error
    assert!(
        !stderr.contains("undefined reference to `getauxval`"),
        "[EY12] getauxval stub must be properly linked"
    );
}

/// Tests: [EY13] build succeeds without libgcc_eh errors
#[test]
#[ignore]
fn test_no_libgcc_eh_errors() {
    // [EY13] Verify no libgcc_eh errors during build
    let output = Command::new("cargo")
        .args(&[
            "build",
            "--release",
            "--target",
            "aarch64-unknown-linux-gnu",
            "--manifest-path",
            "crates/userspace/eyra/libsyscall-tests/Cargo.toml",
        ])
        .output()
        .expect("Failed to run cargo build");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stderr.contains("cannot find -lgcc_eh"),
        "[EY13] Must not have libgcc_eh linking errors"
    );
}

/// Tests: [EY15] binary has LOAD segments at expected addresses
#[test]
#[ignore]
fn test_load_segments_addresses() {
    // [EY15] Verify segment addresses
    let binary_path =
        "crates/userspace/eyra/target/aarch64-unknown-linux-gnu/release/libsyscall-tests";

    if !Path::new(binary_path).exists() {
        eprintln!("WARNING: [EY15] Binary not built, skipping");
        return;
    }

    if !command_exists("readelf") {
        eprintln!("WARNING: [EY15] readelf not found, skipping");
        return;
    }

    let output = Command::new("readelf")
        .args(&["-l", binary_path])
        .output()
        .expect("Failed to run readelf");

    let readelf_output = String::from_utf8_lossy(&output.stdout);

    // Verify text segment at 0x400000
    assert!(
        readelf_output.contains("0x0000000000400000"),
        "[EY15] Text segment must be at 0x400000"
);

    // Verify we have LOAD segments
    let load_count = readelf_output.matches("LOAD").count();
    assert!(
        load_count >= 2,
        "[EY15] Binary must have at least 2 LOAD segments (text + data)"
    );
}

/// Tests: [EY16] text segment has R-X permissions
#[test]
#[ignore]
fn test_text_segment_permissions() {
    // [EY16] Verify text segment permissions
    let binary_path =
        "crates/userspace/eyra/target/aarch64-unknown-linux-gnu/release/libsyscall-tests";

    if !Path::new(binary_path).exists() {
        return;
    }

    if !command_exists("readelf") {
        return;
    }

    let output = Command::new("readelf")
        .args(&["-l", binary_path])
        .output()
        .expect("Failed to run readelf");

    let readelf_output = String::from_utf8_lossy(&output.stdout);

    // Find first LOAD segment (text)
    let lines: Vec<&str> = readelf_output.lines().collect();
    let mut found_rx = false;

    for i in 0..lines.len() {
        if lines[i].contains("LOAD") && lines[i].contains("0x0000000000400000") {
            // Check next line for flags
            if i + 1 < lines.len() && lines[i + 1].contains("R E") {
                found_rx = true;
                break;
            }
        }
    }

    assert!(
        found_rx,
        "[EY16] Text segment must have R-X (R E) permissions"
    );
}

/// Tests: [EY17] data segment has RW permissions
#[test]
#[ignore]
fn test_data_segment_permissions() {
    // [EY17] Verify data segment permissions
    let binary_path =
        "crates/userspace/eyra/target/aarch64-unknown-linux-gnu/release/libsyscall-tests";

    if !Path::new(binary_path).exists() {
        return;
    }

    if !command_exists("readelf") {
        return;
    }

    let output = Command::new("readelf")
        .args(&["-l", binary_path])
        .output()
        .expect("Failed to run readelf");

    let readelf_output = String::from_utf8_lossy(&output.stdout);

    // Find second LOAD segment (data)
    let lines: Vec<&str> = readelf_output.lines().collect();
    let mut found_rw = false;
    let mut load_count = 0;

    for i in 0..lines.len() {
        if lines[i].contains("LOAD") {
            load_count += 1;
            if load_count == 2 {
                // Check next line for flags
                if i + 1 < lines.len() && lines[i + 1].contains("RW") {
                    found_rw = true;
                    break;
                }
            }
        }
    }

    assert!(found_rw, "[EY17] Data segment must have RW permissions");
}

/// Tests: [EY34] x86_64 build fails as expected
#[test]
#[ignore]
fn test_x86_64_build_fails_expected() {
    // [EY34] Verify x86_64 build fails (not supported)
    let output = Command::new("cargo")
        .args(&[
            "build",
            "--release",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--manifest-path",
            "crates/userspace/eyra/libsyscall-tests/Cargo.toml",
        ])
        .output()
        .expect("Failed to run cargo build");

    // Build should fail for x86_64 (uses Rust std instead of Eyra)
    assert!(
        !output.status.success(),
        "[EY34] x86_64 build is expected to fail (not supported)"
);

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should have linking errors related to std
    assert!(
        stderr.contains("undefined") || stderr.contains("error"),
        "[EY34] x86_64 build should fail with linking errors"
    );
}

/// Full integration test: build libsyscall-tests from scratch
#[test]
#[ignore]
fn test_full_build_pipeline() {
    println!("\n=== Full Eyra Build Pipeline Test ===\n");

    // Step 1: Clean
    println!("Step 1: Cleaning previous build...");
    let output = Command::new("cargo")
        .args(&[
            "clean",
            "--manifest-path",
            "crates/userspace/eyra/Cargo.toml",
        ])
        .output()
        .expect("Failed to run cargo clean");
    assert!(output.status.success(), "Clean failed");

    // Step 2: Build libsyscall
    println!("Step 2: Building libsyscall...");
    let output = Command::new("cargo")
        .args(&[
            "build",
            "--release",
            "--target",
            "aarch64-unknown-linux-gnu",
            "--manifest-path",
            "crates/userspace/eyra/libsyscall/Cargo.toml",
            "--features",
            "std",
        ])
        .output()
        .expect("Failed to build libsyscall");

    if !output.status.success() {
        eprintln!("Build failed:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "libsyscall build failed");

    // Step 3: Build libsyscall-tests
    println!("Step 3: Building libsyscall-tests...");
    let output = Command::new("cargo")
        .args(&[
            "build",
            "--release",
            "--target",
            "aarch64-unknown-linux-gnu",
            "--manifest-path",
            "crates/userspace/eyra/libsyscall-tests/Cargo.toml",
        ])
        .output()
        .expect("Failed to build libsyscall-tests");

    if !output.status.success() {
        eprintln!("Build failed:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "libsyscall-tests build failed");

    // Step 4: Verify binary exists and is correct
    println!("Step 4: Verifying binary...");
    let binary_path =
        "crates/userspace/eyra/target/aarch64-unknown-linux-gnu/release/libsyscall-tests";
    assert!(Path::new(binary_path).exists(), "Binary not found");

    let metadata = fs::metadata(binary_path).expect("Failed to get metadata");
    println!("  Binary size: {} bytes", metadata.len());
    assert!(metadata.len() > 1000, "Binary too small (probably empty)");
    assert!(metadata.len() < 100_000, "Binary too large (>100KB)");

    // Step 5: Verify ELF format
    println!("Step 5: Checking ELF format...");
    if command_exists("file") {
        let output = Command::new("file")
            .arg(binary_path)
            .output()
            .expect("Failed to run file");

        let file_output = String::from_utf8_lossy(&output.stdout);
        println!("  {}", file_output.trim());
        assert!(file_output.contains("ELF 64-bit"), "Not a 64-bit ELF");
        assert!(file_output.contains("ARM aarch64"), "Not aarch64");
        assert!(
            file_output.contains("statically linked"),
            "Not statically linked"
        );
}

    println!("\n✅ Full build pipeline completed successfully!\n");
}
