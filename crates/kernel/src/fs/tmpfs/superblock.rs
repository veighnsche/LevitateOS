//! TEAM_194: Tmpfs Superblock
//!
//! TEAM_208: Refactored from tmpfs.rs into separate module.

extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use los_utils::Mutex;

use crate::fs::mode;
use crate::fs::vfs::error::VfsResult;
use crate::fs::vfs::inode::Inode;
use crate::fs::vfs::ops::InodeOps;
use crate::fs::vfs::superblock::{StatFs, Superblock};

use super::node::{MAX_TOTAL_SIZE, TmpfsNode, TmpfsNodeType};
use super::{TMPFS_DIR_OPS, TMPFS_FILE_OPS, TMPFS_SYMLINK_OPS};

/// TEAM_194: The tmpfs filesystem state
pub struct Tmpfs {
    /// Root directory node
    pub(super) root: Arc<Mutex<TmpfsNode>>,
    /// Next inode number
    pub(super) next_ino: AtomicU64,
    /// Total bytes used
    pub(super) bytes_used: AtomicUsize,
    /// VFS root inode (cached)
    pub(super) vfs_root: Mutex<Option<Arc<Inode>>>,
}

impl Tmpfs {
    /// TEAM_194: Create a new tmpfs instance
    pub fn new() -> Self {
        Self {
            root: Arc::new(Mutex::new(TmpfsNode::new_dir(1))),
            next_ino: AtomicU64::new(2),
            bytes_used: AtomicUsize::new(0),
            vfs_root: Mutex::new(None),
        }
    }

    /// TEAM_194: Allocate a new inode number
    pub(super) fn alloc_ino(&self) -> u64 {
        self.next_ino.fetch_add(1, Ordering::SeqCst)
    }

    /// TEAM_194: Get total bytes used
    pub fn bytes_used(&self) -> usize {
        self.bytes_used.load(Ordering::SeqCst)
    }

    /// TEAM_203: Convert a TmpfsNode to a VFS Inode
    pub fn make_inode(
        &self,
        node: Arc<Mutex<TmpfsNode>>,
        sb: Weak<dyn Superblock>,
    ) -> Arc<Inode> {
        let node_locked = node.lock();
        let ino = node_locked.ino;
        let node_type = node_locked.node_type;
        let file_mode = match node_type {
            TmpfsNodeType::File => mode::S_IFREG | 0o666,
            TmpfsNodeType::Directory => mode::S_IFDIR | 0o777,
            TmpfsNodeType::Symlink => mode::S_IFLNK | 0o777,
        };
        let size = node_locked.data.len() as u64;
        let atime = node_locked.atime;
        let mtime = node_locked.mtime;
        let ctime = node_locked.ctime;
        drop(node_locked);

        let ops: &'static dyn InodeOps = match node_type {
            TmpfsNodeType::File => &TMPFS_FILE_OPS,
            TmpfsNodeType::Directory => &TMPFS_DIR_OPS,
            TmpfsNodeType::Symlink => &TMPFS_SYMLINK_OPS,
        };

        let inode = Arc::new(Inode::new(
            ino,
            0, // dev id
            file_mode,
            ops,
            sb,
            Box::new(node),
        ));

        inode.size.store(size, Ordering::Relaxed);
        inode.atime.store(atime, Ordering::Relaxed);
        inode.mtime.store(mtime, Ordering::Relaxed);
        inode.ctime.store(ctime, Ordering::Relaxed);

        inode
    }
}

impl Superblock for Tmpfs {
    fn root(&self) -> Arc<Inode> {
        let root_cache = self.vfs_root.lock();
        if let Some(ref root) = *root_cache {
            return Arc::clone(root);
        }

        // We need a Weak<dyn Superblock> to self.
        // This is tricky for Tmpfs because it's usually inside an Arc.
        // For now, let's assume we can get it from the mount system later or just use Dummy Weak.
        // Actually, the caller of root() usually has the Arc<Superblock>.
        // But we are implementing root(&self).

        // Let's use a Dummy Weak for now if we don't have a way to get self-arc.
        // Or we can initialize it during mount.
        panic!("Tmpfs::root called before vfs_root was initialized");
    }

    fn statfs(&self) -> VfsResult<StatFs> {
        Ok(StatFs {
            f_type: 0x01021994, // Tmpfs magic
            f_bsize: 4096,
            f_blocks: (MAX_TOTAL_SIZE / 4096) as u64,
            f_bfree: ((MAX_TOTAL_SIZE - self.bytes_used()) / 4096) as u64,
            f_bavail: ((MAX_TOTAL_SIZE - self.bytes_used()) / 4096) as u64,
            f_files: 1024, // Arbitrary
            f_ffree: 1024,
            f_namelen: 255,
            f_frsize: 4096,
            f_flags: 0,
        })
    }

    fn fs_type(&self) -> &'static str {
        "tmpfs"
    }

    fn alloc_ino(&self) -> u64 {
        self.next_ino.fetch_add(1, Ordering::SeqCst)
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl Default for Tmpfs {
    fn default() -> Self {
        Self::new()
    }
}
