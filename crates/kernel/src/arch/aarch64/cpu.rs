//! TEAM_222: Architecture-specific CPU instructions

/// Wait for interrupt (WFI/WFE).
///
/// Puts the CPU into a low-power state until an interrupt occurs.
#[inline]
pub fn wait_for_interrupt() {
    aarch64_cpu::asm::wfe();
}

/// Halt the CPU indefinitely.
///
/// This enters a loop of `wait_for_interrupt`.
pub fn halt() -> ! {
    loop {
        wait_for_interrupt();
    }
}
