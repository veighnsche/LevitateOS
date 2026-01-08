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
pub fn kernel_main_unified(boot_info: &crate::boot::BootInfo) -> ! {
    // TEAM_305: Diagnostic 'R' for Rust Unified Entry (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    // SAFETY: Writing to serial port 0x3f8 is a standard debugging technique
    // in early x86_64 boot and is safe in this context.
    unsafe {
        core::arch::asm!("mov dx, 0x3f8", "mov al, 'R'", "out dx, al");
    }

    // TEAM_285: Initialize dynamic PHYS_OFFSET for Limine HHDM
    #[cfg(target_arch = "x86_64")]
    {
        if boot_info.protocol == crate::boot::BootProtocol::Limine {
            // Diagnostic 'i' for Limine
            unsafe {
                core::arch::asm!("mov al, 'i'", "out dx, al");
            }
            if let Some(offset) = crate::boot::limine::hhdm_offset() {
                los_hal::mmu::set_phys_offset(offset as usize);
            }
        } else {
            // Diagnostic 'j' for Non-Limine
            unsafe {
                core::arch::asm!("mov al, 'j'", "out dx, al");
            }
        }
    }

    // Stage 1: Early HAL - Console must be first for debug output
    // TEAM_284: Initialize arch-specific HAL early
    // TEAM_286: For Limine boot, skip CR3 switch - Limine's page tables are already correct
    #[cfg(target_arch = "x86_64")]
    {
        // Diagnostic 'k' before HAL Init
        unsafe {
            core::arch::asm!("mov al, 'k'", "out dx, al");
        }
        let switch_cr3 = boot_info.protocol != crate::boot::BootProtocol::Limine;
        los_hal::arch::init_with_options(switch_cr3);
    }
    #[cfg(not(target_arch = "x86_64"))]
    los_hal::arch::init();

    crate::init::transition_to(crate::init::BootStage::EarlyHAL);
    los_hal::console::init();

    // TEAM_221: Initialize logger (Info level silences Debug/Trace)
    // TEAM_272: Enable Trace level in verbose builds to satisfy behavior tests
    #[cfg(feature = "verbose")]
    logger::init(log::LevelFilter::Trace);
    #[cfg(not(feature = "verbose"))]
    logger::init(log::LevelFilter::Info);

    // Initialize heap (required for alloc)
    crate::arch::init_heap();

    // Log boot protocol
    log::info!("[BOOT] Protocol: {:?}", boot_info.protocol);
    if !boot_info.memory_map.is_empty() {
        log::info!(
            "[BOOT] Memory: {} regions, {} MB usable",
            boot_info.memory_map.len(),
            boot_info.memory_map.total_usable() / (1024 * 1024)
        );
    }

    // Stage 2: Physical Memory Management
    crate::init::transition_to(crate::init::BootStage::MemoryMMU);

    // TEAM_284: x86_64 needs PMO expansion before memory init for higher-half consistency
    #[cfg(target_arch = "x86_64")]
    {
        unsafe extern "C" {
            static mut early_pml4: los_hal::arch::paging::PageTable;
        }

        let mut ram_regions: [Option<los_hal::arch::multiboot2::MemoryRegion>; 16] = [None; 16];
        let mut count = 0;
        for region in boot_info.memory_map.iter() {
            if region.kind == crate::boot::MemoryKind::Usable && count < 16 {
                ram_regions[count] = Some(los_hal::arch::multiboot2::MemoryRegion {
                    start: region.base,
                    end: region.base + region.size,
                    typ: los_hal::arch::multiboot2::MemoryType::Available,
                });
                count += 1;
            }
        }

        // SAFETY: early_pml4 is the initial page table defined in assembly.
        // It is safe to expand it here before the memory manager is initialized
        // as we are still in single-core boot mode.
        unsafe {
            los_hal::arch::mmu::expand_pmo(&mut *core::ptr::addr_of_mut!(early_pml4), &ram_regions);
        }
    }

    crate::memory::init(boot_info);

    // TEAM_299: Initialize x86_64 CPU state (PCR, GS base)
    #[cfg(target_arch = "x86_64")]
    // SAFETY: Initializing CPU-specific registers (GS base, etc.) is required
    // for correct kernel operation and is safe during early boot.
    unsafe {
        crate::arch::cpu::init();
    }

    // TEAM_284: Initialize x86_64 syscalls after memory/heap
    #[cfg(target_arch = "x86_64")]
    // SAFETY: Initializing MSRs for syscall handling is a privileged but
    // necessary operation during kernel startup.
    unsafe {
        crate::arch::syscall::init();
    }

    // TEAM_262: Initialize bootstrap task immediately after heap/memory
    let bootstrap = alloc::sync::Arc::new(crate::task::TaskControlBlock::new_bootstrap());
    // SAFETY: Setting the initial task is required for the scheduler to function.
    // This is safe as it's the first task being set during boot.
    unsafe {
        crate::task::set_current_task(bootstrap);
    }

    // Hand off to init sequence (never returns)
    crate::init::run()
}

/// TEAM_282: Legacy AArch64 entry point (wrapper).
#[cfg(target_arch = "aarch64")]
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
