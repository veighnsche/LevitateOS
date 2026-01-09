//! TEAM_233: Pipe implementation for inter-process communication.
//!
//! Provides POSIX-like pipes with blocking read/write semantics.

extern crate alloc;

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use los_hal::IrqSafeLock;

/// TEAM_233: Pipe buffer size (one page for simplicity).
pub const PIPE_BUF_SIZE: usize = 4096;

/// TEAM_233: Simple ring buffer for pipe data.
pub struct RingBuffer {
    buffer: [u8; PIPE_BUF_SIZE],
    read_pos: usize,
    write_pos: usize,
    count: usize,
}

impl RingBuffer {
    /// Create a new empty ring buffer.
    pub const fn new() -> Self {
        Self {
            buffer: [0; PIPE_BUF_SIZE],
            read_pos: 0,
            write_pos: 0,
            count: 0,
        }
    }

    /// Check if buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if buffer is full.
    pub fn is_full(&self) -> bool {
        self.count == PIPE_BUF_SIZE
    }

    /// Number of bytes available to read.
    pub fn available(&self) -> usize {
        self.count
    }

    /// Number of bytes that can be written.
    pub fn space(&self) -> usize {
        PIPE_BUF_SIZE - self.count
    }

    /// Write data to the buffer. Returns number of bytes written.
    pub fn write(&mut self, data: &[u8]) -> usize {
        let mut written = 0;
        for &byte in data {
            if self.is_full() {
                break;
            }
            self.buffer[self.write_pos] = byte;
            self.write_pos = (self.write_pos + 1) % PIPE_BUF_SIZE;
            self.count += 1;
            written += 1;
        }
        written
    }

    /// Read data from the buffer. Returns number of bytes read.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut read = 0;
        for byte in buf.iter_mut() {
            if self.is_empty() {
                break;
            }
            *byte = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % PIPE_BUF_SIZE;
            self.count -= 1;
            read += 1;
        }
        read
    }
}

/// TEAM_233: Pipe object shared between read and write ends.
pub struct Pipe {
    /// Ring buffer holding pipe data.
    buffer: IrqSafeLock<RingBuffer>,
    /// Is the read end still open?
    read_open: AtomicBool,
    /// Is the write end still open?
    write_open: AtomicBool,
    /// TEAM_333: Active reader count
    readers: AtomicUsize,
    /// TEAM_333: Active writer count
    writers: AtomicUsize,
}

impl Pipe {
    /// Create a new empty pipe (starts with 1 reader, 1 writer).
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            buffer: IrqSafeLock::new(RingBuffer::new()),
            read_open: AtomicBool::new(true),
            write_open: AtomicBool::new(true),
            readers: AtomicUsize::new(1),
            writers: AtomicUsize::new(1),
        })
    }

    /// Read from pipe. Returns bytes read, or 0 if EOF (write end closed).
    ///
    /// For MVP, this is non-blocking: returns 0 immediately if buffer empty
    /// and write end is still open (caller should retry or wait).
    pub fn read(&self, buf: &mut [u8]) -> isize {
        // Check if write end is closed and buffer is empty = EOF
        if !self.write_open.load(Ordering::Acquire) {
            let ring = self.buffer.lock();
            if ring.is_empty() {
                return 0; // EOF
            }
        }

        let mut ring = self.buffer.lock();
        let n = ring.read(buf);

        // If we couldn't read anything but write end is open, return EAGAIN
        if n == 0 && self.write_open.load(Ordering::Acquire) {
            return -11; // EAGAIN
        }

        n as isize
    }

    /// Write to pipe. Returns bytes written, or -EPIPE if read end closed.
    ///
    /// For MVP, this is non-blocking: returns -EAGAIN if buffer full.
    pub fn write(&self, data: &[u8]) -> isize {
        // If read end is closed, return EPIPE
        if !self.read_open.load(Ordering::Acquire) {
            return -32; // EPIPE
        }

        let mut ring = self.buffer.lock();
        let n = ring.write(data);

        // If we couldn't write anything, return EAGAIN
        if n == 0 {
            return -11; // EAGAIN
        }

        n as isize
    }

    /// Increment reader count (for dup/clone)
    pub fn inc_read(&self) {
        self.readers.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment writer count (for dup/clone)
    pub fn inc_write(&self) {
        self.writers.fetch_add(1, Ordering::Relaxed);
    }

    /// Close (decrement) the read end.
    pub fn close_read(&self) {
        if self.readers.fetch_sub(1, Ordering::Release) == 1 {
            // Last reader closed
            self.read_open.store(false, Ordering::Release);
        }
    }

    /// Close (decrement) the write end.
    pub fn close_write(&self) {
        if self.writers.fetch_sub(1, Ordering::Release) == 1 {
            // Last writer closed
            self.write_open.store(false, Ordering::Release);
        }
    }

    /// Check if read end is open.
    pub fn is_read_open(&self) -> bool {
        self.read_open.load(Ordering::Acquire)
    }

    /// Check if write end is open.
    pub fn is_write_open(&self) -> bool {
        self.write_open.load(Ordering::Acquire)
    }
}

/// TEAM_233: Reference to shared pipe.
pub type PipeRef = Arc<Pipe>;
