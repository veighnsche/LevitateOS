#![cfg_attr(not(feature = "std"), no_std)]

// TEAM_260: HAL Crate - Symmetrical Architecture Support
// Root contains generic traits and utilities.
// Arch-specific logic is isolated in aarch64/ and x86_64/ submodules.

pub mod allocator;
pub mod console;
pub mod traits;
pub mod interrupts;
pub mod mmu;
pub mod memory; // TEAM_051: Frame allocator
pub mod virtio;

pub use traits::{InterruptController, MmuInterface, InterruptHandler, IrqId};

#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use self::aarch64 as arch;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use self::x86_64 as arch;

// Re-export common arch-specific modules to root for backward compatibility
#[cfg(target_arch = "aarch64")]
pub use self::arch::{gic, timer, fdt, serial};
#[cfg(target_arch = "x86_64")]
pub use self::arch::{apic, pit, vga, idt, exceptions, ioapic, serial};

/// Get the active interrupt controller.
pub fn active_interrupt_controller() -> &'static dyn InterruptController {
    #[cfg(target_arch = "aarch64")]
    {
        arch::gic::active_api()
    }
    #[cfg(target_arch = "x86_64")]
    {
        arch::apic::active_api()
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        unimplemented!("Interrupt controller not implemented for this architecture")
    }
}

// TEAM_112: Export cache maintenance for DMA
// TEAM_132: Migrate barriers to aarch64-cpu
pub fn cache_clean_range(start_va: usize, size: usize) {
    #[cfg(target_arch = "aarch64")]
    {
        use aarch64_cpu::asm::barrier;
        use core::arch::asm;
        // Clean data cache by VA to PoC (Point of Coherency)
        let line_size = 64; // Assume 64-byte blocks for AArch64 (CTR_EL0 can be read to be sure but 64 is safe min)
        let start = start_va & !(line_size - 1);
        let end = start_va + size;

        let mut addr = start;
        while addr < end {
            // SAFETY: dc cvac is a cache maintenance instruction, safe with valid VA
            unsafe { asm!("dc cvac, {}", in(reg) addr, options(nostack)) };
            addr += line_size;
        }
        barrier::dsb(barrier::SY);
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        // No-op for other architectures or host tests
        let _ = (start_va, size);
    }
}

// TEAM_103: LevitateVirtioHal moved to levitate-virtio/src/hal_impl.rs
// Only VirtioHal (for virtio-drivers) and StaticMmioTransport remain here
pub use virtio::{StaticMmioTransport, VirtioHal};

use core::mem::ManuallyDrop;
use los_utils::{Mutex, MutexGuard};

/// IRQ-safe lock that disables interrupts while held.
/// Behaviors: [L1]-[L4] interrupt-safe locking
pub struct IrqSafeLock<T> {
    inner: Mutex<T>,
}

impl<T> IrqSafeLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            inner: Mutex::new(data),
        }
    }

    /// [L1] Disables interrupts before acquiring, [L4] data accessible through guard
    pub fn lock(&self) -> IrqSafeLockGuard<'_, T> {
        let state = interrupts::disable(); // [L1] disable before acquire
        let guard = self.inner.lock();
        IrqSafeLockGuard {
            guard: ManuallyDrop::new(guard), // [L4] data access
            state,
        }
    }

    /// TEAM_089: Try to acquire the lock without blocking.
    /// Returns Some(guard) if successful, None if lock is already held.
    /// Disables interrupts before attempting to acquire.
    pub fn try_lock(&self) -> Option<IrqSafeLockGuard<'_, T>> {
        let state = interrupts::disable();
        if let Some(guard) = self.inner.try_lock() {
            Some(IrqSafeLockGuard {
                guard: ManuallyDrop::new(guard),
                state,
            })
        } else {
            // Lock not available, restore interrupts and return None
            interrupts::restore(state);
            None
        }
    }
}

pub struct IrqSafeLockGuard<'a, T> {
    guard: ManuallyDrop<MutexGuard<'a, T>>,
    state: u64,
}

impl<T> core::ops::Deref for IrqSafeLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.guard
    }
}

impl<T> core::ops::DerefMut for IrqSafeLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.guard
    }
}

impl<T> Drop for IrqSafeLockGuard<'_, T> {
    /// [L2] Restores interrupts after releasing
    fn drop(&mut self) {
        // SAFETY: guard is only dropped once, here in Drop, before restoring interrupts
        unsafe { ManuallyDrop::drop(&mut self.guard) };
        interrupts::restore(self.state); // [L2] restore on drop
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    /// Tests: [L1] disable before acquire, [L2] restore after release, [L4] data access
    /// Also tests [I1]-[I5] via interrupt mock
    #[test]
    fn test_irq_safe_lock_behavior() {
        let lock = IrqSafeLock::new(10);

        assert!(interrupts::is_enabled()); // [I4] initially enabled

        {
            let mut guard = lock.lock(); // [L1] disables interrupts
            assert_eq!(*guard, 10); // [L4] read access
            *guard = 20; // [L4] write access

            assert!(!interrupts::is_enabled()); // [I5] disabled while held
        } // [L2] restore on drop

        assert!(interrupts::is_enabled()); // [I4] restored
        assert_eq!(*lock.lock(), 20);
    }

    /// Tests: [L3] nested locks work correctly, [I6] disable→restore preserves state
    #[test]
    fn test_irq_safe_lock_nested() {
        let lock1 = IrqSafeLock::new(1);
        let lock2 = IrqSafeLock::new(2);

        assert!(interrupts::is_enabled());
        {
            let _g1 = lock1.lock(); // [L3] first lock
            assert!(!interrupts::is_enabled());
            {
                let _g2 = lock2.lock(); // [L3] nested lock
                assert!(!interrupts::is_enabled());
            }
            assert!(!interrupts::is_enabled()); // [L3] still disabled after inner drop
        }
        assert!(interrupts::is_enabled()); // [I6] finally restored
    }
}
