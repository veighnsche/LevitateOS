#![no_std]
#![no_main]

//! TEAM_244: Signal handling test
//!
//! Tests that:
//! 1. Signal handlers can be registered
//! 2. kill() delivers signals synchronously to self
//! 3. Signal handler executes correctly

extern crate ulib;
use core::sync::atomic::{AtomicBool, Ordering};
use ulib::libsyscall;
use ulib::prelude::println;
use ulib::{kill, signal, Signal};

/// Flag set by signal handler to verify it ran
static HANDLER_RAN: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub fn main() -> i32 {
    println!("Signal test starting...");

    // Register a handler for SIGINT (2)
    signal(Signal::SIGINT, Some(handle_sigint));

    println!("Registered SIGINT handler. Sending SIGINT to self...");

    // Send SIGINT to self - handler should run before kill() returns
    let pid = libsyscall::getpid() as i32;
    kill(pid, Signal::SIGINT);

    println!("Signal sent. Checking if handler ran...");

    // TEAM_244: Verify handler actually ran (don't use pause() - signal already delivered)
    if HANDLER_RAN.load(Ordering::Acquire) {
        println!("Signal handler executed successfully!");
        println!("Signal test complete.");
        0
    } else {
        println!("ERROR: Signal handler did not run!");
        1
    }
}

extern "C" fn handle_sigint(sig: i32) {
    // TEAM_244: Set flag to prove handler ran
    HANDLER_RAN.store(true, Ordering::Release);

    // We are in a signal handler!
    // Using direct syscall write for async-signal-safety
    let msg = b"*** HANDLER: Received signal ";
    let _ = libsyscall::write(1, msg);

    // Simple hex print for signal number
    let mut buf = [0u8; 2];
    buf[0] = (sig as u8 / 10) + b'0';
    buf[1] = (sig as u8 % 10) + b'0';
    let _ = libsyscall::write(1, &buf);
    let _ = libsyscall::write(1, b" ***\n");
}
