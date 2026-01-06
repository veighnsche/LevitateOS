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

// =============================================================================
// Kernel Modules
// =============================================================================

mod arch;
mod block;
mod fs;
mod gpu;
mod init;
mod input;
mod loader;
mod memory;
mod net;
mod syscall;
mod task;
mod terminal;
mod virtio;

// =============================================================================
// Verbose Macro (Feature-Gated)
// =============================================================================

/// Verbose print macro - only outputs when `verbose` feature is enabled.
/// Use for successful initialization messages (Rule 4: Silence is Golden).
/// Errors should always use println! directly.
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

// =============================================================================
// Kernel Entry Point
// =============================================================================

/// Main kernel entry point, called from assembly boot code.
///
/// This function performs minimal early initialization then delegates
/// to `init::run()` for the full boot sequence.
#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    // Stage 1: Early HAL - Console must be first for debug output
    los_hal::console::init();
    init::transition_to(init::BootStage::EarlyHAL);

    // Initialize heap (required for alloc)
    arch::init_heap();

    // Hand off to init sequence (never returns)
    init::run()
}

// =============================================================================
// Panic Handler
// =============================================================================

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop {
        #[cfg(target_arch = "aarch64")]
        aarch64_cpu::asm::wfe();
    }
}
