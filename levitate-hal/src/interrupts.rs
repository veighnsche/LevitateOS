/// AArch64 interrupt control.
/// Behaviors: [I1]-[I6] interrupt enable/disable/restore cycle

/// [I1] Disables interrupts, [I2] returns previous state
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn disable() -> u64 {
    let state: u64;
    unsafe {
        core::arch::asm!("mrs {}, daif", out(reg) state); // [I2] capture prev state
        core::arch::asm!("msr daifset, #2");              // [I1] disable
    }
    state
}

#[cfg(not(target_arch = "aarch64"))]
#[cfg(feature = "std")]
mod mock {
    use std::cell::Cell;
    thread_local! {
        pub static ENABLED: Cell<bool> = Cell::new(true);
    }
}

/// [I1] Disables interrupts, [I2] returns previous state (mock impl)
#[cfg(not(target_arch = "aarch64"))]
#[inline(always)]
pub fn disable() -> u64 {
    #[cfg(feature = "std")]
    {
        let prev = is_enabled();                    // [I2] capture prev
        mock::ENABLED.with(|e| e.set(false));       // [I1] disable
        prev as u64
    }
    #[cfg(not(feature = "std"))]
    0 // Stub for no-std non-aarch64
}

/// [I3] Restores previous interrupt state (mock impl)
#[cfg(not(target_arch = "aarch64"))]
#[inline(always)]
pub fn restore(state: u64) {
    #[cfg(feature = "std")]
    mock::ENABLED.with(|e| e.set(state != 0));      // [I3] restore
}

/// [I3] Restores previous interrupt state
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn restore(state: u64) {
    unsafe {
        core::arch::asm!("msr daif, {}", in(reg) state); // [I3] restore
    }
}

/// [I4] Returns true when enabled, [I5] returns false when disabled
#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub fn is_enabled() -> bool {
    let state: u64;
    unsafe {
        core::arch::asm!("mrs {}, daif", out(reg) state);
    }
    // IRQ is bit 7 (zero-indexed)
    (state & (1 << 7)) == 0  // [I4][I5] check enabled state
}

/// [I4] Returns true when enabled, [I5] returns false when disabled (mock impl)
#[cfg(not(target_arch = "aarch64"))]
#[inline(always)]
pub fn is_enabled() -> bool {
    #[cfg(feature = "std")]
    return mock::ENABLED.with(|e| e.get());         // [I4][I5]
    #[cfg(not(feature = "std"))]
    true // Stub for no-std non-aarch64
}
