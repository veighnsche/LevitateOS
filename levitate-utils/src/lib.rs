#![no_std]

use core::cell::UnsafeCell;
use core::marker::{Send, Sync};
use core::ops::{Deref, DerefMut, Drop};
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

    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        SpinlockGuard {
            lock: self,
            data: unsafe { &mut *self.data.get() },
        }
    }
}

impl<'a, T> Drop for SpinlockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.lock.store(false, Ordering::Release);
    }
}

impl<'a, T> Deref for SpinlockGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T> DerefMut for SpinlockGuard<'a, T> {
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

impl<const N: usize> RingBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0; N],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    pub fn push(&mut self, byte: u8) -> bool {
        if self.full {
            return false;
        }

        self.buffer[self.head] = byte;
        self.head = (self.head + 1) % N;
        self.full = self.head == self.tail;
        true
    }

    pub fn pop(&mut self) -> Option<u8> {
        if !self.full && self.head == self.tail {
            return None;
        }

        let byte = self.buffer[self.tail];
        self.tail = (self.tail + 1) % N;
        self.full = false;
        Some(byte)
    }

    pub fn is_empty(&self) -> bool {
        !self.full && self.head == self.tail
    }
}
