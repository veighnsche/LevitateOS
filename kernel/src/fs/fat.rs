//! FAT32 Filesystem Backend
//!
//! TEAM_032: Uses embedded-sdmmc for FAT32 support on VirtIO block device.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use embedded_sdmmc::{
    Block, BlockCount, BlockDevice, BlockIdx, Mode, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};

use crate::block;

/// Error type for block device
#[derive(Debug, Clone, Copy)]
pub struct BlockError;

/// Adapter implementing embedded-sdmmc BlockDevice over our block driver
pub struct VirtioBlockDevice {
    num_blocks: BlockCount,
}

impl VirtioBlockDevice {
    pub fn new(size_bytes: u64) -> Self {
        let num_blocks = (size_bytes / 512) as u32;
        Self {
            num_blocks: BlockCount(num_blocks),
        }
    }
}

impl BlockDevice for VirtioBlockDevice {
    type Error = BlockError;

    fn read(&self, blocks: &mut [Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        for (i, block) in blocks.iter_mut().enumerate() {
            let block_id = start_block_idx.0 as usize + i;
            block::read_block(block_id, &mut block.contents);
        }
        Ok(())
    }

    fn write(&self, blocks: &[Block], start_block_idx: BlockIdx) -> Result<(), Self::Error> {
        for (i, block) in blocks.iter().enumerate() {
            let block_id = start_block_idx.0 as usize + i;
            block::write_block(block_id, &block.contents);
        }
        Ok(())
    }

    fn num_blocks(&self) -> Result<BlockCount, Self::Error> {
        Ok(self.num_blocks)
    }
}

/// Dummy time source
pub struct DummyTimeSource;

impl TimeSource for DummyTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 56,
            zero_indexed_month: 0,
            zero_indexed_day: 3,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

/// FAT32 disk size (16MB)
const DISK_SIZE_BYTES: u64 = 16 * 1024 * 1024;

/// Try to mount and list directory on FAT32
pub fn mount_and_list() -> Result<Vec<String>, &'static str> {
    let block_device = VirtioBlockDevice::new(DISK_SIZE_BYTES);
    let time_source = DummyTimeSource;
    let volume_mgr = VolumeManager::<_, _, 4, 4, 1>::new(block_device, time_source);

    let volume = volume_mgr
        .open_volume(VolumeIdx(0))
        .map_err(|_| "Failed to open FAT volume")?;

    let root_dir = volume
        .open_root_dir()
        .map_err(|_| "Failed to open root dir")?;

    let mut entries = Vec::new();
    let _ = root_dir.iterate_dir(|entry| {
        entries.push(entry.name.to_string());
    });

    Ok(entries)
}

/// Read file from FAT32
pub fn read_file(path: &str) -> Option<Vec<u8>> {
    let block_device = VirtioBlockDevice::new(DISK_SIZE_BYTES);
    let time_source = DummyTimeSource;
    let volume_mgr = VolumeManager::<_, _, 4, 4, 1>::new(block_device, time_source);

    let volume = volume_mgr.open_volume(VolumeIdx(0)).ok()?;
    let root_dir = volume.open_root_dir().ok()?;

    let file = root_dir.open_file_in_dir(path, Mode::ReadOnly).ok()?;

    let mut contents = Vec::new();
    let mut buf = [0u8; 512];
    let mut file = file;
    loop {
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => contents.extend_from_slice(&buf[..n]),
            Err(_) => return None,
        }
    }

    Some(contents)
}
