use virtio_drivers::{Hal, PhysAddr};
use virtio_drivers::transport::Transport;
use core::ptr::NonNull;
use crate::println;

pub const VIRTIO_MMIO_START: usize = 0x0a000000;
pub const VIRTIO_MMIO_SIZE: usize = 0x200;
pub const VIRTIO_MMIO_COUNT: usize = 32;

pub struct VirtioHal;

unsafe impl Hal for VirtioHal {
    fn dma_alloc(pages: usize, _direction: virtio_drivers::BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let layout = core::alloc::Layout::from_size_align(
            pages * 4096,
            4096,
        ).unwrap();
        // Since we are in a kernel, we can use the global allocator.
        // However, we need zeroed memory.
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            panic!("VirtIO DMA allocation failed");
        }
        (ptr as usize, NonNull::new(ptr).unwrap())
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        let layout = core::alloc::Layout::from_size_align(
            pages * 4096,
            4096,
        ).unwrap();
        unsafe { alloc::alloc::dealloc(paddr as *mut u8, layout) };
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: virtio_drivers::BufferDirection) -> PhysAddr {
        buffer.as_ptr() as *mut u8 as usize
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: virtio_drivers::BufferDirection) {
        // Nothing to do
    }
}

pub fn init() {
    println!("Scanning VirtIO MMIO bus...");
    for i in 0..VIRTIO_MMIO_COUNT {
        let addr = VIRTIO_MMIO_START + i * VIRTIO_MMIO_SIZE;
        let header = core::ptr::NonNull::new(addr as *mut virtio_drivers::transport::mmio::VirtIOHeader).unwrap();
        
        match unsafe { virtio_drivers::transport::mmio::MmioTransport::new(header) } {
            Ok(transport) => {
                let device_type = transport.device_type();
                // println!("Found VirtIO device {:?} at 0x{:x}", device_type, addr);
                
                match device_type {
                    virtio_drivers::transport::DeviceType::GPU => {
                         println!("Initializing GPU...");
                         crate::gpu::init(transport);
                    }
                    virtio_drivers::transport::DeviceType::Input => {
                         println!("Initializing Input...");
                         crate::input::init(transport);
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
