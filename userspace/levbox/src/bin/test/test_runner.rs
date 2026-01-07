//! TEAM_243: OS Internal Test Runner
//!
//! Runs all *_test binaries sequentially and reports results.
//! Designed for AI agent verification - outputs parseable format to stdout.
//!
//! TEAM_244: Added set_foreground() for child processes so Ctrl+C works.
//! TEAM_244: Added interrupt_test with automated SIGINT delivery.

#![no_std]
#![no_main]

extern crate ulib;
use libsyscall::{
    getpid, kill, println, set_foreground, shutdown, shutdown_flags, spawn, waitpid, yield_cpu,
    SIGINT,
};

/// List of test binaries to run (must exist in initramfs)
const TESTS: &[&str] = &[
    "mmap_test",
    "pipe_test",
    "signal_test",
    "clone_test",
    "interrupt_test",
    "tty_test",
    "pty_test",
    "suite_test_core",
    "stat_test",
    "link_test",
    "time_test",
    "sched_yield_test",
    "error_test",
];

#[no_mangle]
pub fn main() -> i32 {
    println!("");
    println!("[TEST_RUNNER] ========================================");
    println!("[TEST_RUNNER] LevitateOS Internal Test Suite");
    println!("[TEST_RUNNER] ========================================");
    println!("[TEST_RUNNER] Test count: {}", TESTS.len());
    println!("");

    let mut passed = 0;
    let mut failed = 0;
    let mut results: [(bool, i32); 16] = [(false, 0); 16];

    for (i, test) in TESTS.iter().enumerate() {
        println!("[TEST_RUNNER] ----------------------------------------");
        println!(
            "[TEST_RUNNER] [{}/{}] Running: {}",
            i + 1,
            TESTS.len(),
            test
        );
        println!("[TEST_RUNNER] ----------------------------------------");

        let pid = spawn(test);
        if pid < 0 {
            println!("[TEST_RUNNER] {}: SPAWN_FAILED (error={})", test, pid);
            failed += 1;
            results[i] = (false, pid as i32);
            continue;
        }

        // TEAM_244: Set child as foreground so Ctrl+C signals the test
        set_foreground(pid as usize);

        // TEAM_244: Special handling for interrupt_test - send SIGINT after brief delay
        if *test == "interrupt_test" {
            println!("[TEST_RUNNER] (sending SIGINT to simulate Ctrl+C...)");
            // Yield a few times to let the child start and enter pause()
            for _ in 0..10 {
                yield_cpu();
            }
            // Send SIGINT to the child (simulating Ctrl+C)
            kill(pid as i32, SIGINT as i32);
        }

        // Wait for test to complete
        let mut status: i32 = -1;
        let wait_result = waitpid(pid as i32, Some(&mut status));

        // TEAM_244: Restore test_runner as foreground after child exits
        set_foreground(getpid() as usize);

        if wait_result < 0 {
            println!(
                "[TEST_RUNNER] {}: WAIT_FAILED (error={})",
                test, wait_result
            );
            failed += 1;
            results[i] = (false, wait_result as i32);
            continue;
        }

        if status == 0 {
            println!("[TEST_RUNNER] {}: PASS", test);
            passed += 1;
            results[i] = (true, 0);
        } else {
            println!("[TEST_RUNNER] {}: FAIL (exit={})", test, status);
            failed += 1;
            results[i] = (false, status);
        }
        println!("");
    }

    // Print summary
    println!("[TEST_RUNNER] ========================================");
    println!("[TEST_RUNNER] SUMMARY");
    println!("[TEST_RUNNER] ========================================");

    for (i, test) in TESTS.iter().enumerate() {
        let (pass, code) = results[i];
        if pass {
            println!("[TEST_RUNNER]   {}: PASS", test);
        } else {
            println!("[TEST_RUNNER]   {}: FAIL ({})", test, code);
        }
    }

    println!("[TEST_RUNNER] ----------------------------------------");
    println!(
        "[TEST_RUNNER] Total: {}/{} tests passed",
        passed,
        passed + failed
    );
    println!("[TEST_RUNNER] ========================================");

    if failed > 0 {
        println!("[TEST_RUNNER] RESULT: FAILED");
    } else {
        println!("[TEST_RUNNER] RESULT: PASSED");
    }

    // Shutdown the system after tests complete
    println!("[TEST_RUNNER] Shutting down...");
    shutdown(shutdown_flags::VERBOSE);
    // shutdown() never returns
}
