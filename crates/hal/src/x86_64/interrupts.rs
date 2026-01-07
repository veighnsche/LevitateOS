// TEAM_260: x86_64 interrupt control.
// TEAM_269: Added mock implementation for std feature (user-space tests can't use cli/sti)

// =============================================================================
// Real implementation for bare metal (no_std)
// =============================================================================

#[cfg(not(feature = "std"))]
mod real_impl {
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
}

// =============================================================================
// Mock implementation for std feature (user-space tests)
// =============================================================================

#[cfg(feature = "std")]
mod mock_impl {
    use core::sync::atomic::{AtomicU64, Ordering};

    // TEAM_269: Mock interrupt state for tests
    // Bit 9 (0x200) is the IF flag in x86 RFLAGS
    static MOCK_FLAGS: AtomicU64 = AtomicU64::new(0x200); // Start with interrupts enabled

    /// [I1] Disables interrupts, [I2] returns previous state
    #[inline(always)]
    pub fn disable() -> u64 {
        // Atomically: read current state, then clear IF bit
        // Returns the state BEFORE clearing (so caller can restore it)
        let prev = MOCK_FLAGS.fetch_and(!0x200, Ordering::SeqCst);
        prev
    }

    /// [I7] Unconditionally enables interrupts
    #[inline(always)]
    pub unsafe fn enable() {
        MOCK_FLAGS.fetch_or(0x200, Ordering::SeqCst);
    }

    /// [I3] Restores previous interrupt state
    #[inline(always)]
    pub fn restore(state: u64) {
        if (state & 0x200) != 0 {
            MOCK_FLAGS.fetch_or(0x200, Ordering::SeqCst);
        } else {
            MOCK_FLAGS.fetch_and(!0x200, Ordering::SeqCst);
        }
    }

    /// [I4] Returns true when enabled, [I5] returns false when disabled
    #[inline(always)]
    pub fn is_enabled() -> bool {
        (MOCK_FLAGS.load(Ordering::SeqCst) & 0x200) != 0
    }
}

// =============================================================================
// Public API - delegates to appropriate implementation
// =============================================================================

#[cfg(not(feature = "std"))]
pub use real_impl::*;

#[cfg(feature = "std")]
pub use mock_impl::*;
