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
pub mod boot; // TEAM_282: Boot abstraction layer
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

/// TEAM_282: Unified kernel entry point accepting BootInfo.
///
/// This is the target signature for all boot paths. Currently called by
/// the legacy entry points after they parse boot info.
///
/// Note: The caller must call `boot::set_boot_info()` before calling this
/// to make boot info available globally.
pub fn kernel_main_unified(boot_info: &boot::BootInfo) -> ! {
    // Stage 1: Early HAL - Console must be first for debug output
    los_hal::console::init();

    // TEAM_221: Initialize logger (Info level silences Debug/Trace)
    // TEAM_272: Enable Trace level in verbose builds to satisfy behavior tests
    #[cfg(feature = "verbose")]
    logger::init(log::LevelFilter::Trace);
    #[cfg(not(feature = "verbose"))]
    logger::init(log::LevelFilter::Info);

    crate::init::transition_to(crate::init::BootStage::EarlyHAL);

    // Initialize heap (required for alloc)
    crate::arch::init_heap();

    // Log boot protocol
    println!("[BOOT] Protocol: {:?}", boot_info.protocol);
    if boot_info.memory_map.len() > 0 {
        println!(
            "[BOOT] Memory: {} regions, {} MB usable",
            boot_info.memory_map.len(),
            boot_info.memory_map.total_usable() / (1024 * 1024)
        );
    }

    // Stage 2: Physical Memory Management
    crate::init::transition_to(crate::init::BootStage::MemoryMMU);
    crate::memory::init(boot_info);

    // TEAM_262: Initialize bootstrap task immediately after heap/memory
    let bootstrap = alloc::sync::Arc::new(crate::task::TaskControlBlock::new_bootstrap());
    unsafe {
        crate::task::set_current_task(bootstrap);
    }

    // Hand off to init sequence (never returns)
    crate::init::run()
}

/// TEAM_282: Legacy AArch64 entry point (wrapper).
#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    // AArch64 requires DTB parsing to get BootInfo if not using Limine
    // For now, AArch64 still uses DTB
    crate::arch::init_heap();
    crate::arch::init_boot_info();

    let boot_info =
        crate::boot::boot_info().expect("AArch64 must have BootInfo initialized from DTB");

    // Transition to unified main
    kernel_main_unified(boot_info)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    crate::arch::cpu::halt();
}
