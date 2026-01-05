//! Hardware Abstraction Layer for VirtIO
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//!
//! This module defines the HAL trait that platform code must implement
//! for DMA allocation and memory translation.

use core::ptr::NonNull;

/// Direction of buffer access for DMA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferDirection {
    /// Driver writes to buffer, device reads.
    DriverToDevice,
    /// Device writes to buffer, driver reads.
    DeviceToDriver,
    /// Both driver and device may access.
    Both,
}

/// Hardware Abstraction Layer for VirtIO devices.
///
/// Platform code must implement this trait to provide:
/// - DMA memory allocation
/// - Virtual-to-physical address translation
/// - MMIO access
///
/// # Safety
///
/// Implementations must ensure:
/// - DMA memory is physically contiguous
/// - Address translations are correct
/// - MMIO mappings are valid
pub unsafe trait VirtioHal {
    /// Allocate DMA-capable memory.
    ///
    /// Returns (physical_address, virtual_pointer).
    ///
    /// # Arguments
    /// * `pages` - Number of 4KB pages to allocate
    /// * `direction` - Buffer access direction
    fn dma_alloc(pages: usize, direction: BufferDirection) -> (u64, NonNull<u8>);

    /// Deallocate DMA memory.
    ///
    /// # Safety
    ///
    /// `paddr` and `vaddr` must have been returned by a previous call to `dma_alloc`.
    unsafe fn dma_dealloc(paddr: u64, vaddr: NonNull<u8>, pages: usize);

    /// Convert a physical MMIO address to a virtual pointer.
    ///
    /// # Safety
    ///
    /// `paddr` must be a valid MMIO address.
    unsafe fn mmio_phys_to_virt(paddr: u64, size: usize) -> NonNull<u8>;

    /// Share a buffer with the device, returning its physical address.
    ///
    /// # Safety
    ///
    /// `buffer` must be valid for the duration of the DMA operation.
    unsafe fn share(buffer: NonNull<[u8]>, direction: BufferDirection) -> u64;

    /// Unshare a buffer previously shared with the device.
    ///
    /// # Safety
    ///
    /// `paddr` and `buffer` must match a previous call to `share`.
    unsafe fn unshare(paddr: u64, buffer: NonNull<[u8]>, direction: BufferDirection);

    /// Convert virtual address to physical address.
    fn virt_to_phys(vaddr: usize) -> usize;

    /// Convert physical address to virtual address.
    fn phys_to_virt(paddr: usize) -> usize;
}

/// Size of a page for DMA allocation.
pub const PAGE_SIZE: usize = 4096;

/// Calculate number of pages needed for a given size.
pub const fn pages_for(size: usize) -> usize {
    (size + PAGE_SIZE - 1) / PAGE_SIZE
}
