//! TEAM_202: Inode Implementation
//!
//! The inode is the core abstraction for files, directories, and other
//! filesystem objects in the VFS.

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use core::any::Any;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::error::VfsResult;
use super::ops::InodeOps;
use super::superblock::Superblock;
use crate::fs::mode;
use crate::syscall::Stat;

/// TEAM_202: Reference to an inode
pub type InodeRef = Arc<Inode>;

/// TEAM_202: Weak reference to an inode (for parent pointers)
pub type WeakInodeRef = Weak<Inode>;

/// TEAM_202: Inode — in-memory representation of a file/directory
///
/// Every file, directory, symlink, device, etc. is represented by an inode.
/// The inode contains metadata and a reference to filesystem-specific
/// operations and data.
pub struct Inode {
    /// Unique identifier within the filesystem
    pub ino: u64,
    /// Device ID of the filesystem containing this inode
    pub dev: u64,
    /// File type and permissions (S_IFMT | mode bits)
    pub mode: AtomicU32,
    /// Number of hard links
    pub nlink: AtomicU32,
    /// Owner user ID
    pub uid: AtomicU32,
    /// Owner group ID
    pub gid: AtomicU32,
    /// Device ID (for block/char devices)
    pub rdev: u64,
    /// Size in bytes
    pub size: AtomicU64,
    /// Block size for I/O
    pub blksize: u64,
    /// Access time (seconds since epoch)
    pub atime: AtomicU64,
    /// Modification time (seconds since epoch)
    pub mtime: AtomicU64,
    /// Status change time (seconds since epoch)
    pub ctime: AtomicU64,
    /// Filesystem-specific private data
    pub private: Box<dyn Any + Send + Sync>,
    /// Operations table for this inode
    pub ops: &'static dyn InodeOps,
    /// Reference to the containing superblock
    pub sb: Weak<dyn Superblock>,
}

// SAFETY: Inode fields are either atomic or immutable after creation
unsafe impl Send for Inode {}
unsafe impl Sync for Inode {}

impl Inode {
    /// TEAM_202: Create a new inode
    pub fn new(
        ino: u64,
        dev: u64,
        mode: u32,
        ops: &'static dyn InodeOps,
        sb: Weak<dyn Superblock>,
        private: Box<dyn Any + Send + Sync>,
    ) -> Self {
        Self {
            ino,
            dev,
            mode: AtomicU32::new(mode),
            nlink: AtomicU32::new(1),
            uid: AtomicU32::new(0),
            gid: AtomicU32::new(0),
            rdev: 0,
            size: AtomicU64::new(0),
            blksize: 4096,
            atime: AtomicU64::new(0),
            mtime: AtomicU64::new(0),
            ctime: AtomicU64::new(0),
            private,
            ops,
            sb,
        }
    }

    /// TEAM_202: Check if this is a regular file
    pub fn is_file(&self) -> bool {
        mode::is_reg(self.mode.load(Ordering::Relaxed))
    }

    /// TEAM_202: Check if this is a directory
    pub fn is_dir(&self) -> bool {
        mode::is_dir(self.mode.load(Ordering::Relaxed))
    }

    /// TEAM_202: Check if this is a symbolic link
    pub fn is_symlink(&self) -> bool {
        mode::is_lnk(self.mode.load(Ordering::Relaxed))
    }

    /// TEAM_202: Check if this is a character device
    pub fn is_chr(&self) -> bool {
        mode::is_chr(self.mode.load(Ordering::Relaxed))
    }

    /// TEAM_202: Check if this is a block device
    pub fn is_blk(&self) -> bool {
        mode::is_blk(self.mode.load(Ordering::Relaxed))
    }

    /// TEAM_202: Get file type from mode
    pub fn file_type(&self) -> u32 {
        mode::file_type(self.mode.load(Ordering::Relaxed))
    }

    /// TEAM_202: Get permission bits
    pub fn permissions(&self) -> u32 {
        mode::permissions(self.mode.load(Ordering::Relaxed))
    }

    /// TEAM_202: Convert to Stat structure
    /// TEAM_258: Use constructor for architecture independence
    pub fn to_stat(&self) -> Stat {
        let size = self.size.load(Ordering::Relaxed);
        Stat::from_inode_data(
            self.dev,
            self.ino,
            self.mode.load(Ordering::Relaxed),
            self.nlink.load(Ordering::Relaxed),
            self.uid.load(Ordering::Relaxed),
            self.gid.load(Ordering::Relaxed),
            self.rdev,
            size as i64,
            self.blksize as i32,
            ((size + 511) / 512) as i64,
            self.atime.load(Ordering::Relaxed) as i64,
            self.mtime.load(Ordering::Relaxed) as i64,
            self.ctime.load(Ordering::Relaxed) as i64,
        )
    }

    /// TEAM_202: Update access time
    pub fn touch_atime(&self) {
        // TODO: Get actual time from clock
        // For now, just increment
        self.atime.fetch_add(1, Ordering::Relaxed);
    }

    /// TEAM_202: Update modification time (also updates ctime)
    pub fn touch_mtime(&self) {
        let now = self.mtime.load(Ordering::Relaxed) + 1; // TODO: real time
        self.mtime.store(now, Ordering::Relaxed);
        self.ctime.store(now, Ordering::Relaxed);
    }

    /// TEAM_202: Update status change time
    pub fn touch_ctime(&self) {
        self.ctime.fetch_add(1, Ordering::Relaxed);
    }

    /// TEAM_202: Increment link count
    pub fn inc_nlink(&self) {
        self.nlink.fetch_add(1, Ordering::Relaxed);
        self.touch_ctime();
    }

    /// TEAM_202: Decrement link count, returns new count
    pub fn dec_nlink(&self) -> u32 {
        let old = self.nlink.fetch_sub(1, Ordering::Relaxed);
        self.touch_ctime();
        old - 1
    }

    /// TEAM_202: Get filesystem-specific data
    pub fn private<T: 'static>(&self) -> Option<&T> {
        self.private.downcast_ref::<T>()
    }

    // ========================================================================
    // Operations delegation
    // ========================================================================

    /// Look up a child in this directory
    pub fn lookup(&self, name: &str) -> VfsResult<Arc<Inode>> {
        self.ops.lookup(self, name)
    }

    /// Create a file in this directory
    pub fn create(&self, name: &str, mode: u32) -> VfsResult<Arc<Inode>> {
        self.ops.create(self, name, mode)
    }

    /// Create a directory in this directory
    pub fn mkdir(&self, name: &str, mode: u32) -> VfsResult<Arc<Inode>> {
        self.ops.mkdir(self, name, mode)
    }

    /// Remove a file from this directory
    pub fn unlink(&self, name: &str) -> VfsResult<()> {
        self.ops.unlink(self, name)
    }

    /// Remove a directory from this directory
    pub fn rmdir(&self, name: &str) -> VfsResult<()> {
        self.ops.rmdir(self, name)
    }

    /// Read from this inode
    pub fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        self.ops.read(self, offset, buf)
    }

    /// Write to this inode
    pub fn write(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        self.ops.write(self, offset, buf)
    }

    /// Truncate this inode
    pub fn truncate(&self, size: u64) -> VfsResult<()> {
        self.ops.truncate(self, size)
    }

    /// Read symlink target
    pub fn readlink(&self) -> VfsResult<alloc::string::String> {
        self.ops.readlink(self)
    }
}

impl core::fmt::Debug for Inode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Inode")
            .field("ino", &self.ino)
            .field("dev", &self.dev)
            .field("mode", &self.mode.load(Ordering::Relaxed))
            .field("nlink", &self.nlink.load(Ordering::Relaxed))
            .field("size", &self.size.load(Ordering::Relaxed))
            .finish()
    }
}
