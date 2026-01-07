// TEAM_260: AArch64 interrupt control.
// Behaviors: [I1]-[I6] interrupt enable/disable/restore cycle
// TEAM_132: Migrate DAIF to aarch64-cpu

use aarch64_cpu::registers::{Readable, Writeable, DAIF};

/// [I1] Disables interrupts, [I2] returns previous state
#[inline(always)]
pub fn disable() -> u64 {
    let state = DAIF.get(); // [I2] capture prev state
    // SAFETY: daifset is a special immediate-only instruction not provided by aarch64-cpu
    unsafe { core::arch::asm!("msr daifset, #2") }; // [I1] disable
    state
}

/// [I7] Unconditionally enables interrupts
#[inline(always)]
pub unsafe fn enable() {
    // SAFETY: daifclr is a special immediate-only instruction not provided by aarch64-cpu
    unsafe { core::arch::asm!("msr daifclr, #2") };
}

/// [I3] Restores previous interrupt state
#[inline(always)]
pub fn restore(state: u64) {
    DAIF.set(state); // [I3] restore
}

/// [I4] Returns true when enabled, [I5] returns false when disabled
#[inline(always)]
pub fn is_enabled() -> bool {
    // IRQ is bit 7 (zero-indexed)
    (DAIF.get() & (1 << 7)) == 0 // [I4][I5] check enabled state
}
