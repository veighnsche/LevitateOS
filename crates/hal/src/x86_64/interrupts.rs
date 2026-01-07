// TEAM_260: x86_64 interrupt control.

/// [I1] Disables interrupts, [I2] returns previous state
#[inline(always)]
pub fn disable() -> u64 {
    let flags: u64;
    unsafe {
        core::arch::asm!("pushfq; pop {}", out(reg) flags, options(nomem, nostack));
        core::arch::asm!("cli", options(nomem, nostack));
    }
    flags
}

/// [I7] Unconditionally enables interrupts
#[inline(always)]
pub unsafe fn enable() {
    unsafe { core::arch::asm!("sti", options(nomem, nostack)); }
}

/// [I3] Restores previous interrupt state
#[inline(always)]
pub fn restore(state: u64) {
    if (state & 0x200) != 0 {
        unsafe { enable(); }
    }
}

/// [I4] Returns true when enabled, [I5] returns false when disabled
#[inline(always)]
pub fn is_enabled() -> bool {
    let flags: u64;
    unsafe {
        core::arch::asm!("pushfq; pop {}", out(reg) flags, options(nomem, nostack));
    }
    (flags & 0x200) != 0
}
