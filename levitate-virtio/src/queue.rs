//! VirtQueue implementation for VirtIO devices.
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//! TEAM_109: Added event fields, padding, volatile writes, DSB barrier.
//!           Still has issues - see docs/VIRTIO_IMPLEMENTATION.md for details.
//!
//! This module provides a split virtqueue implementation per VirtIO 1.1 spec section 2.6.
//!
//! # Known Issues (TEAM_109)
//!
//! This implementation differs architecturally from virtio-drivers:
//! - We embed all data in one struct; virtio-drivers uses separate DMA regions
//! - We write directly; virtio-drivers uses shadow descriptors
//! - We use volatile u16; virtio-drivers uses AtomicU16
//!
//! These differences may cause the device to not respond. If you're debugging
//! timeout issues, consider refactoring to match virtio-drivers' architecture.
//! See `.teams/TEAM_109_fix_gpu_driver_no_fallback.md` for investigation details.

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
// TEAM_106: Added align(16) per VirtIO 1.1 spec section 2.6 requirement.
// Reference: virtio-drivers, Tock OS both use #[repr(C, align(16))]
#[repr(C, align(16))]
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
// TEAM_106: Added align(16) to ensure descriptor table is 16-byte aligned.
// Queue memory should be allocated via HAL's dma_alloc for DMA safety.
#[repr(C, align(16))]
pub struct VirtQueue<const SIZE: usize> {
    /// Descriptor table.
    descriptors: [Descriptor; SIZE],
    /// Available ring flags.
    avail_flags: u16,
    /// Available ring index (next entry to write).
    avail_idx: u16,
    /// Available ring entries.
    avail_ring: [u16; SIZE],
    /// TEAM_109: used_event field per VirtIO spec (even if EVENT_IDX not negotiated)
    used_event: u16,
    /// TEAM_109: Padding to ensure used ring is 4-byte aligned per VirtIO spec 2.6
    /// The used ring MUST be 4-byte aligned, but after avail ring (2-byte aligned fields)
    /// we may be at a 2-byte boundary. This padding ensures proper alignment.
    _padding: u16,
    /// Used ring flags.
    used_flags: u16,
    /// Used ring index (next entry to read).
    used_idx: u16,
    /// Used ring entries.
    used_ring: [UsedRingEntry; SIZE],
    /// TEAM_109: avail_event field per VirtIO spec (even if EVENT_IDX not negotiated)
    avail_event: u16,
    /// Free descriptor head (local driver state, not accessed by device).
    free_head: u16,
    /// Number of free descriptors.
    num_free: u16,
    /// Last seen used index.
    last_used_idx: u16,
}

// TEAM_106: Compile-time alignment verification per VirtIO 1.1 spec
const _: () = assert!(core::mem::align_of::<Descriptor>() >= 16);

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
            used_event: 0,
            _padding: 0,
            used_flags: 0,
            used_idx: 0,
            used_ring: [UsedRingEntry { id: 0, len: 0 }; SIZE],
            avail_event: 0,
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

    /// Add a buffer chain to the available ring with address translation.
    ///
    /// TEAM_100: Added virt_to_phys parameter for proper DMA address translation.
    /// The device needs physical addresses, not virtual addresses.
    ///
    /// Returns the head descriptor index.
    pub fn add_buffer<F>(
        &mut self,
        inputs: &[&[u8]],
        outputs: &mut [&mut [u8]],
        virt_to_phys: F,
    ) -> Result<u16, VirtQueueError>
    where
        F: Fn(usize) -> usize,
    {
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
        // TEAM_109: Use volatile writes - device reads descriptors via DMA
        for (i, input) in inputs.iter().enumerate() {
            let desc_ptr = &mut self.descriptors[desc_idx as usize] as *mut Descriptor;
            let next_idx = unsafe { (*desc_ptr).next };
            // TEAM_100: Convert virtual address to physical for DMA
            unsafe {
                core::ptr::write_volatile(&mut (*desc_ptr).addr, virt_to_phys(input.as_ptr() as usize) as u64);
                core::ptr::write_volatile(&mut (*desc_ptr).len, input.len() as u32);
                core::ptr::write_volatile(&mut (*desc_ptr).flags, if i + 1 < total {
                    DescriptorFlags::NEXT.bits()
                } else {
                    0
                });
            }
            if i + 1 < total {
                desc_idx = next_idx;
            }
        }

        // Add output buffers (device writes)
        // TEAM_109: Use volatile writes - device reads descriptors via DMA
        let output_count = outputs.len();
        for (i, output) in outputs.iter_mut().enumerate() {
            let desc_ptr = &mut self.descriptors[desc_idx as usize] as *mut Descriptor;
            let next_idx = unsafe { (*desc_ptr).next };
            // TEAM_100: Convert virtual address to physical for DMA
            let is_last = i + 1 == output_count;
            unsafe {
                core::ptr::write_volatile(&mut (*desc_ptr).addr, virt_to_phys(output.as_ptr() as usize) as u64);
                core::ptr::write_volatile(&mut (*desc_ptr).len, output.len() as u32);
                core::ptr::write_volatile(&mut (*desc_ptr).flags, if is_last {
                    DescriptorFlags::WRITE.bits()
                } else {
                    DescriptorFlags::WRITE.bits() | DescriptorFlags::NEXT.bits()
                });
            }
            if !is_last {
                desc_idx = next_idx;
            }
        }

        // Update free list
        self.free_head = self.descriptors[desc_idx as usize].next;
        self.num_free -= total as u16;

        // Add to available ring
        // TEAM_109: Volatile write - device reads avail_ring via DMA
        let avail_slot = (self.avail_idx as usize) % SIZE;
        unsafe {
            core::ptr::write_volatile(&mut self.avail_ring[avail_slot], head);
        }

        // Memory barrier before updating index
        fence(Ordering::SeqCst);

        // TEAM_100: Volatile-write avail_idx since device reads via DMA
        let new_idx = self.avail_idx.wrapping_add(1);
        unsafe {
            core::ptr::write_volatile(&mut self.avail_idx as *mut u16, new_idx);
        }

        // TEAM_109: ARM DSB to ensure writes are visible to device
        // The fence above orders CPU memory accesses, but DSB ensures
        // completion of all memory writes before device sees them via DMA.
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("dsb sy", options(nostack, preserves_flags));
        }

        Ok(head)
    }

    /// Check if there are used buffers to process.
    /// 
    /// TEAM_100: Volatile-read the used_idx since device writes via DMA.
    pub fn has_used(&self) -> bool {
        fence(Ordering::SeqCst);
        // The device writes to used_idx via the physical address we gave it.
        // We need to volatile-read it from our memory location.
        let device_used_idx = unsafe {
            core::ptr::read_volatile(&self.used_idx as *const u16)
        };
        self.last_used_idx != device_used_idx
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
        // TEAM_100: Volatile-read since device writes via DMA
        let entry = unsafe {
            core::ptr::read_volatile(&self.used_ring[used_slot] as *const UsedRingEntry)
        };
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
