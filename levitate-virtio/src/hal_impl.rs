//! VirtIO HAL Implementation for LevitateOS
//!
//! TEAM_103: Moved from levitate-hal/src/virtio.rs as part of crate reorganization.
//! This module provides the HAL implementation that bridges levitate-virtio to
//! the platform-specific primitives in levitate-hal.

extern crate alloc;

use core::ptr::NonNull;

use crate::{BufferDirection, VirtioHal};

/// TEAM_103: HAL implementation for levitate-virtio crate
/// Uses levitate-hal primitives for DMA and address translation.
pub struct LevitateVirtioHal;

// TEAM_103: Implementation of VirtioHal trait using levitate-hal primitives
unsafe impl VirtioHal for LevitateVirtioHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (u64, NonNull<u8>) {
        let layout = core::alloc::Layout::from_size_align(pages * 4096, 4096).unwrap();
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            panic!("VirtIO DMA allocation failed");
        }
        let vaddr = ptr as usize;
        let paddr = levitate_hal::mmu::virt_to_phys(vaddr);
        (paddr as u64, NonNull::new(ptr).unwrap())
    }

    unsafe fn dma_dealloc(paddr: u64, _vaddr: NonNull<u8>, pages: usize) {
        let layout = core::alloc::Layout::from_size_align(pages * 4096, 4096).unwrap();
        let vaddr = levitate_hal::mmu::phys_to_virt(paddr as usize);
        unsafe { alloc::alloc::dealloc(vaddr as *mut u8, layout) };
    }

    unsafe fn mmio_phys_to_virt(paddr: u64, _size: usize) -> NonNull<u8> {
        let vaddr = levitate_hal::mmu::phys_to_virt(paddr as usize);
        NonNull::new(vaddr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> u64 {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        levitate_hal::mmu::virt_to_phys(vaddr) as u64
    }

    unsafe fn unshare(
        _paddr: u64,
        _buffer: NonNull<[u8]>,
        _direction: BufferDirection,
    ) {
        // No-op for identity-mapped memory
    }

    fn virt_to_phys(vaddr: usize) -> usize {
        levitate_hal::mmu::virt_to_phys(vaddr)
    }

    fn phys_to_virt(paddr: usize) -> usize {
        levitate_hal::mmu::phys_to_virt(paddr)
    }
}
