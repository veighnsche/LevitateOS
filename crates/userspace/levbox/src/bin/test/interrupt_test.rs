//! TEAM_244: Interrupt test - tests external SIGINT delivery (simulates Ctrl+C)
//!
//! This test:
//! 1. Registers a SIGINT handler
//! 2. Waits for an EXTERNAL signal (from parent/test_runner)
//! 3. Verifies the handler ran
//!
//! Unlike signal_test which sends a signal to itself, this test
//! validates that signals from other processes work correctly.

#![no_std]
#![no_main]

extern crate ulib;
use core::sync::atomic::{AtomicBool, Ordering};
use ulib::libsyscall;
use ulib::prelude::println;
use ulib::{pause, signal, Signal};

/// Flag set by signal handler to verify it ran
static HANDLER_RAN: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub fn main() -> i32 {
    println!("[interrupt_test] Starting external interrupt test...");
    println!("[interrupt_test] Registering SIGINT handler...");

    // Register handler for SIGINT (Ctrl+C)
    signal(Signal::SIGINT, Some(handle_sigint));

    println!("[interrupt_test] Waiting for external SIGINT (Ctrl+C simulation)...");
    
    // Wait for signal from parent process
    // pause() will return when a signal is delivered
    pause();

    // Check if handler ran
    if HANDLER_RAN.load(Ordering::Acquire) {
        println!("[interrupt_test] External interrupt received successfully!");
        println!("[interrupt_test] PASS");
        0
    } else {
        println!("[interrupt_test] ERROR: pause() returned but handler didn't run!");
        println!("[interrupt_test] FAIL");
        1
    }
}

extern "C" fn handle_sigint(sig: i32) {
    // Set flag to prove handler ran
    HANDLER_RAN.store(true, Ordering::Release);

    // Print confirmation (async-signal-safe via direct syscall)
    let msg = b"[interrupt_test] *** SIGINT received (sig=";
    let _ = libsyscall::write(1, msg);
    
    let mut buf = [0u8; 2];
    buf[0] = (sig as u8 / 10) + b'0';
    buf[1] = (sig as u8 % 10) + b'0';
    let _ = libsyscall::write(1, &buf);
    let _ = libsyscall::write(1, b") ***\n");
}
