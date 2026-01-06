#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod cpio;
pub mod hex;

// TEAM_211: Re-export spin crate types as our lock API
// Note: spin::Mutex is re-exported as Mutex for API compatibility
pub use spin::{Barrier, Lazy, Once};
pub use spin::{Mutex, MutexGuard};
pub use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

// TEAM_212: Re-export hashbrown collections
pub use hashbrown::{HashMap, HashSet};

pub struct RingBuffer<T: Copy, const N: usize> {
    buffer: [T; N],
    head: usize,
    tail: usize,
    full: bool,
}

impl<T: Copy + Default, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    /// [R1] New buffer is empty
    #[must_use]
    pub const fn new(default_val: T) -> Self {
        Self {
            buffer: [default_val; N],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    /// [R2] Push adds element, [R4] returns false when full
    pub fn push(&mut self, item: T) -> bool {
        if self.full {
            return false; // [R4]
        }

        self.buffer[self.head] = item; // [R2]
        self.head = (self.head + 1) % N; // [R6] wrap around
        self.full = self.head == self.tail;
        true
    }

    /// [R3] Pop removes oldest (FIFO), [R5] returns None when empty
    pub fn pop(&mut self) -> Option<T> {
        if !self.full && self.head == self.tail {
            return None; // [R5]
        }

        let item = self.buffer[self.tail]; // [R3] FIFO order
        self.tail = (self.tail + 1) % N; // [R6] wrap around
        self.full = false;
        Some(item)
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
        let lock = Mutex::new(42);
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

        let lock = Arc::new(Mutex::new(()));
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
        let mut rb = RingBuffer::<u8, 4>::new(0);
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
        let mut rb = RingBuffer::<u8, 2>::new(0);
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
        let mut rb = RingBuffer::<u8, 4>::new(0);
        assert!(rb.is_empty());
        rb.push(42);
        assert!(!rb.is_empty()); // [R8] false when has data
        rb.push(43);
        assert!(!rb.is_empty());
    }

    /// Tests: HashMap insert/get
    #[test]
    fn test_hashmap_basic() {
        let mut map = HashMap::new();
        map.insert(1, "one");
        map.insert(2, "two");
        assert_eq!(map.get(&1), Some(&"one"));
        assert_eq!(map.get(&3), None);
    }
}
