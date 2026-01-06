//! # levitate-virtio
//!
//! General-purpose VirtIO transport layer for LevitateOS.
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//! TEAM_141: Removed LevitateVirtioHal - committing to virtio-drivers (Option A)
//!
//! This crate provides reference abstractions for VirtIO devices:
//! - [`VirtQueue`] - Split virtqueue implementation
//! - [`Transport`] trait - Abstraction over MMIO transports
//! - [`Descriptor`] and buffer management
//!
//! Note: Active drivers use `virtio-drivers` crate. These abstractions are kept
//! for reference and potential future use on platforms without virtio-drivers support.

#![no_std]

pub mod hal;
mod queue;
mod transport;

pub use hal::{BufferDirection, VirtioHal, PAGE_SIZE, pages_for};
pub use queue::{Descriptor, DescriptorFlags, VirtQueue, VirtQueueError};
pub use transport::{DeviceType, MmioTransport, Transport, TransportError};

/// VirtIO device status flags per VirtIO 1.1 spec section 2.1
pub mod status {
    pub const ACKNOWLEDGE: u32 = 1;
    pub const DRIVER: u32 = 2;
    pub const DRIVER_OK: u32 = 4;
    pub const FEATURES_OK: u32 = 8;
    pub const DEVICE_NEEDS_RESET: u32 = 64;
    pub const FAILED: u32 = 128;
}

/// VirtIO feature bits common to all devices
pub mod features {
    pub const RING_INDIRECT_DESC: u64 = 1 << 28;
    pub const RING_EVENT_IDX: u64 = 1 << 29;
    pub const VERSION_1: u64 = 1 << 32;
}
