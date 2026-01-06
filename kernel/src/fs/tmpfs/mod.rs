//! TEAM_194: Tmpfs — In-memory writable filesystem for LevitateOS.
//!
//! Provides a writable scratch space at `/tmp` for levbox utilities.
//!
//! Design decisions (from phase-2.md):
//! - Mount point: `/tmp` only
//! - Max file size: 16MB
//! - Max total size: 64MB
//! - Locking: Global lock
//! - Hard links/symlinks: Deferred (EOPNOTSUPP)
//!
//! TEAM_208: Refactored into submodules for maintainability:
//! - `node.rs` - TmpfsNode, TmpfsNodeType, TmpfsError, constants
//! - `superblock.rs` - Tmpfs struct, Superblock impl
//! - `file_ops.rs` - TmpfsFileOps (InodeOps for files)
//! - `dir_ops.rs` - TmpfsDirOps (InodeOps for directories)
//! - `symlink_ops.rs` - TmpfsSymlinkOps (InodeOps for symlinks)

extern crate alloc;

use alloc::sync::Arc;
use los_utils::Mutex;

use crate::fs::vfs::superblock::Superblock;

// Submodules
mod dir_ops;
mod file_ops;
pub mod node;
mod superblock;
mod symlink_ops;

// Re-exports for public API
// pub use node::{TmpfsError, TmpfsNode, TmpfsNodeType};
pub use superblock::Tmpfs;

// Internal static ops for use in superblock.rs
use dir_ops::TmpfsDirOps;
use file_ops::TmpfsFileOps;
use symlink_ops::TmpfsSymlinkOps;

pub(self) static TMPFS_FILE_OPS: TmpfsFileOps = TmpfsFileOps;
pub(self) static TMPFS_DIR_OPS: TmpfsDirOps = TmpfsDirOps;
pub(self) static TMPFS_SYMLINK_OPS: TmpfsSymlinkOps = TmpfsSymlinkOps;

/// TEAM_194: Global tmpfs instance
pub static TMPFS: Mutex<Option<Arc<Tmpfs>>> = Mutex::new(None);

/// TEAM_194: Initialize the tmpfs
pub fn init() {
    let mut tmpfs_lock = TMPFS.lock();
    let tmpfs = Arc::new(Tmpfs::new());

    // Initialize VFS root
    let root_inode = tmpfs.make_inode(
        Arc::clone(&tmpfs.root),
        Arc::downgrade(&(Arc::clone(&tmpfs) as Arc<dyn Superblock>)),
    );
    *tmpfs.vfs_root.lock() = Some(root_inode);

    *tmpfs_lock = Some(tmpfs);
}

/// TEAM_194: Check if a path is under /tmp
pub fn is_tmpfs_path(path: &str) -> bool {
    let normalized = path.trim_start_matches('/');
    normalized.starts_with("tmp/") || normalized == "tmp"
}

/// TEAM_194: Strip /tmp prefix from path
pub fn strip_tmp_prefix(path: &str) -> &str {
    let normalized = path.trim_start_matches('/');
    if normalized == "tmp" {
        ""
    } else if let Some(rest) = normalized.strip_prefix("tmp/") {
        rest
    } else {
        normalized
    }
}
