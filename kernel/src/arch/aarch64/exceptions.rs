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

    if crate::arch::is_svc_exception(esr) {
        // SVC exception - this is a syscall
        let frame = unsafe { &mut *frame };
        crate::syscall::syscall_dispatch(frame);

        // TEAM_216: Check for signals before returning to EL0
        check_signals(frame);
    } else {
        // Other exception from user mode - kill process
        use aarch64_cpu::registers::{ELR_EL1, FAR_EL1};
        let elr: u64 = ELR_EL1.get();
        let far: u64 = FAR_EL1.get(); // TEAM_212: Add faulting address for debugging

        let ec = crate::arch::esr_exception_class(esr);
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
pub extern "C" fn handle_irq(frame: *mut crate::arch::SyscallFrame) {
    use los_hal::aarch64::gic;
    let gic_api = gic::active_api();
    let irq = gic_api.acknowledge();

    if gic::Gic::is_spurious(irq) {
        return;
    }

    if !gic::dispatch(irq) {
        crate::println!("Unhandled IRQ: {}", irq);
    }

    gic_api.end_interrupt(irq);

    // TEAM_216: If IRQ came from userspace, check for signals
    if !frame.is_null() {
        let frame = unsafe { &mut *frame };
        check_signals(frame);
    }
}

/// TEAM_216: Check for pending unmasked signals and deliver them.
pub fn check_signals(frame: &mut crate::arch::SyscallFrame) {
    use crate::task::current_task;
    use core::sync::atomic::Ordering;

    let task = current_task();
    let pending = task.pending_signals.load(Ordering::Acquire);
    let blocked = task.blocked_signals.load(Ordering::Acquire);
    let unmasked = pending & !blocked;

    if unmasked == 0 {
        return;
    }

    // Find the first unmasked signal (simple priority: 0 to 31)
    for sig in 0..32 {
        if unmasked & (1 << sig) != 0 {
            // Found a signal to deliver
            if deliver_signal(frame, sig as i32) {
                // Clear the signal (since we don't have queueing)
                task.pending_signals
                    .fetch_and(!(1 << sig), Ordering::Release);
                break;
            }
        }
    }
}

/// TEAM_216: Set up userspace stack for signal handler execution.
fn deliver_signal(frame: &mut crate::arch::SyscallFrame, sig: i32) -> bool {
    use crate::task::current_task;
    use core::sync::atomic::Ordering;

    let task = current_task();
    let handlers = task.signal_handlers.lock();
    let handler = handlers[sig as usize];

    if handler == 0 {
        // Default action: many signals terminate the process
        // SIGKILL (9) is always fatal. SIGCHLD (17) is ignored.
        use crate::syscall::signal::*;
        if sig == SIGKILL || (sig != SIGCHLD && sig != SIGCONT) {
            crate::println!("[SIGNAL] PID={} terminated by signal {}", task.id.0, sig);
            crate::task::terminate_with_signal(sig); // This never returns
        }
        return false;
    }

    // Prepare trampoline delivery
    let trampoline = task.signal_trampoline.load(Ordering::Acquire);
    if trampoline == 0 {
        return false; // Can't deliver without trampoline
    }

    // 1. Save current frame to user stack (Signal Frame)
    let user_sp = frame.sp;
    let sig_frame_size = core::mem::size_of::<crate::arch::SyscallFrame>();
    let new_user_sp = (user_sp - sig_frame_size as u64) & !15; // Align 16

    // Copy kernel's current frame to user stack
    let frame_slice = unsafe {
        core::slice::from_raw_parts(
            (frame as *const crate::arch::SyscallFrame) as *const u8,
            sig_frame_size,
        )
    };

    for (i, &byte) in frame_slice.iter().enumerate() {
        if !crate::syscall::write_to_user_buf(task.ttbr0, new_user_sp as usize, i, byte) {
            crate::println!(
                "[SIGNAL] PID={} ERROR: Failed to write signal frame to user stack",
                task.id.0
            );
            return false;
        }
    }

    // 2. Redirect user execution to handler
    frame.sp = new_user_sp;
    frame.pc = handler as u64;
    frame.regs[0] = sig as u64; // arg0 = signal number
    frame.regs[30] = trampoline as u64; // LR = trampoline

    true
}

pub fn init() {
    unsafe extern "C" {
        static vectors: u8;
    }
    use aarch64_cpu::registers::{VBAR_EL1, Writeable};
    let vectors_ptr = unsafe { &vectors as *const u8 as u64 };
    VBAR_EL1.set(vectors_ptr);
}
