//! VirtIO MMIO Transport and HAL Implementation
//!
//! TEAM_032: Updated for virtio-drivers v0.12.0
//! - PhysAddr is now u64 (not usize)
//! - MmioTransport::new() requires mmio_size argument
//! - MmioTransport has lifetime parameter
//!
//! # ⚠️ WARNING: GPU INITIALIZATION IS BROKEN ⚠️
//!
//! The `init_gpu()` function below uses `levitate-gpu` which gives **FALSE POSITIVES**.
//! The GPU driver initializes successfully but the display shows NOTHING.
//!
//! **DO NOT** think this code is correct just because tests pass.
//! **DO NOT** revert to this approach when fixing levitate-drivers-gpu.
//!
//! The real fix is in `levitate-drivers-gpu` - see `docs/VIRTIO_IMPLEMENTATION.md`

// Allow unwrap/panic in HAL trait impls - these are low-level allocators where
// failure to allocate is unrecoverable (system cannot continue)
#![allow(clippy::unwrap_used, clippy::panic)]

extern crate alloc;

pub use levitate_hal::virtio::{StaticMmioTransport, VirtioHal};
use virtio_drivers::transport::Transport;

// TEAM_078: Use high VA for VirtIO MMIO (accessible via TTBR1 regardless of TTBR0 state)
pub const VIRTIO_MMIO_START: usize = levitate_hal::mmu::VIRTIO_MMIO_VA;
pub const VIRTIO_MMIO_SIZE: usize = 0x200;
pub const VIRTIO_MMIO_COUNT: usize = 32;

/// TEAM_065: Initialize GPU device only (Stage 3 - BootConsole)
/// GPU must be available before terminal operations.
/// Returns true if GPU was found and initialized.
pub fn init_gpu() -> bool {
    crate::verbose!("Scanning VirtIO MMIO bus for GPU...");
    for i in 0..VIRTIO_MMIO_COUNT {
        let addr = VIRTIO_MMIO_START + i * VIRTIO_MMIO_SIZE;
        let header =
            core::ptr::NonNull::new(addr as *mut virtio_drivers::transport::mmio::VirtIOHeader)
                .unwrap();

        match unsafe {
            virtio_drivers::transport::mmio::MmioTransport::new(header, VIRTIO_MMIO_SIZE)
        } {
            Ok(transport) => {
                if transport.device_type() == virtio_drivers::transport::DeviceType::GPU {
                    crate::gpu::init(transport);
                    return true;
                }
            }
            Err(_) => {
                // Not a valid VirtIO device at this address
            }
        }
    }
    false
}

/// TEAM_065: Initialize non-GPU VirtIO devices (Stage 4 - Discovery)
/// Block, Network, Input devices are initialized here.
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
                    // GPU already initialized in Stage 3 via init_gpu()
                    virtio_drivers::transport::DeviceType::GPU => {}
                    virtio_drivers::transport::DeviceType::Input => {
                        crate::input::init(transport);
                    }
                    virtio_drivers::transport::DeviceType::Block => {
                        crate::block::init(transport);
                    }
                    // TEAM_057: VirtIO Net driver
                    virtio_drivers::transport::DeviceType::Network => {
                        crate::net::init(transport);
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
