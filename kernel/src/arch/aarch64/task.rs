/// TEAM_162: Saved CPU context for AArch64.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Context {
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64, // Frame Pointer
    pub lr: u64,  // Link Register (x30)
    pub sp: u64,  // Stack Pointer
}

/// TEAM_162: Enter user mode at the specified entry point.
pub unsafe fn enter_user_mode(entry_point: usize, user_sp: usize) -> ! {
    unsafe {
        core::arch::asm!(
            "msr elr_el1, {entry}",
            "msr spsr_el1, xzr",
            "msr sp_el0, {sp}",
            "mov x0, xzr", "mov x1, xzr", "mov x2, xzr", "mov x3, xzr",
            "mov x4, xzr", "mov x5, xzr", "mov x6, xzr", "mov x7, xzr",
            "mov x8, xzr", "mov x9, xzr", "mov x10, xzr", "mov x11, xzr",
            "mov x12, xzr", "mov x13, xzr", "mov x14, xzr", "mov x15, xzr",
            "mov x16, xzr", "mov x17, xzr", "mov x18, xzr", "mov x19, xzr",
            "mov x20, xzr", "mov x21, xzr", "mov x22, xzr", "mov x23, xzr",
            "mov x24, xzr", "mov x25, xzr", "mov x26, xzr", "mov x27, xzr",
            "mov x28, xzr", "mov x29, xzr", "mov x30, xzr",
            "eret",
            entry = in(reg) entry_point,
            sp = in(reg) user_sp,
            options(noreturn)
        );
    }
    #[allow(unreachable_code)]
    loop {
        core::hint::spin_loop();
    }
}

/// TEAM_162: Switch to a new user address space.
pub unsafe fn switch_mmu_config(config_phys: usize) {
    unsafe {
        los_hal::mmu::switch_ttbr0(config_phys);
    }
}

unsafe extern "C" {
    pub fn cpu_switch_to(old: *mut Context, new: *const Context);
    pub fn task_entry_trampoline();
}

core::arch::global_asm!(
    r#"
.global cpu_switch_to
cpu_switch_to:
    mov     x10, sp
    stp     x19, x20, [x0, #16 * 0]
    stp     x21, x22, [x0, #16 * 1]
    stp     x23, x24, [x0, #16 * 2]
    stp     x25, x26, [x0, #16 * 3]
    stp     x27, x28, [x0, #16 * 4]
    stp     x29, x30, [x0, #16 * 5]
    str     x10,      [x0, #16 * 6]

    ldp     x19, x20, [x1, #16 * 0]
    ldp     x21, x22, [x1, #16 * 1]
    ldp     x23, x24, [x1, #16 * 2]
    ldp     x25, x26, [x1, #16 * 3]
    ldp     x27, x28, [x1, #16 * 4]
    ldp     x29, x30, [x1, #16 * 5]
    ldr     x10,      [x1, #16 * 6]
    mov     sp, x10
    ret

.global task_entry_trampoline
task_entry_trampoline:
    bl      post_switch_hook
    mov     x0, #0
    blr     x19
    bl      task_exit
    b       .
"#
);
