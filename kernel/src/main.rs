//! TEAM_146: Kernel Entry Point
//!
//! This is the minimal kernel entry point. The actual work is split into:
//! - `boot.rs` - Architecture-specific boot code (rarely changes)
//! - `init.rs` - Device discovery and initialization (changes often)
//!
//! This separation improves upgradability by isolating stable boot code
//! from frequently-modified initialization logic.

#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;
use los_hal::println;

#[cfg(feature = "verbose")]
#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => { $crate::println!($($arg)*) };
}

#[cfg(not(feature = "verbose"))]
#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => {};
}

pub mod arch;
pub mod block;
pub mod fs;
pub mod gpu;
pub mod init;
pub mod input;
pub mod loader;
pub mod logger;
pub mod memory;
pub mod net;
pub mod syscall;
pub mod task;
pub mod terminal;
pub mod virtio;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    // Stage 1: Early HAL - Console must be first for debug output
    los_hal::console::init();

    // TEAM_221: Initialize logger (Info level silences Debug/Trace)
    logger::init(log::LevelFilter::Info);

    crate::init::transition_to(crate::init::BootStage::EarlyHAL);

    // Initialize heap (required for alloc)
    crate::arch::init_heap();

    // --- Stage 1: CPU & Basic Initialization ---
    crate::arch::print_boot_regs();

    // TEAM_262: Initialize bootstrap task immediately after heap
    // This satisfies current_task() calls (e.g. from early IRQs)
    let bootstrap = alloc::sync::Arc::new(crate::task::TaskControlBlock::new_bootstrap());
    unsafe {
        crate::task::set_current_task(bootstrap);
    }

    // Hand off to init sequence (never returns)
    crate::init::run()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    crate::arch::cpu::halt();
}
