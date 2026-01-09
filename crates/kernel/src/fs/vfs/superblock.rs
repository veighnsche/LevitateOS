//! TEAM_202: Superblock Implementation
//!
//! The superblock represents a mounted filesystem instance.

extern crate alloc;

use alloc::sync::Arc;

use super::error::VfsResult;
use super::inode::Inode;

/// TEAM_202: Reference to a superblock
pub type SuperblockRef = Arc<dyn Superblock>;

/// TEAM_202: Filesystem statistics
#[derive(Clone, Copy, Debug, Default)]
pub struct StatFs {
    /// Filesystem type magic number
    pub f_type: u64,
    /// Optimal transfer block size
    pub f_bsize: u64,
    /// Total data blocks in filesystem
    pub f_blocks: u64,
    /// Free blocks in filesystem
    pub f_bfree: u64,
    /// Free blocks available to unprivileged user
    pub f_bavail: u64,
    /// Total file nodes in filesystem
    pub f_files: u64,
    /// Free file nodes in filesystem
    pub f_ffree: u64,
    /// Maximum length of filenames
    pub f_namelen: u64,
    /// Fragment size
    pub f_frsize: u64,
    /// Mount flags
    pub f_flags: u64,
}

/// TEAM_202: Superblock Trait
///
/// Represents a mounted filesystem instance. Each mount creates a new
/// superblock that provides access to the filesystem's root and operations.
pub trait Superblock: Send + Sync {
    /// Get the root inode of this filesystem
    fn root(&self) -> Arc<Inode>;

    /// Get filesystem statistics
    fn statfs(&self) -> VfsResult<StatFs> {
        Ok(StatFs::default())
    }

    /// Sync filesystem buffers to disk
    fn sync(&self) -> VfsResult<()> {
        Ok(())
    }

    /// Get filesystem type name
    fn fs_type(&self) -> &'static str;

    /// Called when the filesystem is unmounted
    fn unmount(&self) -> VfsResult<()> {
        Ok(())
    }

    /// Allocate a new inode number
    fn alloc_ino(&self) -> u64;

    /// Cast to Any for downcasting
    fn as_any(&self) -> &dyn core::any::Any;
}
