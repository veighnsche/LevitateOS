//! VirtIO MMIO Transport and HAL Implementation
//!
//! TEAM_032: Updated for virtio-drivers v0.12.0
//! - PhysAddr is now u64 (not usize)
//! - MmioTransport::new() requires mmio_size argument
//! - MmioTransport has lifetime parameter

extern crate alloc;

use core::ptr::NonNull;
use virtio_drivers::transport::Transport;
use virtio_drivers::{Hal, PhysAddr};

pub const VIRTIO_MMIO_START: usize = 0x0a000000;
pub const VIRTIO_MMIO_SIZE: usize = 0x200;
pub const VIRTIO_MMIO_COUNT: usize = 32;

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
        let paddr = levitate_hal::mmu::virt_to_phys(vaddr);
        // TEAM_032: PhysAddr is now u64
        (paddr as u64, NonNull::new(ptr).unwrap())
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        let layout = core::alloc::Layout::from_size_align(pages * 4096, 4096).unwrap();
        // TEAM_032: PhysAddr is now u64, convert to usize
        let vaddr = levitate_hal::mmu::phys_to_virt(paddr as usize);
        unsafe { alloc::alloc::dealloc(vaddr as *mut u8, layout) };
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        // TEAM_032: PhysAddr is now u64, convert to usize
        let vaddr = levitate_hal::mmu::phys_to_virt(paddr as usize);
        NonNull::new(vaddr as *mut u8).unwrap()
    }

    unsafe fn share(
        buffer: NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        // TEAM_032: Return u64 PhysAddr
        levitate_hal::mmu::virt_to_phys(vaddr) as u64
    }

    unsafe fn unshare(
        _paddr: PhysAddr,
        _buffer: NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) {
        // Nothing to do
    }
}

/// Type alias for MmioTransport with 'static lifetime
/// This works because MMIO addresses are fixed hardware addresses
pub type StaticMmioTransport = virtio_drivers::transport::mmio::MmioTransport<'static>;

pub fn init() {
    crate::verbose!("Scanning VirtIO MMIO bus...");
    for i in 0..VIRTIO_MMIO_COUNT {
        let addr = VIRTIO_MMIO_START + i * VIRTIO_MMIO_SIZE;
        let header =
            core::ptr::NonNull::new(addr as *mut virtio_drivers::transport::mmio::VirtIOHeader)
                .unwrap();

        // TEAM_032: MmioTransport::new now requires mmio_size
        match unsafe {
            virtio_drivers::transport::mmio::MmioTransport::new(header, VIRTIO_MMIO_SIZE)
        } {
            Ok(transport) => {
                let device_type = transport.device_type();
                match device_type {
                    virtio_drivers::transport::DeviceType::GPU => {
                        crate::gpu::init(transport);
                    }
                    virtio_drivers::transport::DeviceType::Input => {
                        crate::input::init(transport);
                    }
                    virtio_drivers::transport::DeviceType::Block => {
                        crate::block::init(transport);
                    }
                    _ => {}
                }
            }
            Err(_) => {
                // Not a valid VirtIO device
            }
        }
    }
}
