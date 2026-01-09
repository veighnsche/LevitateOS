//! NVMe Driver for LevitateOS
//!
//! This is a stub implementation that will be expanded to support
//! NVMe storage devices via PCI.

#![no_std]

use storage_device::StorageDevice;
use storage_device::StorageError;

/// NVMe Controller
pub struct NvmeController {
    // TODO: Add PCI transport and MMIO registers
}

impl NvmeController {
    /// Create a new NVMe controller stub
    pub fn new() -> Self {
        Self {}
    }
}

impl StorageDevice for NvmeController {
    fn block_size(&self) -> usize {
        512 // Default for now
    }

    fn size_in_blocks(&self) -> usize {
        0
    }

    fn read_blocks(&mut self, _block_id: usize, _buf: &mut [u8]) -> Result<(), StorageError> {
        Err(StorageError::NotReady)
    }

    fn write_blocks(&mut self, _block_id: usize, _buf: &[u8]) -> Result<(), StorageError> {
        Err(StorageError::NotReady)
    }
}
