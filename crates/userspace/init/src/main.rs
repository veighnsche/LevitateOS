//! TEAM_120: LevitateOS Init Process (PID 1)
//!
//! Responsible for starting the system and managing services.
//! Currently just starts the shell.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libsyscall::{common_panic_handler, println, spawn, yield_cpu};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let pid = libsyscall::getpid();
    println!("[INIT] PID {} starting...", pid);

    // Spawn the shell
    println!("[INIT] Spawning shell...");
    let shell_pid = spawn("shell");

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
