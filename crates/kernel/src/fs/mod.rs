//! Virtual Filesystem Layer
//!
//! TEAM_032: Provides unified filesystem access with multiple backend support.
//! - FAT32 via embedded-sdmmc (boot partition)
//! - ext4 via ext4-view (root partition, read-only)
//!
//! Note: Some functions are kept for future VFS integration.
#![allow(dead_code)]

extern crate alloc;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use los_utils::Mutex;

pub mod ext4;
pub mod fat;
pub mod mode;
pub mod mount;
pub mod path;
pub mod pipe;
pub mod tmpfs;
pub mod tty;
pub mod vfs;

use los_error::define_kernel_error;

define_kernel_error! {
    /// TEAM_152: Filesystem error type with error codes (0x05xx) per unified error system plan.
    /// TEAM_155: Migrated to define_kernel_error! macro.
    pub enum FsError(0x05) {
        /// Failed to open volume
        VolumeOpen = 0x01 => "Failed to open volume",
        /// Failed to open directory
        DirOpen = 0x02 => "Failed to open directory",
        /// Failed to open file
        FileOpen = 0x03 => "Failed to open file",
        /// Read error
        ReadError = 0x04 => "Read error",
        /// Write error
        WriteError = 0x05 => "Write error",
        /// Filesystem not mounted
        NotMounted = 0x06 => "Filesystem not mounted",
        /// Block device error
        BlockError(crate::block::BlockError) = 0x07 => "Block device error",
    }
}

impl From<crate::block::BlockError> for FsError {
    fn from(e: crate::block::BlockError) -> Self {
        FsError::BlockError(e)
    }
}

/// Filesystem type enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FsType {
    Fat32,
    Ext4,
}

/// Mount status
static FAT32_MOUNTED: Mutex<bool> = Mutex::new(false);
static EXT4_MOUNTED: Mutex<bool> = Mutex::new(false);

/// Initialize filesystems
///
/// Attempts to mount FAT32 boot partition.
/// ext4 root partition is optional and can be mounted later.
pub mod initramfs;

pub static INITRAMFS: Mutex<Option<Arc<initramfs::InitramfsSuperblock>>> = Mutex::new(None);

pub fn init() -> Result<(), FsError> {
    // TEAM_152: Updated to use FsError
    // Mount FAT32 boot partition
    match fat::mount_and_list() {
        Ok(entries) => {
            *FAT32_MOUNTED.lock() = true;
            crate::verbose!("FAT32 mounted. Root contains {} entries.", entries.len());
            for _entry in entries.iter().take(5) {
                // Entries logged via verbose macro if enabled
            }
            Ok(())
        }
        Err(e) => {
            crate::println!("ERROR: Failed to mount FAT32: {}", e);
            Err(e)
        }
    }
}

/// Initialize ext4 filesystem (optional second disk)
pub fn init_ext4() -> Result<(), FsError> {
    // TEAM_152: Updated to use FsError
    match ext4::mount_and_list() {
        Ok(entries) => {
            *EXT4_MOUNTED.lock() = true;
            crate::verbose!("ext4 mounted. Root contains {} entries.", entries.len());
            for _entry in entries.iter().take(5) {
                // Entries logged via verbose macro if enabled
            }
            Ok(())
        }
        Err(e) => {
            crate::verbose!("ext4 not available: {}", e);
            Err(e)
        }
    }
}

/// Read file from mounted filesystem
///
/// Tries FAT32 first, then ext4 if available.
pub fn read_file(path: &str) -> Option<Vec<u8>> {
    // Try FAT32 first
    if *FAT32_MOUNTED.lock() {
        if let Some(data) = fat::read_file(path) {
            return Some(data);
        }
    }

    // Try ext4
    if *EXT4_MOUNTED.lock() {
        if let Some(data) = ext4::read_file(path) {
            return Some(data);
        }
    }

    None
}

/// TEAM_065: List directory contents with proper error propagation (Rule 6)
/// TEAM_152: Updated to use FsError
pub fn list_dir(_path: &str, fs_type: FsType) -> Result<Vec<String>, FsError> {
    match fs_type {
        FsType::Fat32 => fat::mount_and_list(),
        FsType::Ext4 => ext4::mount_and_list(),
    }
}
