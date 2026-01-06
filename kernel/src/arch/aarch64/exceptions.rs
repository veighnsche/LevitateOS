use core::arch::global_asm;

global_asm!(include_str!("asm/exceptions.S"));

/// TEAM_073: Handle synchronous exception from lower EL (userspace).
///
/// This dispatches SVC (syscall) exceptions to the syscall handler,
/// and handles other exceptions (faults, etc.) by killing the process.
#[unsafe(no_mangle)]
pub extern "C" fn handle_sync_lower_el(frame: *mut crate::arch::SyscallFrame) {
    // Read ESR to determine exception type
    use aarch64_cpu::registers::{ESR_EL1, Readable};
    let esr: u64 = ESR_EL1.get();

    if crate::syscall::is_svc_exception(esr) {
        // SVC exception - this is a syscall
        let frame = unsafe { &mut *frame };
        crate::syscall::syscall_dispatch(frame);
    } else {
        // Other exception from user mode - kill process
        use aarch64_cpu::registers::{ELR_EL1, FAR_EL1};
        let elr: u64 = ELR_EL1.get();
        let far: u64 = FAR_EL1.get(); // TEAM_212: Add faulting address for debugging

        let ec = crate::syscall::esr_exception_class(esr);
        crate::println!("\n*** USER EXCEPTION ***");
        crate::println!("Exception Class: 0x{:02x}", ec);
        crate::println!("ESR: 0x{:016x}", esr);
        crate::println!("ELR (instruction): 0x{:016x}", elr);
        crate::println!("FAR (fault addr):  0x{:016x}", far);

        // Decode common exception classes
        match ec {
            0b100000 | 0b100001 => crate::println!("Type: Instruction Abort"),
            0b100100 | 0b100101 => crate::println!("Type: Data Abort"),
            0b100010 => crate::println!("Type: PC Alignment Fault"),
            0b100110 => crate::println!("Type: SP Alignment Fault"),
            _ => crate::println!("Type: Unknown (EC=0x{:02x})", ec),
        }

        crate::println!("Terminating user process.\n");

        loop {
            aarch64_cpu::asm::wfi();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_sync_exception(esr: u64, elr: u64) {
    // raw prints to avoid core::fmt
    use core::fmt::Write;
    use los_hal::console;
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
#[unsafe(no_mangle)]
pub extern "C" fn handle_irq() {
    let gic = los_hal::gic::active_api();
    let irq = gic.acknowledge();

    if los_hal::gic::Gic::is_spurious(irq) {
        return;
    }

    if !los_hal::gic::dispatch(irq) {
        crate::println!("Unhandled IRQ: {}", irq);
    }

    gic.end_interrupt(irq);
}

pub fn init() {
    unsafe extern "C" {
        static vectors: u8;
    }
    use aarch64_cpu::registers::{VBAR_EL1, Writeable};
    let vectors_ptr = unsafe { &vectors as *const u8 as u64 };
    VBAR_EL1.set(vectors_ptr);
}
