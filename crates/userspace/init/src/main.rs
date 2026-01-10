//! TEAM_120: LevitateOS Init Process (PID 1)
//!
//! Responsible for starting the system and managing services.
//! Currently just starts the shell.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libsyscall::{common_panic_handler, println, spawn, spawn_args, yield_cpu};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

/// TEAM_378: Run basic coreutils tests to verify utilities work
/// Note: Eyra/uutils binaries require argv[0] (program name), so we use spawn_args
fn run_coreutils_tests() -> bool {
    println!("[COREUTILS_TEST] Starting coreutils verification...");
    let mut passed = 0;
    let mut failed = 0;

    // Test 1: 'true' should exit with code 0
    let argv: [&str; 1] = ["true"];
    let pid = spawn_args("/true", &argv);
    if pid > 0 {
        let mut status: i32 = -1;
        libsyscall::waitpid(pid as i32, Some(&mut status));
        if status == 0 {
            println!("[COREUTILS_TEST] true: PASS (exit=0)");
            passed += 1;
        } else {
            println!("[COREUTILS_TEST] true: FAIL (exit={})", status);
            failed += 1;
        }
    } else {
        println!("[COREUTILS_TEST] true: FAIL (spawn failed)");
        failed += 1;
    }

    // Test 2: 'false' should exit with code 1
    let argv: [&str; 1] = ["false"];
    let pid = spawn_args("/false", &argv);
    if pid > 0 {
        let mut status: i32 = -1;
        libsyscall::waitpid(pid as i32, Some(&mut status));
        if status == 1 {
            println!("[COREUTILS_TEST] false: PASS (exit=1)");
            passed += 1;
        } else {
            println!("[COREUTILS_TEST] false: FAIL (exit={})", status);
            failed += 1;
        }
    } else {
        println!("[COREUTILS_TEST] false: FAIL (spawn failed)");
        failed += 1;
    }

    // Test 3: 'pwd' should exit with code 0
    let argv: [&str; 1] = ["pwd"];
    let pid = spawn_args("/pwd", &argv);
    if pid > 0 {
        let mut status: i32 = -1;
        libsyscall::waitpid(pid as i32, Some(&mut status));
        if status == 0 {
            println!("[COREUTILS_TEST] pwd: PASS (exit=0)");
            passed += 1;
        } else {
            println!("[COREUTILS_TEST] pwd: FAIL (exit={})", status);
            failed += 1;
        }
    } else {
        println!("[COREUTILS_TEST] pwd: FAIL (spawn failed)");
        failed += 1;
    }

    // Test 4: 'echo hello' should exit with code 0
    let argv: [&str; 2] = ["echo", "hello"];
    let pid = spawn_args("/echo", &argv);
    if pid > 0 {
        let mut status: i32 = -1;
        libsyscall::waitpid(pid as i32, Some(&mut status));
        if status == 0 {
            println!("[COREUTILS_TEST] echo: PASS (exit=0)");
            passed += 1;
        } else {
            println!("[COREUTILS_TEST] echo: FAIL (exit={})", status);
            failed += 1;
        }
    } else {
        println!("[COREUTILS_TEST] echo: FAIL (spawn failed)");
        failed += 1;
    }

    println!("[COREUTILS_TEST] Summary: {}/{} passed", passed, passed + failed);
    failed == 0
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let pid = libsyscall::getpid();
    println!("[INIT] PID {} starting...", pid);

    // TEAM_374: Check for eyra-test-runner and run it if present (test mode)
    // This allows run-test.sh to automatically test all Eyra utilities
    let test_runner_pid = spawn("eyra-test-runner");
    if test_runner_pid > 0 {
        println!("[INIT] Test runner spawned as PID {}", test_runner_pid);
        // Wait for test runner to complete before spawning shell
        let mut status: i32 = 0;
        let wait_result = libsyscall::waitpid(test_runner_pid as i32, Some(&mut status));
        println!("[INIT] Test runner exited: wait={}, status={}", wait_result, status);

        // TEAM_378: Run coreutils tests after eyra-test-runner
        let coreutils_ok = run_coreutils_tests();
        if coreutils_ok {
            println!("[COREUTILS_TEST] RESULT: PASSED");
        } else {
            println!("[COREUTILS_TEST] RESULT: FAILED");
        }
    }

    // TEAM_404: Spawn brush shell (Eyra-based with std support)
    println!("[INIT] Spawning shell...");
    let shell_pid = spawn("brush");

    if shell_pid < 0 {
        println!("[INIT] ERROR: Failed to spawn shell: {}", shell_pid);
    } else {
        println!("[INIT] Shell spawned as PID {}", shell_pid);
    }

    // PID 1 must never exit
    // TEAM_129: Yield to allow shell to run
    loop {
        yield_cpu();
    }
}
