//! VirtIO Block Device Driver
//!
//! TEAM_032: Updated for virtio-drivers v0.12.0
//! - Uses StaticMmioTransport for 'static lifetime compatibility
//!
//! TEAM_150: Converted panics to Result types for proper error handling

use crate::virtio::{StaticMmioTransport, VirtioHal};
use los_utils::Mutex;
use virtio_drivers::device::blk::VirtIOBlk;

use los_error::define_kernel_error;

define_kernel_error! {
    /// TEAM_150: Block device error type with error codes.
    /// Error codes in range 0x06xx (Block subsystem).
    /// TEAM_155: Migrated to define_kernel_error! macro.
    pub enum BlockError(0x06) {
        /// Device not initialized
        NotInitialized = 0x01 => "Block device not initialized",
        /// Read operation failed
        ReadFailed = 0x02 => "Block read failed",
        /// Write operation failed
        WriteFailed = 0x03 => "Block write failed",
        /// Invalid buffer size
        InvalidBufferSize = 0x04 => "Invalid buffer size",
    }
}

// TEAM_032: Use StaticMmioTransport (MmioTransport<'static>) for static storage
static BLOCK_DEVICE: Mutex<Option<VirtIOBlk<VirtioHal, StaticMmioTransport>>> =
    Mutex::new(None);

pub const BLOCK_SIZE: usize = 512;

pub fn init(transport: StaticMmioTransport) {
    log::info!("Initializing VirtIO Block...");
    match VirtIOBlk::<VirtioHal, StaticMmioTransport>::new(transport) {
        Ok(blk) => {
            log::info!("VirtIO Block initialized successfully.");
            *BLOCK_DEVICE.lock() = Some(blk);
        }
        Err(e) => log::error!("Failed to init VirtIO Block: {:?}", e),
    }
}

/// TEAM_150: Read a single block from the device.
///
/// # Arguments
/// * `block_id` - Block number to read
/// * `buf` - Buffer to read into (must be exactly 512 bytes)
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(BlockError)` on failure
pub fn read_block(block_id: usize, buf: &mut [u8]) -> Result<(), BlockError> {
    if buf.len() != BLOCK_SIZE {
        return Err(BlockError::InvalidBufferSize);
    }
    let mut dev = BLOCK_DEVICE.lock();
    if let Some(ref mut blk) = *dev {
        let blk: &mut VirtIOBlk<VirtioHal, StaticMmioTransport> = blk;
        blk.read_blocks(block_id, buf)
            .map_err(|_| BlockError::ReadFailed)
    } else {
        Err(BlockError::NotInitialized)
    }
}

/// TEAM_150: Write a single block to the device.
///
/// # Arguments
/// * `block_id` - Block number to write
/// * `buf` - Buffer to write from (must be exactly 512 bytes)
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(BlockError)` on failure
pub fn write_block(block_id: usize, buf: &[u8]) -> Result<(), BlockError> {
    if buf.len() != BLOCK_SIZE {
        return Err(BlockError::InvalidBufferSize);
    }
    let mut dev = BLOCK_DEVICE.lock();
    if let Some(ref mut blk) = *dev {
        let blk: &mut VirtIOBlk<VirtioHal, StaticMmioTransport> = blk;
        blk.write_blocks(block_id, buf)
            .map_err(|_| BlockError::WriteFailed)
    } else {
        Err(BlockError::NotInitialized)
    }
}
