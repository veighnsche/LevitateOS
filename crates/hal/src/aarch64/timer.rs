use bitflags::bitflags;

bitflags! {
    /// Control register bits for the AArch64 generic timer (CNTP_CTL_EL0 / CNTV_CTL_EL0).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TimerCtrlFlags: u64 {
        /// Timer enabled.
        const ENABLE = 1 << 0;
        /// Timer interrupt masked.
        const IMASK  = 1 << 1;
        /// Timer interrupt status (read-only).
        const ISTATUS = 1 << 2;
    }
}

/// A generic interface for a hardware timer.
pub trait Timer {
    /// Read the current system counter value (CNTPCT_EL0).
    fn read_counter(&self) -> u64;

    /// Read the system counter frequency (CNTFRQ_EL0).
    fn read_frequency(&self) -> u64;

    /// Set the timer value for a one-shot interrupt (CNTP_TVAL_EL0).
    /// The timer will decrement this value and fire when it reaches zero.
    fn set_timeout(&self, ticks: u64);

    /// Configure the timer control register (CNTP_CTL_EL0).
    fn configure(&self, flags: TimerCtrlFlags);

    /// Convenience: Enable the timer and unmask its interrupt.
    fn enable(&self) {
        self.configure(TimerCtrlFlags::ENABLE);
    }

    /// Convenience: Disable the timer or mask its interrupt.
    fn disable(&self) {
        self.configure(TimerCtrlFlags::IMASK);
    }

    /// Check if the timer interrupt is pending.
    fn is_pending(&self) -> bool;

    /// Clear the timer interrupt by temporarily masking it.
    /// TEAM_017: Matches Redox explicit IRQ clearing pattern.
    fn clear_irq(&self) {
        if self.is_pending() {
            self.configure(TimerCtrlFlags::ENABLE | TimerCtrlFlags::IMASK);
        }
    }
}

use core::sync::atomic::{AtomicU8, Ordering};

// TEAM_046: Cache VHE detection result to avoid repeated system register reads
// 0 = not checked, 1 = no VHE, 2 = VHE present
static VHE_CACHE: AtomicU8 = AtomicU8::new(0);

/// Check if Virtualization Host Extensions (VHE) are present.
/// [T2] Reads ID_AA64MMFR1_EL1 to detect VHE support.
/// Result is cached after first call for performance.
pub fn vhe_present() -> bool {
    match VHE_CACHE.load(Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => {
            // First call - detect and cache
            let result = detect_vhe();
            VHE_CACHE.store(if result { 2 } else { 1 }, Ordering::Relaxed);
            result
        }
    }
}

/// Actually detect VHE by reading system register (called once).
/// TEAM_132: Migrate to aarch64-cpu
fn detect_vhe() -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        use aarch64_cpu::registers::{Readable, ID_AA64MMFR1_EL1};
        let mmfr1 = ID_AA64MMFR1_EL1.get();
        ((mmfr1 >> 8) & 0xF) != 0
    }
    #[cfg(not(target_arch = "aarch64"))]
    false
}

/// Implementation of the AArch64 Generic Timer.
/// Automatically selects between physical and virtual timer based on VHE presence.
pub struct AArch64Timer;

/// TEAM_132: Migrate timer registers to aarch64-cpu
#[cfg(target_arch = "aarch64")]
impl Timer for AArch64Timer {
    fn read_counter(&self) -> u64 {
        use aarch64_cpu::registers::{Readable, CNTPCT_EL0, CNTVCT_EL0};
        if vhe_present() {
            CNTPCT_EL0.get()
        } else {
            CNTVCT_EL0.get()
        }
    }

    fn read_frequency(&self) -> u64 {
        use aarch64_cpu::registers::{Readable, CNTFRQ_EL0};
        CNTFRQ_EL0.get()
    }

    fn set_timeout(&self, ticks: u64) {
        use aarch64_cpu::registers::{Writeable, CNTP_TVAL_EL0, CNTV_TVAL_EL0};
        if vhe_present() {
            CNTP_TVAL_EL0.set(ticks);
        } else {
            CNTV_TVAL_EL0.set(ticks);
        }
    }

    fn configure(&self, flags: TimerCtrlFlags) {
        use aarch64_cpu::registers::{Writeable, CNTP_CTL_EL0, CNTV_CTL_EL0};
        if vhe_present() {
            CNTP_CTL_EL0.set(flags.bits());
        } else {
            CNTV_CTL_EL0.set(flags.bits());
        }
    }

    fn is_pending(&self) -> bool {
        use aarch64_cpu::registers::{Readable, CNTP_CTL_EL0, CNTV_CTL_EL0};
        let val = if vhe_present() {
            CNTP_CTL_EL0.get()
        } else {
            CNTV_CTL_EL0.get()
        };
        TimerCtrlFlags::from_bits_truncate(val).contains(TimerCtrlFlags::ISTATUS)
    }
}

#[cfg(not(target_arch = "aarch64"))]
impl Timer for AArch64Timer {
    fn read_counter(&self) -> u64 {
        0
    }
    fn read_frequency(&self) -> u64 {
        1
    } // Avoid div by zero
    fn set_timeout(&self, _ticks: u64) {}
    fn configure(&self, _flags: TimerCtrlFlags) {}
    fn is_pending(&self) -> bool {
        false
    }
}

/// Global instance of the AArch64 physical timer.
pub static API: AArch64Timer = AArch64Timer;

/// [T1] Returns the uptime in seconds (counter / frequency)
pub fn uptime_seconds() -> u64 {
    let cnt = API.read_counter();
    let freq = API.read_frequency();
    if freq == 0 { 0 } else { cnt / freq } // [T1]
}

/// Spin-wait for a certain number of cycles.
pub fn delay_cycles(cycles: u64) {
    let start = API.read_counter();
    while API.read_counter() - start < cycles {
        core::hint::spin_loop();
    }
}

// Run tests with: cargo test -p levitate-hal --features std
#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    struct MockTimer {
        counter: core::cell::Cell<u64>,
        frequency: u64,
    }

    impl Timer for MockTimer {
        fn read_counter(&self) -> u64 {
            self.counter.get()
        }

        fn read_frequency(&self) -> u64 {
            self.frequency
        }

        fn set_timeout(&self, _ticks: u64) {}
        fn configure(&self, _flags: TimerCtrlFlags) {}
        fn is_pending(&self) -> bool {
            false
        }
    }

    /// Tests: [T1] uptime_seconds = counter / frequency
    #[test]
    fn test_uptime_seconds() {
        let timer = MockTimer {
            counter: core::cell::Cell::new(1000),
            frequency: 100,
        };
        // [T1] 1000 ticks / 100 ticks per second = 10 seconds
        assert_eq!(timer.read_counter() / timer.read_frequency(), 10);

        timer.counter.set(250);
        assert_eq!(timer.read_counter() / timer.read_frequency(), 2); // [T1]
    }
}
