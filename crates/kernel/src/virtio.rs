//! VirtIO MMIO Transport and HAL Implementation
//!
//! TEAM_032: Updated for virtio-drivers v0.12.0
//! - PhysAddr is now u64 (not usize)
//! - MmioTransport::new() requires mmio_size argument
//! - MmioTransport has lifetime parameter
//!
//! # TEAM_114: GPU Migration to PCI
//!
//! GPU initialization via MMIO is temporarily disabled.
//! The plan is to switch to VirtIO PCI transport for GPU.
//! See: `docs/planning/virtio-pci/` for implementation plan.

// Allow unwrap/panic in HAL trait impls - these are low-level allocators where
// failure to allocate is unrecoverable (system cannot continue)
#![allow(clippy::unwrap_used, clippy::panic)]

extern crate alloc;

pub use los_hal::virtio::{StaticMmioTransport, VirtioHal};
#[cfg(target_arch = "aarch64")]
use virtio_drivers::transport::Transport;

// TEAM_078: Use high VA for VirtIO MMIO (accessible via TTBR1 regardless of TTBR0 state)
pub const VIRTIO_MMIO_START: usize = los_hal::mmu::VIRTIO_MMIO_VA;
pub const VIRTIO_MMIO_SIZE: usize = 0x200;
pub const VIRTIO_MMIO_COUNT: usize = 32;

/// TEAM_065: Initialize GPU device only (Stage 3 - BootConsole)
/// GPU must be available before terminal operations.
/// Returns true if GPU was found and initialized.
/// TEAM_336: Now supports both PCI (x86_64) and MMIO (AArch64) transports
pub fn init_gpu() -> bool {
    // TEAM_336: Detect GPU transport based on architecture
    let transport = detect_gpu_transport();
    
    // Call gpu::init with the detected transport
    crate::gpu::init(transport);

    // Check if GPU was successfully initialized
    crate::gpu::get_resolution().is_some()
}

/// TEAM_337: GPU uses PCI on both architectures (QEMU uses virtio-gpu-pci for all)
fn detect_gpu_transport() -> Option<los_pci::PciTransport> {
    los_pci::find_virtio_gpu::<VirtioHal>()
}

/// TEAM_065: Initialize non-GPU VirtIO devices (Stage 4 - Discovery)
/// Block, Network, Input devices are initialized here.
/// TEAM_287: MMIO scanning only on aarch64. x86_64 uses PCI for VirtIO devices.
pub fn init() {
    #[cfg(target_arch = "aarch64")]
    {
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
                            // TEAM_241: Pass MMIO slot index for IRQ computation
                            crate::input::init(transport, i);
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

    #[cfg(target_arch = "x86_64")]
    {
        // TEAM_331: x86_64 uses PCI for VirtIO devices, not MMIO
        // Initialize input device via PCI
        crate::input::init_pci();
    }
}
