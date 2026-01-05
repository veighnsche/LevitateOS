//! VirtIO transport abstraction.
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//!
//! This module provides the Transport trait and MMIO implementation.

use core::ptr::{read_volatile, write_volatile};

/// Error types for transport operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportError {
    /// Device not found or invalid.
    NotFound,
    /// Invalid device configuration.
    InvalidConfig,
    /// Feature negotiation failed.
    FeatureNegotiationFailed,
    /// Queue setup failed.
    QueueSetupFailed,
    /// Device is in failed state.
    DeviceFailed,
}

/// VirtIO device type identifiers per VirtIO 1.1 spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DeviceType {
    Invalid = 0,
    Network = 1,
    Block = 2,
    Console = 3,
    Entropy = 4,
    Balloon = 5,
    IoMemory = 6,
    Rpmsg = 7,
    Scsi = 8,
    Transport9P = 9,
    Mac80211 = 10,
    RprocSerial = 11,
    Caif = 12,
    MemoryBalloon = 13,
    Gpu = 16,
    Timer = 17,
    Input = 18,
    Socket = 19,
    Crypto = 20,
    SignalDist = 21,
    Pstore = 22,
    Iommu = 23,
    Memory = 24,
}

impl From<u32> for DeviceType {
    fn from(value: u32) -> Self {
        match value {
            1 => DeviceType::Network,
            2 => DeviceType::Block,
            3 => DeviceType::Console,
            16 => DeviceType::Gpu,
            18 => DeviceType::Input,
            _ => DeviceType::Invalid,
        }
    }
}

/// Transport trait for VirtIO devices.
///
/// Abstracts over MMIO and PCI transports.
pub trait Transport {
    /// Get the device type.
    fn device_type(&self) -> DeviceType;

    /// Read device features.
    fn read_device_features(&mut self) -> u64;

    /// Write driver features.
    fn write_driver_features(&mut self, features: u64);

    /// Get device status.
    fn read_status(&self) -> u32;

    /// Set device status.
    fn write_status(&mut self, status: u32);

    /// Get the maximum queue size for a queue.
    fn max_queue_size(&mut self, queue_idx: u16) -> u16;

    /// Configure a queue.
    fn queue_set(
        &mut self,
        queue_idx: u16,
        queue_size: u16,
        desc_addr: usize,
        avail_addr: usize,
        used_addr: usize,
    );

    /// Notify the device about a queue.
    fn queue_notify(&mut self, queue_idx: u16);

    /// Read from device configuration space.
    fn read_config<T: Copy>(&self, offset: usize) -> T;

    /// Check and acknowledge interrupt.
    fn ack_interrupt(&mut self) -> bool;

    /// Perform device reset.
    fn reset(&mut self);
}

/// VirtIO MMIO register offsets per VirtIO 1.1 spec section 4.2.2.
mod mmio_regs {
    pub const MAGIC: usize = 0x000;
    pub const VERSION: usize = 0x004;
    pub const DEVICE_ID: usize = 0x008;
    pub const DEVICE_FEATURES: usize = 0x010;
    pub const DEVICE_FEATURES_SEL: usize = 0x014;
    pub const DRIVER_FEATURES: usize = 0x020;
    pub const DRIVER_FEATURES_SEL: usize = 0x024;
    pub const QUEUE_SEL: usize = 0x030;
    pub const QUEUE_NUM_MAX: usize = 0x034;
    pub const QUEUE_NUM: usize = 0x038;
    // Version 2 only
    pub const QUEUE_READY: usize = 0x044;
    pub const QUEUE_NOTIFY: usize = 0x050;
    pub const INTERRUPT_STATUS: usize = 0x060;
    pub const INTERRUPT_ACK: usize = 0x064;
    pub const STATUS: usize = 0x070;
    pub const QUEUE_DESC_LOW: usize = 0x080;
    pub const QUEUE_DESC_HIGH: usize = 0x084;
    pub const QUEUE_AVAIL_LOW: usize = 0x090;
    pub const QUEUE_AVAIL_HIGH: usize = 0x094;
    pub const QUEUE_USED_LOW: usize = 0x0a0;
    pub const QUEUE_USED_HIGH: usize = 0x0a4;
    // Version 1 only
    pub const GUEST_PAGE_SIZE: usize = 0x028;
    pub const QUEUE_ALIGN: usize = 0x03c;
    pub const QUEUE_PFN: usize = 0x040;

    pub const CONFIG: usize = 0x100;
}

/// VirtIO MMIO magic value.
const VIRTIO_MAGIC: u32 = 0x7472_6976; // "virt" in little-endian

/// VirtIO MMIO transport implementation.
pub struct MmioTransport {
    base: usize,
    version: u32,
}

impl MmioTransport {
    /// Create a new MMIO transport.
    ///
    /// # Safety
    ///
    /// `base` must be a valid MMIO address for a VirtIO device.
    pub unsafe fn new(base: usize) -> Result<Self, TransportError> {
        let mut transport = Self { base, version: 0 };

        // Verify magic value
        let magic = transport.read_reg(mmio_regs::MAGIC);
        if magic != VIRTIO_MAGIC {
            return Err(TransportError::NotFound);
        }

        // TEAM_100: Accept both legacy (1) and modern (2) VirtIO MMIO
        // QEMU's virt machine uses version 2, but be lenient
        let version = transport.read_reg(mmio_regs::VERSION);
        if version == 0 || version > 2 {
            return Err(TransportError::InvalidConfig);
        }
        transport.version = version;

        Ok(transport)
    }

    #[inline]
    fn read_reg(&self, offset: usize) -> u32 {
        // SAFETY: Caller guarantees base is valid MMIO address.
        unsafe { read_volatile((self.base + offset) as *const u32) }
    }

    #[inline]
    fn write_reg(&mut self, offset: usize, value: u32) {
        // SAFETY: Caller guarantees base is valid MMIO address.
        unsafe { write_volatile((self.base + offset) as *mut u32, value) }
    }
}

impl Transport for MmioTransport {
    fn device_type(&self) -> DeviceType {
        DeviceType::from(self.read_reg(mmio_regs::DEVICE_ID))
    }

    fn read_device_features(&mut self) -> u64 {
        self.write_reg(mmio_regs::DEVICE_FEATURES_SEL, 0);
        let low = self.read_reg(mmio_regs::DEVICE_FEATURES) as u64;
        self.write_reg(mmio_regs::DEVICE_FEATURES_SEL, 1);
        let high = self.read_reg(mmio_regs::DEVICE_FEATURES) as u64;
        low | (high << 32)
    }

    fn write_driver_features(&mut self, features: u64) {
        self.write_reg(mmio_regs::DRIVER_FEATURES_SEL, 0);
        self.write_reg(mmio_regs::DRIVER_FEATURES, features as u32);
        self.write_reg(mmio_regs::DRIVER_FEATURES_SEL, 1);
        self.write_reg(mmio_regs::DRIVER_FEATURES, (features >> 32) as u32);
    }

    fn read_status(&self) -> u32 {
        self.read_reg(mmio_regs::STATUS)
    }

    fn write_status(&mut self, status: u32) {
        self.write_reg(mmio_regs::STATUS, status);
    }

    fn max_queue_size(&mut self, queue_idx: u16) -> u16 {
        self.write_reg(mmio_regs::QUEUE_SEL, queue_idx as u32);
        self.read_reg(mmio_regs::QUEUE_NUM_MAX) as u16
    }

    fn queue_set(
        &mut self,
        queue_idx: u16,
        queue_size: u16,
        desc_addr: usize,
        avail_addr: usize,
        used_addr: usize,
    ) {
        self.write_reg(mmio_regs::QUEUE_SEL, queue_idx as u32);

        if self.version == 2 {
            self.write_reg(mmio_regs::QUEUE_NUM, queue_size as u32);
            self.write_reg(mmio_regs::QUEUE_DESC_LOW, desc_addr as u32);
            self.write_reg(mmio_regs::QUEUE_DESC_HIGH, (desc_addr >> 32) as u32);
            self.write_reg(mmio_regs::QUEUE_AVAIL_LOW, avail_addr as u32);
            self.write_reg(mmio_regs::QUEUE_AVAIL_HIGH, (avail_addr >> 32) as u32);
            self.write_reg(mmio_regs::QUEUE_USED_LOW, used_addr as u32);
            self.write_reg(mmio_regs::QUEUE_USED_HIGH, (used_addr >> 32) as u32);
            self.write_reg(mmio_regs::QUEUE_READY, 1);
        } else {
            // Version 1 (Legacy) support
            self.write_reg(mmio_regs::QUEUE_NUM, queue_size as u32);
            // We use a single struct for everything, so alignment is minimal (4 bytes for Used ring)
            self.write_reg(mmio_regs::QUEUE_ALIGN, 4);
            self.write_reg(mmio_regs::GUEST_PAGE_SIZE, 4096);
            self.write_reg(mmio_regs::QUEUE_PFN, (desc_addr / 4096) as u32);
        }
    }

    fn queue_notify(&mut self, queue_idx: u16) {
        self.write_reg(mmio_regs::QUEUE_NOTIFY, queue_idx as u32);
    }

    fn read_config<T: Copy>(&self, offset: usize) -> T {
        // SAFETY: Caller guarantees T fits in config space.
        unsafe { read_volatile((self.base + mmio_regs::CONFIG + offset) as *const T) }
    }

    fn ack_interrupt(&mut self) -> bool {
        let status = self.read_reg(mmio_regs::INTERRUPT_STATUS);
        if status != 0 {
            self.write_reg(mmio_regs::INTERRUPT_ACK, status);
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.write_reg(mmio_regs::STATUS, 0);
    }
}
