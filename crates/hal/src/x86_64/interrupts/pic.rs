//! 8259 PIC (Programmable Interrupt Controller) for x86_64.
//! Reference: https://wiki.osdev.org/8259_PIC
//!
//! TEAM_319: Legacy PIC mode for timer interrupts.
//! Used because APIC MMIO addresses (0xFEE00000) are outside the 1GB HHDM range.

use core::arch::asm;

// PIC ports
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

// ICW1 flags
const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;

// ICW4 flags
const ICW4_8086: u8 = 0x01;

/// Initialize the 8259 PIC in cascade mode.
/// Remaps IRQ0-7 to vectors 32-39, IRQ8-15 to vectors 40-47.
/// Initially masks all IRQs, then unmasks only IRQ0 (timer).
pub fn init() {
    unsafe {
        // First, mask ALL IRQs on both PICs to prevent spurious interrupts during init
        asm!("out dx, al", in("al") 0xFFu8, in("dx") PIC1_DATA, options(nomem, nostack, preserves_flags));
        asm!("out dx, al", in("al") 0xFFu8, in("dx") PIC2_DATA, options(nomem, nostack, preserves_flags));

        // ICW1: Start initialization sequence (cascade mode)
        asm!("out dx, al", in("al") ICW1_INIT | ICW1_ICW4, in("dx") PIC1_COMMAND, options(nomem, nostack, preserves_flags));
        io_wait();
        asm!("out dx, al", in("al") ICW1_INIT | ICW1_ICW4, in("dx") PIC2_COMMAND, options(nomem, nostack, preserves_flags));
        io_wait();

        // ICW2: Vector offsets
        // Master PIC: IRQ0-7 → vectors 32-39
        asm!("out dx, al", in("al") 32u8, in("dx") PIC1_DATA, options(nomem, nostack, preserves_flags));
        io_wait();
        // Slave PIC: IRQ8-15 → vectors 40-47
        asm!("out dx, al", in("al") 40u8, in("dx") PIC2_DATA, options(nomem, nostack, preserves_flags));
        io_wait();

        // ICW3: Cascade identity
        // Master: slave on IRQ2 (bit 2 = 0x04)
        asm!("out dx, al", in("al") 0x04u8, in("dx") PIC1_DATA, options(nomem, nostack, preserves_flags));
        io_wait();
        // Slave: cascade identity = 2
        asm!("out dx, al", in("al") 0x02u8, in("dx") PIC2_DATA, options(nomem, nostack, preserves_flags));
        io_wait();

        // ICW4: 8086 mode
        asm!("out dx, al", in("al") ICW4_8086, in("dx") PIC1_DATA, options(nomem, nostack, preserves_flags));
        io_wait();
        asm!("out dx, al", in("al") ICW4_8086, in("dx") PIC2_DATA, options(nomem, nostack, preserves_flags));
        io_wait();

        // Set masks: ONLY unmask IRQ0 (timer) for now
        // All other IRQs stay masked to prevent GPFs from unhandled interrupts
        // Mask byte: bit=1 means masked/disabled, bit=0 means unmasked/enabled
        // IRQ0 = bit 0, so mask = 0xFE (all masked except IRQ0)
        asm!("out dx, al", in("al") 0xFEu8, in("dx") PIC1_DATA, options(nomem, nostack, preserves_flags));
        // Slave: mask all
        asm!("out dx, al", in("al") 0xFFu8, in("dx") PIC2_DATA, options(nomem, nostack, preserves_flags));
    }
}

/// Small I/O delay for PIC operations.
#[inline(always)]
unsafe fn io_wait() {
    // Write to unused port 0x80 for a small delay
    // Use dx register form for consistency
    asm!("out dx, al", in("al") 0u8, in("dx") 0x80u16, options(nomem, nostack, preserves_flags));
}

/// Send End of Interrupt to the master PIC.
#[inline]
pub fn send_eoi_master() {
    unsafe {
        asm!("out dx, al", in("al") 0x20u8, in("dx") PIC1_COMMAND, options(nomem, nostack));
    }
}

/// Send End of Interrupt to both PICs (for IRQ8-15).
#[inline]
pub fn send_eoi_slave() {
    unsafe {
        asm!("out dx, al", in("al") 0x20u8, in("dx") PIC2_COMMAND, options(nomem, nostack));
        asm!("out dx, al", in("al") 0x20u8, in("dx") PIC1_COMMAND, options(nomem, nostack));
    }
}
