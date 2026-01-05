//! VirtQueue implementation for VirtIO devices.
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//!
//! This module provides a split virtqueue implementation per VirtIO 1.1 spec section 2.6.

use bitflags::bitflags;
use core::sync::atomic::{fence, Ordering};

/// Error types for VirtQueue operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtQueueError {
    /// Queue is full, no descriptors available.
    QueueFull,
    /// Invalid descriptor index.
    InvalidDescriptor,
    /// Buffer too small for response.
    BufferTooSmall,
    /// Device returned an error.
    DeviceError,
    /// Queue not ready.
    NotReady,
}

bitflags! {
    /// Descriptor flags per VirtIO 1.1 spec section 2.6.5.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DescriptorFlags: u16 {
        /// Buffer continues via the next field.
        const NEXT = 1;
        /// Buffer is device write-only (otherwise read-only).
        const WRITE = 2;
        /// Buffer contains a list of buffer descriptors.
        const INDIRECT = 4;
    }
}

/// A single descriptor in the descriptor table.
///
/// Per VirtIO 1.1 spec section 2.6.5.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Descriptor {
    /// Physical address of the buffer.
    pub addr: u64,
    /// Length of the buffer in bytes.
    pub len: u32,
    /// Descriptor flags.
    pub flags: u16,
    /// Next descriptor index if NEXT flag is set.
    pub next: u16,
}

/// Available ring entry.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct AvailRingEntry {
    flags: u16,
    idx: u16,
}

/// Used ring entry.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct UsedRingEntry {
    id: u32,
    len: u32,
}

/// A split virtqueue implementation.
///
/// TEAM_098: This is a simplified implementation focused on correctness
/// and debuggability over performance.
pub struct VirtQueue<const SIZE: usize> {
    /// Descriptor table.
    descriptors: [Descriptor; SIZE],
    /// Available ring flags.
    avail_flags: u16,
    /// Available ring index (next entry to write).
    avail_idx: u16,
    /// Available ring entries.
    avail_ring: [u16; SIZE],
    /// Used ring flags.
    used_flags: u16,
    /// Used ring index (next entry to read).
    used_idx: u16,
    /// Used ring entries.
    used_ring: [UsedRingEntry; SIZE],
    /// Free descriptor head.
    free_head: u16,
    /// Number of free descriptors.
    num_free: u16,
    /// Last seen used index.
    last_used_idx: u16,
}

impl<const SIZE: usize> VirtQueue<SIZE> {
    /// Create a new VirtQueue.
    ///
    /// # Safety
    ///
    /// SIZE must be a power of 2 and <= 32768.
    pub const fn new() -> Self {
        Self {
            descriptors: [Descriptor {
                addr: 0,
                len: 0,
                flags: 0,
                next: 0,
            }; SIZE],
            avail_flags: 0,
            avail_idx: 0,
            avail_ring: [0; SIZE],
            used_flags: 0,
            used_idx: 0,
            used_ring: [UsedRingEntry { id: 0, len: 0 }; SIZE],
            free_head: 0,
            num_free: SIZE as u16,
            last_used_idx: 0,
        }
    }

    /// Initialize the free list.
    pub fn init(&mut self) {
        for i in 0..(SIZE - 1) {
            self.descriptors[i].next = (i + 1) as u16;
        }
        self.descriptors[SIZE - 1].next = 0;
        self.free_head = 0;
        self.num_free = SIZE as u16;
    }

    /// Check if the queue has any free descriptors.
    pub fn has_free_descriptors(&self) -> bool {
        self.num_free > 0
    }

    /// Add a buffer chain to the available ring.
    ///
    /// Returns the head descriptor index.
    pub fn add_buffer(
        &mut self,
        inputs: &[&[u8]],
        outputs: &mut [&mut [u8]],
    ) -> Result<u16, VirtQueueError> {
        let total = inputs.len() + outputs.len();
        if total == 0 {
            return Err(VirtQueueError::InvalidDescriptor);
        }
        if self.num_free < total as u16 {
            return Err(VirtQueueError::QueueFull);
        }

        let head = self.free_head;
        let mut desc_idx = head;

        // Add input buffers (device reads)
        for (i, input) in inputs.iter().enumerate() {
            let desc = &mut self.descriptors[desc_idx as usize];
            desc.addr = input.as_ptr() as u64;
            desc.len = input.len() as u32;
            desc.flags = if i + 1 < total {
                DescriptorFlags::NEXT.bits()
            } else {
                0
            };
            if i + 1 < total {
                desc_idx = desc.next;
            }
        }

        // Add output buffers (device writes)
        let output_count = outputs.len();
        for (i, output) in outputs.iter_mut().enumerate() {
            let desc = &mut self.descriptors[desc_idx as usize];
            desc.addr = output.as_ptr() as u64;
            desc.len = output.len() as u32;
            let is_last = i + 1 == output_count;
            desc.flags = if is_last {
                DescriptorFlags::WRITE.bits()
            } else {
                DescriptorFlags::WRITE.bits() | DescriptorFlags::NEXT.bits()
            };
            if !is_last {
                desc_idx = desc.next;
            }
        }

        // Update free list
        self.free_head = self.descriptors[desc_idx as usize].next;
        self.num_free -= total as u16;

        // Add to available ring
        let avail_slot = (self.avail_idx as usize) % SIZE;
        self.avail_ring[avail_slot] = head;

        // Memory barrier before updating index
        fence(Ordering::SeqCst);

        self.avail_idx = self.avail_idx.wrapping_add(1);

        Ok(head)
    }

    /// Check if there are used buffers to process.
    pub fn has_used(&self) -> bool {
        fence(Ordering::SeqCst);
        self.last_used_idx != self.used_idx
    }

    /// Pop a used buffer from the used ring.
    ///
    /// Returns (descriptor head index, bytes written by device).
    pub fn pop_used(&mut self) -> Option<(u16, u32)> {
        if !self.has_used() {
            return None;
        }

        fence(Ordering::SeqCst);

        let used_slot = (self.last_used_idx as usize) % SIZE;
        let entry = self.used_ring[used_slot];
        self.last_used_idx = self.last_used_idx.wrapping_add(1);

        // Return descriptors to free list
        let mut idx = entry.id as u16;
        loop {
            let desc = &self.descriptors[idx as usize];
            self.num_free += 1;
            if desc.flags & DescriptorFlags::NEXT.bits() == 0 {
                break;
            }
            idx = desc.next;
        }
        self.descriptors[idx as usize].next = self.free_head;
        self.free_head = entry.id as u16;

        Some((entry.id as u16, entry.len))
    }

    /// Get physical addresses for MMIO configuration.
    ///
    /// Returns (desc_addr, avail_addr, used_addr).
    pub fn addresses(&self) -> (usize, usize, usize) {
        let desc_addr = self.descriptors.as_ptr() as usize;
        let avail_addr = &self.avail_flags as *const u16 as usize;
        let used_addr = &self.used_flags as *const u16 as usize;
        (desc_addr, avail_addr, used_addr)
    }

    /// Get the available ring index for notification.
    pub fn avail_idx(&self) -> u16 {
        self.avail_idx
    }
}

impl<const SIZE: usize> Default for VirtQueue<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
