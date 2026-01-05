//! # levitate-virtio
//!
//! General-purpose VirtIO transport layer for LevitateOS.
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//!
//! This crate provides the foundational abstractions for VirtIO devices:
//! - [`VirtQueue`] - Split virtqueue implementation
//! - [`Transport`] trait - Abstraction over MMIO transports
//! - [`Descriptor`] and buffer management
//!
//! Device-specific drivers (GPU, Block, Net) build on top of these primitives.

#![no_std]

pub mod hal;
mod queue;
mod transport;

// TEAM_103: HAL implementation moved from levitate-hal
// Only available when hal-impl feature is enabled (to avoid circular deps)
#[cfg(feature = "hal-impl")]
mod hal_impl;
#[cfg(feature = "hal-impl")]
pub use hal_impl::LevitateVirtioHal;

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
