use core::arch::global_asm;
use levitate_hal::timer::Timer;
use levitate_hal::{print, println};

// Exception Vector Table
// 16 entries, each 128 bytes (0x80)
global_asm!(
    r#"
.section ".text.vectors", "ax"
.global vectors
.balign 0x800

vectors:
    /* Current EL with SP_EL0 */
    .balign 0x80
    b       sync_handler_entry
    .balign 0x80
    b       exception_hang
    .balign 0x80
    b       exception_hang
    .balign 0x80
    b       exception_hang

    /* Current EL with SP_ELx */
    .balign 0x80
    b       sync_handler_entry  // Synchronous
    .balign 0x80
    b       irq_entry           // IRQ
    .balign 0x80
    b       exception_hang      // FIQ
    .balign 0x80
    b       exception_hang      // SError

    /* Lower EL using AArch64 */
    .balign 0x80
    b       sync_handler_entry
    .balign 0x80
    b       exception_hang
    .balign 0x80
    b       exception_hang
    .balign 0x80
    b       exception_hang

    /* Lower EL using AArch32 */
    .balign 0x80
    b       sync_handler_entry
    .balign 0x80
    b       exception_hang
    .balign 0x80
    b       exception_hang
    .balign 0x80
    b       exception_hang

exception_hang:
    wfe
    b       exception_hang

sync_handler_entry:
    /* Save context */
    sub     sp, sp, #256
    stp     x0, x1, [sp, #0]
    stp     x2, x3, [sp, #16]
    stp     x4, x5, [sp, #32]
    stp     x6, x7, [sp, #48]
    stp     x8, x9, [sp, #64]
    stp     x10, x11, [sp, #80]
    stp     x12, x13, [sp, #96]
    stp     x14, x15, [sp, #112]
    stp     x16, x17, [sp, #128]
    stp     x18, x19, [sp, #144]
    stp     x20, x21, [sp, #160]
    stp     x22, x23, [sp, #176]
    stp     x24, x25, [sp, #192]
    stp     x26, x27, [sp, #208]
    stp     x28, x29, [sp, #224]
    str     x30, [sp, #240]

    /* Pass ESR and ELR as arguments */
    mrs     x0, esr_el1
    mrs     x1, elr_el1
    bl      handle_sync_exception

    /* Hanging here for now */
    b       exception_hang

irq_entry:
    /* Save context */
    sub     sp, sp, #256
    stp     x0, x1, [sp, #0]
    stp     x2, x3, [sp, #16]
    stp     x4, x5, [sp, #32]
    stp     x6, x7, [sp, #48]
    stp     x8, x9, [sp, #64]
    stp     x10, x11, [sp, #80]
    stp     x12, x13, [sp, #96]
    stp     x14, x15, [sp, #112]
    stp     x16, x17, [sp, #128]
    stp     x18, x19, [sp, #144]
    stp     x20, x21, [sp, #160]
    stp     x22, x23, [sp, #176]
    stp     x24, x25, [sp, #192]
    stp     x26, x27, [sp, #208]
    stp     x28, x29, [sp, #224]
    str     x30, [sp, #240]

    /* Call Rust handler */
    bl      handle_irq

    /* Restore context */
    ldp     x0, x1, [sp, #0]
    ldp     x2, x3, [sp, #16]
    ldp     x4, x5, [sp, #32]
    ldp     x6, x7, [sp, #48]
    ldp     x8, x9, [sp, #64]
    ldp     x10, x11, [sp, #80]
    ldp     x12, x13, [sp, #96]
    ldp     x14, x15, [sp, #112]
    ldp     x16, x17, [sp, #128]
    ldp     x18, x19, [sp, #144]
    ldp     x20, x21, [sp, #160]
    ldp     x22, x23, [sp, #176]
    ldp     x24, x25, [sp, #192]
    ldp     x26, x27, [sp, #208]
    ldp     x28, x29, [sp, #224]
    ldr     x30, [sp, #240]
    add     sp, sp, #256

    eret
"#
);

#[unsafe(no_mangle)]
pub extern "C" fn handle_sync_exception(esr: u64, elr: u64) {
    println!("\n*** KERNEL EXCEPTION: Synchronous ***");
    print!("ESR: {:#x}\n", esr);
    print!("ELR: {:#x}\n", elr);
}

/// Handle IRQs.
///
/// NOTE: Any drivers or shared data structures accessed here MUST use `IrqSafeLock`
/// or equivalent to prevent deadlocks when interrupted threads hold the same lock.
#[unsafe(no_mangle)]
pub extern "C" fn handle_irq() {
    let irq = levitate_hal::gic::API.acknowledge();
    if irq == 27 {
        // Reload timer for next interrupt (1 second)
        let freq = levitate_hal::timer::API.read_frequency();
        levitate_hal::timer::API.set_timeout(freq);
    } else if irq == 33 {
        levitate_hal::console::handle_interrupt();
    } else if irq < 1020 {
        println!("IRQ Received: {}", irq);
    }
    levitate_hal::gic::API.end_interrupt(irq);
}
pub fn init() {
    unsafe extern "C" {
        static vectors: u8;
    }
    unsafe {
        let vectors_ptr = &vectors as *const u8 as u64;
        core::arch::asm!("msr vbar_el1, {}", in(reg) vectors_ptr);
    }
}
