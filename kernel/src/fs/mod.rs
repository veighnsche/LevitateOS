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
use alloc::vec::Vec;
use levitate_utils::Spinlock;

pub mod ext4;
pub mod fat;

/// Filesystem type enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FsType {
    Fat32,
    Ext4,
}

/// Mount status
static FAT32_MOUNTED: Spinlock<bool> = Spinlock::new(false);
static EXT4_MOUNTED: Spinlock<bool> = Spinlock::new(false);

/// Initialize filesystems
///
/// Attempts to mount FAT32 boot partition.
/// ext4 root partition is optional and can be mounted later.
pub mod initramfs;

pub fn init() -> Result<(), &'static str> {
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
pub fn init_ext4() -> Result<(), &'static str> {
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

/// List directory contents
pub fn list_dir(_path: &str, fs_type: FsType) -> Vec<String> {
    match fs_type {
        FsType::Fat32 => fat::mount_and_list().unwrap_or_default(),
        FsType::Ext4 => ext4::mount_and_list().unwrap_or_default(),
    }
}
