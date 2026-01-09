//! TEAM_202: File (Open File Handle) Implementation
//!
//! A File represents an open file descriptor, containing the inode
//! reference, current offset, and open flags.

extern crate alloc;

use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

use super::error::{VfsError, VfsResult};
use super::inode::Inode;
use super::ops::{FileOps, SeekWhence};

/// TEAM_202: Reference to an open file
pub type FileRef = Arc<File>;

/// TEAM_202: Open file flags
#[derive(Clone, Copy, Debug)]
pub struct OpenFlags(u32);

impl OpenFlags {
    // Access modes (mutually exclusive)
    pub const O_RDONLY: u32 = 0;
    pub const O_WRONLY: u32 = 1;
    pub const O_RDWR: u32 = 2;
    pub const O_ACCMODE: u32 = 3;

    // File creation flags
    pub const O_CREAT: u32 = 0o100;
    pub const O_EXCL: u32 = 0o200;
    pub const O_TRUNC: u32 = 0o1000;

    // File status flags
    pub const O_APPEND: u32 = 0o2000;
    pub const O_NONBLOCK: u32 = 0o4000;
    pub const O_DIRECTORY: u32 = 0o200000;
    pub const O_NOFOLLOW: u32 = 0o400000;
    pub const O_CLOEXEC: u32 = 0o2000000;

    pub const fn new(flags: u32) -> Self {
        Self(flags)
    }

    pub const fn bits(&self) -> u32 {
        self.0
    }

    pub fn access_mode(&self) -> u32 {
        self.0 & Self::O_ACCMODE
    }

    pub fn is_readable(&self) -> bool {
        let mode = self.access_mode();
        mode == Self::O_RDONLY || mode == Self::O_RDWR
    }

    pub fn is_writable(&self) -> bool {
        let mode = self.access_mode();
        mode == Self::O_WRONLY || mode == Self::O_RDWR
    }

    pub fn is_create(&self) -> bool {
        self.0 & Self::O_CREAT != 0
    }

    pub fn is_exclusive(&self) -> bool {
        self.0 & Self::O_EXCL != 0
    }

    pub fn is_truncate(&self) -> bool {
        self.0 & Self::O_TRUNC != 0
    }

    pub fn is_append(&self) -> bool {
        self.0 & Self::O_APPEND != 0
    }

    pub fn is_nonblocking(&self) -> bool {
        self.0 & Self::O_NONBLOCK != 0
    }

    pub fn is_directory(&self) -> bool {
        self.0 & Self::O_DIRECTORY != 0
    }

    pub fn is_nofollow(&self) -> bool {
        self.0 & Self::O_NOFOLLOW != 0
    }

    pub fn is_cloexec(&self) -> bool {
        self.0 & Self::O_CLOEXEC != 0
    }
}

impl Default for OpenFlags {
    fn default() -> Self {
        Self(Self::O_RDONLY)
    }
}

// SeekWhence is in ops module

/// TEAM_202: Open File Handle
///
/// Represents an open file. Multiple File structs can point to the same
/// inode (e.g., after dup() or fork()). Each has its own offset and flags.
pub struct File {
    /// The inode this file refers to
    pub inode: Arc<Inode>,
    /// Current read/write position
    pub offset: AtomicU64,
    /// Open flags
    pub flags: OpenFlags,
    /// File-specific operations (may differ from inode ops for devices)
    pub ops: Option<&'static dyn FileOps>,
}

impl File {
    /// TEAM_202: Create a new open file
    pub fn new(inode: Arc<Inode>, flags: OpenFlags) -> Self {
        Self {
            inode,
            offset: AtomicU64::new(0),
            flags,
            ops: None,
        }
    }

    /// TEAM_202: Create with custom file operations
    pub fn with_ops(inode: Arc<Inode>, flags: OpenFlags, ops: &'static dyn FileOps) -> Self {
        Self {
            inode,
            offset: AtomicU64::new(0),
            flags,
            ops: Some(ops),
        }
    }

    /// TEAM_202: Read from the file
    pub fn read(&self, buf: &mut [u8]) -> VfsResult<usize> {
        if !self.flags.is_readable() {
            return Err(VfsError::BadFd);
        }

        // Use custom ops if available, otherwise use inode ops
        if let Some(ops) = self.ops {
            return ops.read(self, buf);
        }

        let offset = self.offset.load(Ordering::Relaxed);
        let n = self.inode.read(offset, buf)?;
        self.offset.fetch_add(n as u64, Ordering::Relaxed);
        self.inode.touch_atime();
        Ok(n)
    }

    /// TEAM_202: Write to the file
    pub fn write(&self, buf: &[u8]) -> VfsResult<usize> {
        if !self.flags.is_writable() {
            return Err(VfsError::BadFd);
        }

        // Use custom ops if available, otherwise use inode ops
        if let Some(ops) = self.ops {
            return ops.write(self, buf);
        }

        let offset = if self.flags.is_append() {
            self.inode.size.load(Ordering::Relaxed)
        } else {
            self.offset.load(Ordering::Relaxed)
        };

        let n = self.inode.write(offset, buf)?;
        self.offset.store(offset + n as u64, Ordering::Relaxed);
        self.inode.touch_mtime();
        Ok(n)
    }

    /// TEAM_202: Seek to a new position
    pub fn seek(&self, offset: i64, whence: SeekWhence) -> VfsResult<u64> {
        if let Some(ops) = self.ops {
            return ops.seek(self, offset, whence);
        }

        let current = self.offset.load(Ordering::Relaxed);
        let size = self.inode.size.load(Ordering::Relaxed);

        let new_offset = match whence {
            SeekWhence::Set => {
                if offset < 0 {
                    return Err(VfsError::InvalidArgument);
                }
                offset as u64
            }
            SeekWhence::Cur => {
                if offset < 0 {
                    current
                        .checked_sub((-offset) as u64)
                        .ok_or(VfsError::InvalidArgument)?
                } else {
                    current
                        .checked_add(offset as u64)
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

        self.offset.store(new_offset, Ordering::Relaxed);
        Ok(new_offset)
    }

    /// TEAM_202: Get current offset
    pub fn tell(&self) -> u64 {
        self.offset.load(Ordering::Relaxed)
    }

    /// TEAM_202: Flush file buffers
    pub fn flush(&self) -> VfsResult<()> {
        if let Some(ops) = self.ops {
            ops.flush(self)
        } else {
            Ok(())
        }
    }
}

impl core::fmt::Debug for File {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("File")
            .field("inode", &self.inode.ino)
            .field("offset", &self.offset.load(Ordering::Relaxed))
            .field("flags", &self.flags)
            .finish()
    }
}
