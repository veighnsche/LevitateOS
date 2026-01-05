//! VirtIO MMIO Transport and HAL Implementation
//!
//! TEAM_092: Extracted from kernel/src/virtio.rs
//! TEAM_103: Removed LevitateVirtioHal - now in levitate-virtio/src/hal_impl.rs
//!
//! This module provides HAL implementation for the virtio-drivers crate.
//! The levitate-virtio HAL impl has been moved to avoid circular dependencies.

extern crate alloc;

pub use core::ptr::NonNull;
pub use virtio_drivers::{Hal, PhysAddr};

/// Type alias for static MMIO transport from virtio-drivers
pub type StaticMmioTransport = virtio_drivers::transport::mmio::MmioTransport<'static>;

/// HAL implementation for virtio-drivers crate
/// Used by levitate-gpu which wraps virtio-drivers::VirtIOGpu
pub struct VirtioHal;

unsafe impl Hal for VirtioHal {
    fn dma_alloc(
        pages: usize,
        _direction: virtio_drivers::BufferDirection,
    ) -> (PhysAddr, NonNull<u8>) {
        let layout = core::alloc::Layout::from_size_align(pages * 4096, 4096).unwrap();
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            panic!("VirtIO DMA allocation failed");
        }
        let vaddr = ptr as usize;
        let paddr = crate::mmu::virt_to_phys(vaddr);
        (paddr as u64, NonNull::new(ptr).unwrap())
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        let layout = core::alloc::Layout::from_size_align(pages * 4096, 4096).unwrap();
        let vaddr = crate::mmu::phys_to_virt(paddr as usize);
        unsafe { alloc::alloc::dealloc(vaddr as *mut u8, layout) };
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        let vaddr = crate::mmu::phys_to_virt(paddr as usize);
        NonNull::new(vaddr as *mut u8).unwrap()
    }

    unsafe fn share(
        buffer: NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        crate::mmu::virt_to_phys(vaddr) as u64
    }

    unsafe fn unshare(
        _paddr: PhysAddr,
        _buffer: NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) {
        // No-op for identity-mapped memory
    }
}
