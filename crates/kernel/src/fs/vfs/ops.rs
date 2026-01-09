//! TEAM_202: VFS Operation Traits
//!
//! Defines the traits that filesystems must implement to integrate with the VFS.

extern crate alloc;

use alloc::string::String;
use alloc::sync::Arc;

use super::error::{VfsError, VfsResult};
use super::inode::Inode;
use super::file::File;
use crate::syscall::Stat;

/// TEAM_202: Directory entry returned by readdir
#[derive(Clone, Debug)]
pub struct DirEntry {
    /// Inode number
    pub ino: u64,
    /// Entry name
    pub name: String,
    /// File type (from mode constants)
    pub file_type: u32,
}

/// TEAM_202: Attributes that can be set on an inode
#[derive(Clone, Debug, Default)]
pub struct SetAttr {
    /// New mode (permissions), if Some
    pub mode: Option<u32>,
    /// New owner UID, if Some
    pub uid: Option<u32>,
    /// New owner GID, if Some
    pub gid: Option<u32>,
    /// New size (for truncate), if Some
    pub size: Option<u64>,
    /// New access time, if Some
    pub atime: Option<u64>,
    /// New modification time, if Some
    pub mtime: Option<u64>,
}

/// TEAM_202: Inode Operations Trait
///
/// Filesystems implement this trait to provide inode-level operations.
/// Not all operations need to be implemented - default implementations
/// return `VfsError::NotSupported`.
pub trait InodeOps: Send + Sync {
    // ========================================================================
    // Directory Operations
    // ========================================================================

    /// Look up a child by name in this directory
    fn lookup(&self, _inode: &Inode, _name: &str) -> VfsResult<Arc<Inode>> {
        Err(VfsError::NotSupported)
    }

    /// Create a regular file in this directory
    fn create(&self, _inode: &Inode, _name: &str, _mode: u32) -> VfsResult<Arc<Inode>> {
        Err(VfsError::NotSupported)
    }

    /// Create a directory in this directory
    fn mkdir(&self, _inode: &Inode, _name: &str, _mode: u32) -> VfsResult<Arc<Inode>> {
        Err(VfsError::NotSupported)
    }

    /// Remove a file from this directory
    fn unlink(&self, _inode: &Inode, _name: &str) -> VfsResult<()> {
        Err(VfsError::NotSupported)
    }

    /// Remove a directory from this directory
    fn rmdir(&self, _inode: &Inode, _name: &str) -> VfsResult<()> {
        Err(VfsError::NotSupported)
    }

    /// Create a symbolic link
    fn symlink(&self, _inode: &Inode, _name: &str, _target: &str) -> VfsResult<Arc<Inode>> {
        Err(VfsError::NotSupported)
    }

    /// Create a hard link
    fn link(&self, _inode: &Inode, _name: &str, _target: &Inode) -> VfsResult<()> {
        Err(VfsError::NotSupported)
    }

    /// Rename/move an entry
    fn rename(
        &self,
        _old_dir: &Inode,
        _old_name: &str,
        _new_dir: &Inode,
        _new_name: &str,
    ) -> VfsResult<()> {
        Err(VfsError::NotSupported)
    }

    // ========================================================================
    // File Operations
    // ========================================================================

    /// Read data from this inode
    fn read(&self, _inode: &Inode, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
        Err(VfsError::NotSupported)
    }

    /// Write data to this inode
    fn write(&self, _inode: &Inode, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::NotSupported)
    }

    /// Truncate or extend the file to the specified size
    fn truncate(&self, _inode: &Inode, _size: u64) -> VfsResult<()> {
        Err(VfsError::NotSupported)
    }

    // ========================================================================
    // Symlink Operations
    // ========================================================================

    /// Read the target of a symbolic link
    fn readlink(&self, _inode: &Inode) -> VfsResult<String> {
        Err(VfsError::NotSupported)
    }

    // ========================================================================
    // Directory Iteration
    // ========================================================================

    /// Read a directory entry at the given offset
    /// Returns None when there are no more entries
    fn readdir(&self, _inode: &Inode, _offset: usize) -> VfsResult<Option<DirEntry>> {
        Err(VfsError::NotSupported)
    }

    // ========================================================================
    // Metadata Operations
    // ========================================================================

    /// Get inode attributes (stat)
    fn getattr(&self, inode: &Inode) -> VfsResult<Stat> {
        // Default implementation builds Stat from inode fields
        Ok(inode.to_stat())
    }

    /// Set inode attributes
    fn setattr(&self, _inode: &Inode, _attr: &SetAttr) -> VfsResult<()> {
        Err(VfsError::NotSupported)
    }
}

/// TEAM_202: File Operations Trait
///
/// Operations on an open file handle. These may differ from inode operations
/// for special files (devices, sockets, etc.).
pub trait FileOps: Send + Sync {
    /// Read from the file at the current offset
    fn read(&self, file: &File, buf: &mut [u8]) -> VfsResult<usize>;

    /// Write to the file at the current offset
    fn write(&self, file: &File, buf: &[u8]) -> VfsResult<usize>;

    /// Seek to a new position
    fn seek(&self, file: &File, offset: i64, whence: SeekWhence) -> VfsResult<u64> {
        use core::sync::atomic::Ordering;
        
        let current = file.offset.load(Ordering::Relaxed);
        let size = file.inode.size.load(Ordering::Relaxed);
        
        let new_offset = match whence {
            SeekWhence::Set => {
                if offset < 0 {
                    return Err(VfsError::InvalidArgument);
                }
                offset as u64
            }
            SeekWhence::Cur => {
                if offset < 0 {
                    current.checked_sub((-offset) as u64)
                        .ok_or(VfsError::InvalidArgument)?
                } else {
                    current.checked_add(offset as u64)
                        .ok_or(VfsError::InvalidArgument)?
                }
            }
            SeekWhence::End => {
                if offset < 0 {
                    size.checked_sub((-offset) as u64)
                        .ok_or(VfsError::InvalidArgument)?
                } else {
                    size.checked_add(offset as u64)
                        .ok_or(VfsError::InvalidArgument)?
                }
            }
        };
        
        file.offset.store(new_offset, Ordering::Relaxed);
        Ok(new_offset)
    }

    /// Poll for events (for select/poll/epoll)
    fn poll(&self, _file: &File) -> PollEvents {
        PollEvents::empty()
    }

    /// Device-specific control operation
    fn ioctl(&self, _file: &File, _cmd: u32, _arg: usize) -> VfsResult<i32> {
        Err(VfsError::NotSupported)
    }

    /// Flush file buffers
    fn flush(&self, _file: &File) -> VfsResult<()> {
        Ok(())
    }

    /// Release (close) the file
    fn release(&self, _file: &File) -> VfsResult<()> {
        Ok(())
    }
}

/// TEAM_202: Seek origin
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SeekWhence {
    /// Seek from beginning of file
    Set = 0,
    /// Seek from current position
    Cur = 1,
    /// Seek from end of file
    End = 2,
}

impl SeekWhence {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(SeekWhence::Set),
            1 => Some(SeekWhence::Cur),
            2 => Some(SeekWhence::End),
            _ => None,
        }
    }
}

/// TEAM_202: Poll events bitmask
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PollEvents(u32);

impl PollEvents {
    pub const POLLIN: u32 = 0x0001;
    pub const POLLOUT: u32 = 0x0004;
    pub const POLLERR: u32 = 0x0008;
    pub const POLLHUP: u32 = 0x0010;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn readable() -> Self {
        Self(Self::POLLIN)
    }

    pub const fn writable() -> Self {
        Self(Self::POLLOUT)
    }

    pub fn is_readable(&self) -> bool {
        self.0 & Self::POLLIN != 0
    }

    pub fn is_writable(&self) -> bool {
        self.0 & Self::POLLOUT != 0
    }
}
