//! x86_64 Hardware Abstraction Layer
//!
//! # Compartments
//!
//! The x86_64 HAL is organized into logical compartments:
//!
//! - **cpu/** - CPU structures (GDT, TSS, IDT, exceptions)
//! - **mem/** - Memory management (paging, MMU, frame allocator)
//! - **interrupts/** - Interrupt handling (APIC, IOAPIC, PIT)
//! - **io/** - I/O devices (serial, VGA, console)
//! - **boot/** - Boot protocols (Multiboot2)
//!
//! See the README.md in this directory for architecture diagrams.

// === Compartments ===
pub mod boot;
pub mod cpu;
pub mod interrupts;
pub mod io;
pub mod mem;

// === Re-exports for backward compatibility ===
// CPU
pub use cpu::exceptions;
pub use cpu::gdt;
pub use cpu::idt;
pub mod tss {
    pub use crate::x86_64::cpu::gdt::*;
}

// Memory
pub use mem::frame_alloc;
pub use mem::mmu;
pub use mem::paging;

// Interrupts
pub use interrupts::apic;
pub use interrupts::ioapic;
pub use interrupts::pit;

// I/O
pub use io::console;
pub use io::serial;
pub use io::vga;

// Boot - TEAM_316: Limine-only, no multiboot re-exports needed

/// TEAM_316: Initialize HAL for Limine boot (simplified, Unix philosophy).
///
/// Limine provides:
/// - Correct page tables with HHDM mapping
/// - Memory map
/// - Framebuffer (optional)
///
/// We just need to initialize our CPU structures and drivers.
pub fn init() {
    // 1. Initialize serial for early logging
    unsafe {
        core::arch::asm!("mov dx, 0x3f8", "mov al, 'd'", "out dx, al", out("ax") _, out("dx") _);
    }
    unsafe { console::WRITER.lock().init() };

    // 2. Initialize GDT, IDT and exceptions
    unsafe {
        core::arch::asm!("mov al, 'e'", "out dx, al", out("ax") _, out("dx") _);
    }
    unsafe { gdt::init() };
    idt::init();
    exceptions::init();

    // 3. Initialize legacy 8259 PIC
    // TEAM_319: APIC MMIO (0xFEE00000) is outside HHDM range, use legacy PIC instead.
    // This remaps IRQ0-7 to vectors 32-39 and unmasks IRQ0 (timer).
    unsafe {
        core::arch::asm!("mov al, 'f'", "out dx, al", out("ax") _, out("dx") _);
    }
    interrupts::init_pic();

    // 4. Initialize PIT timer (legacy mode, works without APIC)
    unsafe {
        core::arch::asm!("mov al, 'g'", "out dx, al", out("ax") _, out("dx") _);
    }
    pit::Pit::init(100); // 100Hz

    // Done
    unsafe {
        core::arch::asm!("mov al, 'h'", "out dx, al", out("ax") _, out("dx") _);
    }
}
