//! ext4 Filesystem Backend
//!
//! TEAM_032: Uses ext4-view for read-only ext4 support.
//! This is designed for the root filesystem partition.

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use ext4_view::{Ext4, Ext4Read};

use crate::block;

/// Block device adapter for ext4-view
///
/// ext4-view requires Ext4Read trait which reads bytes at offset.
pub struct Ext4BlockDevice {
    size_bytes: u64,
}

impl Ext4BlockDevice {
    pub fn new(size_bytes: u64) -> Self {
        Self { size_bytes }
    }
}

impl Ext4Read for Ext4BlockDevice {
    // TEAM_032: Ext4Read::read requires &mut self and returns Box<dyn Error>
    fn read(
        &mut self,
        offset: u64,
        data: &mut [u8],
    ) -> Result<(), Box<dyn core::error::Error + Send + Sync>> {
        // Calculate block-aligned reads
        let block_size = 512u64;
        let start_block = offset / block_size;
        let offset_in_block = (offset % block_size) as usize;

        let mut buf = [0u8; 512];
        let mut remaining = data.len();
        let mut data_offset = 0;
        let mut current_block = start_block;
        let mut block_offset = offset_in_block;

        while remaining > 0 {
            // TEAM_150: Propagate block errors instead of panicking
            block::read_block(current_block as usize, &mut buf)
                .map_err(|e| alloc::boxed::Box::new(e) as Box<dyn core::error::Error + Send + Sync>)?;

            let bytes_to_copy = (512 - block_offset).min(remaining);
            data[data_offset..data_offset + bytes_to_copy]
                .copy_from_slice(&buf[block_offset..block_offset + bytes_to_copy]);

            data_offset += bytes_to_copy;
            remaining -= bytes_to_copy;
            current_block += 1;
            block_offset = 0; // After first block, always start at 0
        }

        Ok(())
    }
}

/// ext4 disk size (32MB - larger for root filesystem)
const EXT4_DISK_SIZE_BYTES: u64 = 32 * 1024 * 1024;

use super::FsError;

/// Try to mount ext4 filesystem and list root directory
/// TEAM_152: Updated to use FsError
pub fn mount_and_list() -> Result<Vec<String>, FsError> {
    let block_device = Ext4BlockDevice::new(EXT4_DISK_SIZE_BYTES);

    // Box the device for Ext4::load
    let fs = Ext4::load(Box::new(block_device)).map_err(|_| FsError::VolumeOpen)?;

    let mut entries = Vec::new();
    if let Ok(dir) = fs.read_dir("/") {
        for entry in dir {
            if let Ok(e) = entry {
                // TEAM_032: DirEntryName::as_str() returns Result
                if let Ok(name) = e.file_name().as_str() {
                    entries.push(String::from(name));
                }
            }
        }
    }

    Ok(entries)
}

/// Read file from ext4 filesystem
pub fn read_file(path: &str) -> Option<Vec<u8>> {
    let block_device = Ext4BlockDevice::new(EXT4_DISK_SIZE_BYTES);
    let fs = Ext4::load(Box::new(block_device)).ok()?;

    fs.read(path).ok()
}
