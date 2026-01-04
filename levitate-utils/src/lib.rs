#![cfg_attr(not(feature = "std"), no_std)]

pub mod cpio;
pub mod hex;

use core::cell::UnsafeCell;
use core::marker::{Send, Sync};
use core::ops::{Deref, DerefMut, Drop};
use core::option::Option::{self, None, Some};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct Spinlock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Spinlock<T> {}
unsafe impl<T: Send> Send for Spinlock<T> {}

pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
    data: &'a mut T,
}

impl<T> Spinlock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquire the lock, blocking until available.
    /// Behaviors: [S1] exclusive access, [S2] blocks until released
    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop(); // [S2] spin until lock released
        }
        SpinlockGuard {
            lock: self,
            data: unsafe { &mut *self.data.get() }, // [S4][S5] data access
        }
    }
}

impl<T> Drop for SpinlockGuard<'_, T> {
    /// [S3] Guard releases lock on drop
    fn drop(&mut self) {
        self.lock.lock.store(false, Ordering::Release);
    }
}

impl<T> Deref for SpinlockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T> DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

pub struct RingBuffer<const N: usize> {
    buffer: [u8; N],
    head: usize,
    tail: usize,
    full: bool,
}

impl<const N: usize> Default for RingBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> RingBuffer<N> {
    /// [R1] New buffer is empty
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: [0; N],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    /// [R2] Push adds element, [R4] returns false when full
    pub fn push(&mut self, byte: u8) -> bool {
        if self.full {
            return false; // [R4]
        }

        self.buffer[self.head] = byte; // [R2]
        self.head = (self.head + 1) % N; // [R6] wrap around
        self.full = self.head == self.tail;
        true
    }

    /// [R3] Pop removes oldest (FIFO), [R5] returns None when empty
    pub fn pop(&mut self) -> Option<u8> {
        if !self.full && self.head == self.tail {
            return None; // [R5]
        }

        let byte = self.buffer[self.tail]; // [R3] FIFO order
        self.tail = (self.tail + 1) % N; // [R6] wrap around
        self.full = false;
        Some(byte)
    }

    /// [R7] returns true when empty, [R8] returns false when has data
    pub fn is_empty(&self) -> bool {
        !self.full && self.head == self.tail
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    /// Tests: [S1] exclusive access, [S3] release on drop, [S4] read, [S5] write, [S6] cycles
    #[test]
    fn test_spinlock_basic() {
        let lock = Spinlock::new(42);
        {
            let mut guard = lock.lock(); // [S1] acquire
            assert_eq!(*guard, 42); // [S4] read access
            *guard = 43; // [S5] write access
        } // [S3] release on drop
        assert_eq!(*lock.lock(), 43); // [S6] multiple cycles
    }

    /// Tests: [S2] Lock blocks until released
    #[test]
    fn test_spinlock_blocking() {
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        let lock = Arc::new(Spinlock::new(()));
        let lock_clone = lock.clone();

        let start = std::time::Instant::now();

        // Thread takes lock and holds it for 100ms
        let h = thread::spawn(move || {
            let _g = lock_clone.lock();
            thread::sleep(Duration::from_millis(100));
        });

        // Give thread time to acquire
        thread::sleep(Duration::from_millis(10));

        // This should block until thread releases (~90ms remaining)
        let _g = lock.lock();

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(100));

        h.join().unwrap();
    }

    /// Tests: [R1] new empty, [R2] push, [R3] FIFO, [R4] full, [R5] empty pop, [R7] is_empty true
    #[test]
    fn test_ring_buffer_fifo() {
        let mut rb = RingBuffer::<4>::new();
        assert!(rb.is_empty()); // [R1] new is empty

        assert!(rb.push(1)); // [R2] push adds
        assert!(rb.push(2));
        assert!(rb.push(3));
        assert!(rb.push(4));
        assert!(!rb.push(5)); // [R4] full returns false

        assert_eq!(rb.pop(), Some(1)); // [R3] FIFO order
        assert_eq!(rb.pop(), Some(2));
        assert_eq!(rb.pop(), Some(3));
        assert_eq!(rb.pop(), Some(4));
        assert_eq!(rb.pop(), None); // [R5] empty returns None
        assert!(rb.is_empty()); // [R7] is_empty true
    }

    /// Tests: [R6] wrap around
    #[test]
    fn test_ring_buffer_wrap_around() {
        let mut rb = RingBuffer::<2>::new();
        rb.push(1);
        rb.push(2);
        rb.pop();
        rb.push(3); // [R6] wraps around
        assert_eq!(rb.pop(), Some(2));
        assert_eq!(rb.pop(), Some(3));
        assert!(rb.is_empty());
    }

    /// Tests: [R8] is_empty returns false when has data
    #[test]
    fn test_ring_buffer_is_empty_false_when_has_data() {
        let mut rb = RingBuffer::<4>::new();
        assert!(rb.is_empty());
        rb.push(42);
        assert!(!rb.is_empty()); // [R8] false when has data
        rb.push(43);
        assert!(!rb.is_empty());
    }
}
