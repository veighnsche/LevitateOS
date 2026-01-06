use core::arch::global_asm;
use levitate_hal::println;

// TEAM_073: Import syscall handling
use crate::syscall::{self, SyscallFrame};

// Exception Vector Table
// 16 entries, each 128 bytes (0x80)
// TEAM_073: Updated sync handlers to support syscall return
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

    /* Lower EL using AArch64 (userspace) */
    .balign 0x80
    b       sync_lower_el_entry // TEAM_073: Separate entry for lower EL (userspace) syscalls
    .balign 0x80
    b       irq_lower_el_entry  // TEAM_073: IRQ from userspace needs different handling
    .balign 0x80
    b       exception_hang      // FIQ
    .balign 0x80
    b       exception_hang      // SError

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

/* TEAM_073: Sync handler for lower EL (userspace) - supports returning to user */
/* TEAM_148: Added TTBR0 save/restore for yield during syscalls
 *
 * SyscallFrame layout (must match kernel/src/syscall.rs):
 *   Offset 0-240:   regs[31]  (x0-x30)
 *   Offset 248:     sp        (SP_EL0)
 *   Offset 256:     pc        (ELR_EL1) 
 *   Offset 264:     pstate    (SPSR_EL1)
 *   Offset 272:     ttbr0     (TTBR0_EL1)
 *   Total: 280 bytes
 */
sync_lower_el_entry:
    /* Save full user context to stack */
    sub     sp, sp, #280
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
    
    /* Save SP_EL0, ELR_EL1, SPSR_EL1, TTBR0_EL1 */
    mrs     x0, sp_el0
    str     x0, [sp, #248]
    mrs     x0, elr_el1
    str     x0, [sp, #256]
    mrs     x0, spsr_el1
    str     x0, [sp, #264]
    /* TEAM_148: Save TTBR0 for yield support */
    mrs     x0, ttbr0_el1
    str     x0, [sp, #272]
    
    /* Pass stack pointer as syscall frame */
    mov     x0, sp
    bl      handle_sync_lower_el
    
    /* TEAM_148: Restore TTBR0 first (before other system regs) */
    /* This ensures correct page tables if we yielded to another task */
    ldr     x0, [sp, #272]
    msr     ttbr0_el1, x0
    /* TLB invalidation: required since we don't use ASIDs */
    tlbi    vmalle1
    dsb     sy
    isb
    
    /* Restore SPSR and ELR */
    ldr     x0, [sp, #264]
    msr     spsr_el1, x0
    ldr     x0, [sp, #256]
    msr     elr_el1, x0
    ldr     x0, [sp, #248]
    msr     sp_el0, x0
    
    /* Restore user registers (x0 contains return value from syscall) */
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
    add     sp, sp, #280
    
    eret

/* TEAM_073: IRQ from lower EL - save user context and return */
irq_lower_el_entry:
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

    /* Hanging here for now (kernel exceptions are fatal) */
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

/// TEAM_073: Handle synchronous exception from lower EL (userspace).
///
/// This dispatches SVC (syscall) exceptions to the syscall handler,
/// and handles other exceptions (faults, etc.) by killing the process.
#[unsafe(no_mangle)]
pub extern "C" fn handle_sync_lower_el(frame: *mut SyscallFrame) {
    // Read ESR to determine exception type
    // TEAM_133: Migrate ESR_EL1 to aarch64-cpu
    use aarch64_cpu::registers::{ESR_EL1, Readable};
    let esr: u64 = ESR_EL1.get();

    if syscall::is_svc_exception(esr) {
        // SVC exception - this is a syscall
        let frame = unsafe { &mut *frame };
        syscall::syscall_dispatch(frame);
    } else {
        // Other exception from user mode - kill process
        // Per Phase 2 decision: Option A (print error and kill)
        // TEAM_133: Migrate ELR_EL1 to aarch64-cpu
        use aarch64_cpu::registers::ELR_EL1;
        let elr: u64 = ELR_EL1.get();

        let ec = syscall::esr_exception_class(esr);
        println!("\n*** USER EXCEPTION ***");
        println!("Exception Class: 0x{:02x}", ec);
        println!("ESR: 0x{:016x}", esr);
        println!("ELR (fault address): 0x{:016x}", elr);

        // Decode common exception classes
        match ec {
            0b100000 | 0b100001 => println!("Type: Instruction Abort"),
            0b100100 | 0b100101 => println!("Type: Data Abort"),
            0b100010 => println!("Type: PC Alignment Fault"),
            0b100110 => println!("Type: SP Alignment Fault"),
            _ => println!("Type: Unknown (EC=0x{:02x})", ec),
        }

        println!("Terminating user process.\n");

        // TODO(TEAM_073): Integrate with task/process management to properly terminate
        // For now, just loop (will be fixed in Step 5 integration)
        // TEAM_132: Migrate wfi to aarch64-cpu
        loop {
            #[cfg(target_arch = "aarch64")]
            aarch64_cpu::asm::wfi();
            #[cfg(not(target_arch = "aarch64"))]
            core::hint::spin_loop();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_sync_exception(esr: u64, elr: u64) {
    // raw prints to avoid core::fmt
    use core::fmt::Write;
    use levitate_hal::console;
    let _ = console::WRITER
        .lock()
        .write_str("\n*** KERNEL EXCEPTION: Synchronous ***\n");
    let _ = console::WRITER.lock().write_str("ESR: ");
    console::print_hex(esr);
    let _ = console::WRITER.lock().write_str("\nELR: ");
    console::print_hex(elr);
    let _ = console::WRITER.lock().write_str("\n");
}

/// Handle IRQs.
///
/// NOTE: Any drivers or shared data structures accessed here MUST use `IrqSafeLock`
/// or equivalent to prevent deadlocks when interrupted threads hold the same lock.
#[unsafe(no_mangle)]
pub extern "C" fn handle_irq() {
    // TEAM_045: Use detected active GIC API
    let gic = levitate_hal::gic::active_api();
    let irq = gic.acknowledge();

    // TEAM_017: Skip spurious interrupts (no EOI needed)
    if levitate_hal::gic::Gic::is_spurious(irq) {
        return;
    }

    // TEAM_015: Use gic::dispatch() instead of hardcoded IRQ numbers
    if !levitate_hal::gic::dispatch(irq) {
        println!("Unhandled IRQ: {}", irq);
    }

    gic.end_interrupt(irq);
}

pub fn init() {
    unsafe extern "C" {
        static vectors: u8;
    }
    // TEAM_133: Migrate VBAR_EL1 to aarch64-cpu
    use aarch64_cpu::registers::{VBAR_EL1, Writeable};
    let vectors_ptr = unsafe { &vectors as *const u8 as u64 };
    VBAR_EL1.set(vectors_ptr);
}
