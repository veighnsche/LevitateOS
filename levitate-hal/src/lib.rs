#![no_std]

pub mod console;
pub mod gic;
pub mod interrupts;
pub mod timer;
pub mod uart_pl011;

use levitate_utils::{Spinlock, SpinlockGuard};

pub struct IrqSafeLock<T> {
    inner: Spinlock<T>,
}

impl<T> IrqSafeLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            inner: Spinlock::new(data),
        }
    }

    pub fn lock(&self) -> IrqSafeLockGuard<'_, T> {
        let state = interrupts::disable();
        let guard = self.inner.lock();
        IrqSafeLockGuard {
            guard: Some(guard),
            state,
        }
    }
}

pub struct IrqSafeLockGuard<'a, T> {
    guard: Option<SpinlockGuard<'a, T>>,
    state: u64,
}

impl<'a, T> core::ops::Deref for IrqSafeLockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.guard.as_ref().unwrap()
    }
}

impl<'a, T> core::ops::DerefMut for IrqSafeLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.as_mut().unwrap()
    }
}

impl<'a, T> Drop for IrqSafeLockGuard<'a, T> {
    fn drop(&mut self) {
        self.guard.take();
        interrupts::restore(self.state);
    }
}
