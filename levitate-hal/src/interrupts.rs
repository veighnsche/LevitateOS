/// AArch64 interrupt control.

#[inline(always)]
pub fn disable() -> u64 {
    let state: u64;
    unsafe {
        core::arch::asm!("mrs {}, daif", out(reg) state);
        core::arch::asm!("msr daifset, #2");
    }
    state
}

#[inline(always)]
pub fn restore(state: u64) {
    unsafe {
        core::arch::asm!("msr daif, {}", in(reg) state);
    }
}

#[inline(always)]
pub fn is_enabled() -> bool {
    let state: u64;
    unsafe {
        core::arch::asm!("mrs {}, daif", out(reg) state);
    }
    // IRQ is bit 7 (zero-indexed)
    (state & (1 << 7)) == 0
}
