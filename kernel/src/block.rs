//! VirtIO Block Device Driver
//!
//! TEAM_032: Updated for virtio-drivers v0.12.0
//! - Uses StaticMmioTransport for 'static lifetime compatibility

use crate::virtio::{StaticMmioTransport, VirtioHal};
use levitate_utils::Spinlock;
use virtio_drivers::device::blk::VirtIOBlk;

// TEAM_032: Use StaticMmioTransport (MmioTransport<'static>) for static storage
static BLOCK_DEVICE: Spinlock<Option<VirtIOBlk<VirtioHal, StaticMmioTransport>>> =
    Spinlock::new(None);

pub const BLOCK_SIZE: usize = 512;

pub fn init(transport: StaticMmioTransport) {
    crate::verbose!("Initializing Block device...");
    match VirtIOBlk::<VirtioHal, StaticMmioTransport>::new(transport) {
        Ok(blk) => {
            crate::verbose!("VirtIO Block initialized successfully.");
            *BLOCK_DEVICE.lock() = Some(blk);
        }
        Err(e) => crate::println!("Failed to init VirtIO Block: {:?}", e),
    }
}

pub fn read_block(block_id: usize, buf: &mut [u8]) {
    assert_eq!(
        buf.len(),
        BLOCK_SIZE,
        "Buffer size must be exactly 512 bytes"
    );
    let mut dev = BLOCK_DEVICE.lock();
    if let Some(ref mut blk) = *dev {
        match blk.read_blocks(block_id, buf) {
            Ok(_) => {}
            Err(e) => panic!("Failed to read block {}: {:?}", block_id, e),
        }
    } else {
        panic!("Block device not initialized");
    }
}

pub fn write_block(block_id: usize, buf: &[u8]) {
    assert_eq!(
        buf.len(),
        BLOCK_SIZE,
        "Buffer size must be exactly 512 bytes"
    );
    let mut dev = BLOCK_DEVICE.lock();
    if let Some(ref mut blk) = *dev {
        match blk.write_blocks(block_id, buf) {
            Ok(_) => {}
            Err(e) => panic!("Failed to write block {}: {:?}", block_id, e),
        }
    } else {
        panic!("Block device not initialized");
    }
}
