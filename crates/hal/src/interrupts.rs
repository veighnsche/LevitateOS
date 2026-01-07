// TEAM_260: Generic interrupt control wrapper.
// Delegates to architecture-specific implementations.

#[cfg(target_arch = "aarch64")]
use crate::aarch64::interrupts as arch_interrupts;

#[cfg(target_arch = "x86_64")]
use crate::x86_64::interrupts as arch_interrupts;

/// [I1] Disables interrupts, [I2] returns previous state
#[inline(always)]
pub fn disable() -> u64 {
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        arch_interrupts::disable()
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        0
    }
}

/// [I7] Unconditionally enables interrupts
/// 
/// # Safety
/// This function can cause race conditions if not used carefully.
#[inline(always)]
pub unsafe fn enable() {
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        unsafe { arch_interrupts::enable(); }
    }
}

/// [I3] Restores previous interrupt state
#[inline(always)]
pub fn restore(state: u64) {
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        arch_interrupts::restore(state)
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        let _ = state;
    }
}

/// [I4] Returns true when enabled, [I5] returns false when disabled
#[inline(always)]
pub fn is_enabled() -> bool {
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        arch_interrupts::is_enabled()
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        true
    }
}
