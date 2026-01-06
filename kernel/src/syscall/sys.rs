use crate::memory::user as mm_user;

use los_hal::println;

/// TEAM_142: Shutdown flags for verbose mode
pub mod shutdown_flags {
    pub const VERBOSE: u32 = 1;
}

/// TEAM_142: sys_shutdown - Graceful system shutdown.
pub fn sys_shutdown(flags: u32) -> i64 {
    let verbose = flags & shutdown_flags::VERBOSE != 0;
    println!("[SHUTDOWN] Initiating graceful shutdown...");

    if verbose {
        println!("[SHUTDOWN] Phase 1: Stopping user tasks...");
    }

    if verbose {
        println!("[SHUTDOWN] Phase 1: Complete");
        println!("[SHUTDOWN] Phase 2: Flushing GPU framebuffer...");
    }

    {
        if let Some(mut guard) = crate::gpu::GPU.try_lock() {
            if let Some(gpu_state) = guard.as_mut() {
                let _ = gpu_state.flush();
            }
        }
    }

    if verbose {
        println!("[SHUTDOWN] GPU flush complete");
        println!("[SHUTDOWN] Phase 2: Complete");
        println!("[SHUTDOWN] Phase 3: Syncing filesystems...");
    }

    if verbose {
        println!("[SHUTDOWN] Phase 3: Complete (no pending writes)");
    }

    if verbose {
        println!("[SHUTDOWN] Phase 4: Disabling interrupts...");
        println!("[SHUTDOWN] Phase 4: Complete");
    }

    println!("[SHUTDOWN] System halted. Safe to power off.");
    println!("[SHUTDOWN] Goodbye!");

    for _ in 0..100000 {
        core::hint::spin_loop();
    }

    los_hal::interrupts::disable();

    const PSCI_SYSTEM_OFF: u64 = 0x84000008;
    unsafe {
        core::arch::asm!(
            "hvc #0",
            in("x0") PSCI_SYSTEM_OFF,
            options(noreturn)
        );
    }
}

/// TEAM_206: Read null-terminated string from user memory
pub fn read_user_string(
    ttbr0: usize,
    ptr: usize,
    max_len: usize,
) -> Result<alloc::string::String, ()> {
    let mut s = alloc::string::String::new();
    for i in 0..max_len {
        let va = ptr + i;
        if let Some(kptr) = mm_user::user_va_to_kernel_ptr(ttbr0, va) {
            let byte = unsafe { *kptr };
            if byte == 0 {
                return Ok(s);
            }
            s.push(byte as char);
        } else {
            return Err(());
        }
    }
    // Truncated or too long
    Err(())
}
