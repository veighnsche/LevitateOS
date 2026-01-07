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

// TEAM_130: VirtioHal implementation uses unwrap/expect in places where
// failure would be unrecoverable (DMA allocation, layout creation).
// Per Rule 14 (Fail Loud, Fail Fast), these are acceptable since:
// - Layout::from_size_align only fails if size overflows or align is invalid
// - DMA allocation failure means the system cannot function
// - These are init-time operations, not runtime recoverable errors
#[allow(clippy::expect_used)]
unsafe impl Hal for VirtioHal {
    fn dma_alloc(
        pages: usize,
        _direction: virtio_drivers::BufferDirection,
    ) -> (PhysAddr, NonNull<u8>) {
        // SAFETY: 4096 is a valid alignment (power of 2), size is pages * 4096
        let layout = core::alloc::Layout::from_size_align(pages * 4096, 4096)
            .expect("TEAM_130: Layout creation failed - invalid page count");
        // SAFETY: layout is valid, alloc_zeroed returns null on OOM
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        // Rule 14: DMA allocation failure is unrecoverable - fail fast
        let ptr = NonNull::new(ptr).expect("TEAM_130: VirtIO DMA allocation failed - OOM");
        let vaddr = ptr.as_ptr() as usize;
        let paddr = crate::arch::mmu::virt_to_phys(vaddr);
        (paddr as u64, ptr)
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        // SAFETY: Same layout constraints as dma_alloc
        let layout = core::alloc::Layout::from_size_align(pages * 4096, 4096)
            .expect("TEAM_130: Layout creation failed - invalid page count");
        let vaddr = crate::arch::mmu::phys_to_virt(paddr as usize);
        // SAFETY: vaddr was allocated by dma_alloc with same layout
        unsafe { alloc::alloc::dealloc(vaddr as *mut u8, layout) };
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        let vaddr = crate::arch::mmu::phys_to_virt(paddr as usize);
        // SAFETY: phys_to_virt returns valid mapped address for MMIO regions
        // Rule 14: Null MMIO mapping is unrecoverable - fail fast
        NonNull::new(vaddr as *mut u8).expect("TEAM_130: MMIO phys_to_virt returned null")
    }

    unsafe fn share(
        buffer: NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        let paddr = crate::arch::mmu::virt_to_phys(vaddr);
        paddr as u64
    }

    unsafe fn unshare(
        _paddr: PhysAddr,
        _buffer: NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) {
        // No-op for identity-mapped memory
    }
}
