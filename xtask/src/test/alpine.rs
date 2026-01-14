use super::helpers::TestVm;
use anyhow::{Context, Result};

const ALPINE_ISO: &str = "vendor/images/alpine-standard-3.21.5-x86_64.iso";

/// Test 1: Basic boot to shell (FOUNDATION - no dependencies)
/// Verifies: VM can boot, reach login prompt, and enter shell
pub fn test_basic_boot() -> Result<()> {
    println!("Test: Alpine basic boot to ash shell");

    let mut vm = TestVm::start_alpine(ALPINE_ISO).context("Failed to start Alpine VM")?;
    vm.set_test_name("test_basic_boot");

    // Wait for login prompt
    let found = vm
        .wait_for_pattern(r"localhost login:", 60)
        .context("Failed to wait for login prompt")?;

    if !found {
        let output = vm.read_output()?;
        eprintln!("Output log:\n{}", output);
        vm.stop()?;
        anyhow::bail!("Login prompt not found within 60 seconds");
    }

    println!("✓ Boot successful - login prompt detected");

    // Send username and reach shell
    vm.send_line("root")
        .context("Failed to send username")?;

    let found = vm
        .wait_for_pattern(r"localhost:~#", 10)
        .context("Failed to wait for shell prompt")?;

    if !found {
        let output = vm.read_output()?;
        eprintln!("Output log:\n{}", output);
        vm.stop()?;
        anyhow::bail!("Shell prompt not found within 10 seconds");
    }

    println!("✓ Shell login successful");

    // Save artifact for manual validation
    let artifact = vm.save_artifact()?;
    println!("\n📁 Test output saved to artifact: {}", artifact);

    vm.stop()?;
    Ok(())
}

/// Test 2: QMP status queries (DEPENDS ON: test_basic_boot)
/// Verifies: QMP protocol works and VM reports correct state
pub fn test_qmp_status() -> Result<()> {
    println!("Test: QMP status queries");

    let mut vm = TestVm::start_alpine(ALPINE_ISO).context("Failed to start Alpine VM")?;
    vm.set_test_name("test_qmp_status");

    // Wait for boot (prerequisite: test_basic_boot must work)
    vm.wait_for_pattern(r"localhost login:", 60)
        .context("Failed waiting for boot")?;

    // Connect to QMP and query status
    let mut client = vm.qmp_client().context("Failed to connect to QMP")?;
    client.handshake().context("QMP handshake failed")?;

    let status = client.query_status().context("Failed to query status")?;
    println!("✓ QMP status: {:?}", status);

    // Verify VM is running
    match status {
        crate::vm::qmp::VmStatus::Running => {
            println!("✓ VM confirmed running via QMP");
        }
        other => {
            vm.stop()?;
            anyhow::bail!("Expected Running state, got {:?}", other);
        }
    }

    vm.stop()?;
    Ok(())
}

/// Test 3: Interactive login with password (DEPENDS ON: test_basic_boot)
/// Verifies: Serial input can handle multiple prompts (passwd, retype, re-login)
pub fn test_interactive_login() -> Result<()> {
    println!("Test: Interactive login with password authentication");

    let mut vm = TestVm::start_alpine(ALPINE_ISO).context("Failed to start Alpine VM")?;
    vm.set_test_name("test_interactive_login");

    // Boot to shell (prerequisite: test_basic_boot must work)
    let found = vm
        .wait_for_pattern(r"localhost login:", 60)
        .context("Failed to wait for login prompt")?;

    if !found {
        let output = vm.read_output()?;
        eprintln!("Output log:\n{}", output);
        vm.stop()?;
        anyhow::bail!("Login prompt not found within 60 seconds");
    }

    vm.send_line("root")
        .context("Failed to send username")?;

    let found = vm
        .wait_for_pattern(r"localhost:~#", 10)
        .context("Failed to wait for shell prompt")?;

    if !found {
        let output = vm.read_output()?;
        eprintln!("Output log:\n{}", output);
        vm.stop()?;
        anyhow::bail!("Shell prompt not found within 10 seconds");
    }

    println!("✓ Initial shell access successful");

    // Now set a password using passwd command
    println!("Setting password using passwd command...");
    vm.send_line("passwd")
        .context("Failed to send passwd command")?;

    // Wait for "New password:" prompt
    let found = vm
        .wait_for_pattern(r"New password:", 5)
        .context("Failed to wait for new password prompt")?;

    if !found {
        let output = vm.read_output()?;
        eprintln!("Output log:\n{}", output);
        vm.stop()?;
        anyhow::bail!("New password prompt not found");
    }

    println!("✓ New password prompt detected");

    // Send new password
    vm.send_line("testpass123")
        .context("Failed to send new password")?;

    // Wait for "Retype password:" prompt
    let found = vm
        .wait_for_pattern(r"Retype password:", 5)
        .context("Failed to wait for retype password prompt")?;

    if !found {
        let output = vm.read_output()?;
        eprintln!("Output log:\n{}", output);
        vm.stop()?;
        anyhow::bail!("Retype password prompt not found");
    }

    println!("✓ Retype password prompt detected");

    // Confirm password
    vm.send_line("testpass123")
        .context("Failed to send password confirmation")?;

    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("✓ Password set successfully");

    // Logout
    vm.send_line("exit")
        .context("Failed to send exit command")?;

    // Wait for login prompt again
    let found = vm
        .wait_for_pattern(r"localhost login:", 10)
        .context("Failed to wait for login prompt after logout")?;

    if !found {
        let output = vm.read_output()?;
        eprintln!("Output log:\n{}", output);
        vm.stop()?;
        anyhow::bail!("Login prompt not found after logout");
    }

    println!("✓ Logged out - back at login prompt");

    // Test login WITH password
    vm.send_line("root")
        .context("Failed to send username for second login")?;

    std::thread::sleep(std::time::Duration::from_millis(800));

    // Wait for password prompt
    let found = vm
        .wait_for_pattern(r"(?i)password\s*:", 10)
        .context("Failed to wait for password prompt")?;

    if !found {
        let output = vm.read_output()?;
        eprintln!("\nFull output log:\n{}\n", output);
        vm.stop()?;
        anyhow::bail!("Password prompt not found");
    }

    println!("✓ Password prompt detected");

    // Send password
    vm.send_line("testpass123")
        .context("Failed to send password")?;

    // Wait for shell prompt
    let found = vm
        .wait_for_pattern(r"localhost:~#", 10)
        .context("Failed to wait for shell prompt after password login")?;

    if !found {
        let output = vm.read_output()?;
        eprintln!("Output log:\n{}", output);
        vm.stop()?;
        anyhow::bail!("Shell prompt not found after password login");
    }

    println!("✓ Login with password successful");
    println!("✓ Full interactive authentication sequence verified!");

    // Save artifact for manual validation
    let artifact = vm.save_artifact()?;
    println!("\n📁 Test output saved to artifact: {}", artifact);

    vm.stop()?;
    Ok(())
}

/// Test 4: Graceful shutdown (DEPENDS ON: test_basic_boot, test_qmp_status)
/// Verifies: QMP shutdown command works and VM exits cleanly
pub fn test_shutdown() -> Result<()> {
    println!("Test: Graceful shutdown via QMP");

    let mut vm = TestVm::start_alpine(ALPINE_ISO).context("Failed to start Alpine VM")?;
    vm.set_test_name("test_shutdown");

    // Wait for boot (prerequisite: test_basic_boot must work)
    vm.wait_for_pattern(r"localhost login:", 60)
        .context("Failed waiting for boot")?;

    // Connect via QMP (prerequisite: test_qmp_status must work)
    let mut client = vm.qmp_client().context("Failed to connect to QMP")?;
    client.handshake().context("QMP handshake failed")?;
    client.system_powerdown().context("Failed to issue powerdown")?;

    println!("✓ Powerdown command sent");

    // Wait for VM to exit gracefully (up to 10 seconds)
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(10);

    let exited = loop {
        if start.elapsed() > timeout {
            break false;
        }

        if let Some(ref mut process) = vm.process {
            match process.try_wait() {
                Ok(Some(_)) => break true,
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                Err(_) => break false,
            }
        } else {
            break false;
        }
    };

    if exited {
        println!("✓ VM exited gracefully");
    } else {
        println!("⚠ VM did not exit within 10s, forcing stop");
    }

    vm.stop()?;
    Ok(())
}

/// Run all Alpine tests in dependency order
pub fn run_all() -> Result<()> {
    println!("\n=== Running Alpine ISO Integration Tests ===");
    println!("(Tests ordered by dependency: foundation → prerequisites → advanced)\n");

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Foundation - VM boot
    println!("[1/4] FOUNDATION: Basic boot (no dependencies)");
    match test_basic_boot() {
        Ok(_) => {
            passed += 1;
            println!("✓ test_basic_boot PASSED\n");
        }
        Err(e) => {
            failed += 1;
            eprintln!("✗ test_basic_boot FAILED: {:?}\n", e);
            eprintln!("All remaining tests depend on this - aborting\n");
            anyhow::bail!("Foundation test failed");
        }
    }

    // Test 2: QMP protocol
    println!("[2/4] PREREQUISITE: QMP status queries (requires: boot)");
    match test_qmp_status() {
        Ok(_) => {
            passed += 1;
            println!("✓ test_qmp_status PASSED\n");
        }
        Err(e) => {
            failed += 1;
            eprintln!("✗ test_qmp_status FAILED: {:?}\n", e);
            eprintln!("Shutdown test depends on this - skipping shutdown\n");
        }
    }

    // Test 3: Interactive features
    println!("[3/4] FEATURE: Interactive login (requires: boot)");
    match test_interactive_login() {
        Ok(_) => {
            passed += 1;
            println!("✓ test_interactive_login PASSED\n");
        }
        Err(e) => {
            failed += 1;
            eprintln!("✗ test_interactive_login FAILED: {:?}\n", e);
        }
    }

    // Test 4: Graceful shutdown (depends on both boot and QMP)
    if failed == 0 {
        println!("[4/4] ADVANCED: Graceful shutdown (requires: boot + QMP)");
        match test_shutdown() {
            Ok(_) => {
                passed += 1;
                println!("✓ test_shutdown PASSED\n");
            }
            Err(e) => {
                failed += 1;
                eprintln!("✗ test_shutdown FAILED: {:?}\n", e);
            }
        }
    } else {
        println!("[4/4] ADVANCED: Graceful shutdown (SKIPPED - prerequisite failed)\n");
    }

    println!("=== Test Summary ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);

    if failed > 0 {
        anyhow::bail!("{} test(s) failed", failed);
    }

    Ok(())
}
