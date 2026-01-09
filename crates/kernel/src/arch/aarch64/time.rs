//! TEAM_222: Architecture-specific time management

/// Read the timer counter.
///
/// Returns the current value of the system timer (CNTVCT_EL0).
#[inline]
pub fn read_timer_counter() -> u64 {
    let cnt: u64;
    unsafe {
        core::arch::asm!("mrs {}, CNTVCT_EL0", out(reg) cnt);
    }
    cnt
}

/// Read the timer frequency.
///
/// Returns the frequency of the system timer in Hz (CNTFRQ_EL0).
#[inline]
pub fn read_timer_frequency() -> u64 {
    let freq: u64;
    unsafe {
        core::arch::asm!("mrs {}, CNTFRQ_EL0", out(reg) freq);
    }
    freq
}
