//! TEAM_203: Tmpfs Symlink Operations
//!
//! TEAM_208: Refactored from tmpfs.rs into separate module.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::sync::atomic::Ordering;
use los_utils::Mutex;

use crate::fs::vfs::error::{VfsError, VfsResult};
use crate::fs::vfs::inode::Inode;
use crate::fs::vfs::ops::InodeOps;

use super::node::TmpfsNode;

/// TEAM_203: Tmpfs Symlink Operations
pub(super) struct TmpfsSymlinkOps;

impl InodeOps for TmpfsSymlinkOps {
    fn readlink(&self, inode: &Inode) -> VfsResult<String> {
        let node = inode
            .private::<Arc<Mutex<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;
        let node_inner = node.lock();
        if !node_inner.is_symlink() {
            return Err(VfsError::InvalidArgument);
        }
        Ok(String::from_utf8_lossy(&node_inner.data).to_string())
    }

    fn setattr(&self, inode: &Inode, attr: &crate::fs::vfs::ops::SetAttr) -> VfsResult<()> {
        let node = inode
            .private::<Arc<Mutex<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;
        let mut node_inner = node.lock();

        if let Some(mode) = attr.mode {
            inode.mode.store(mode, Ordering::Relaxed);
            node_inner.mtime = crate::syscall::time::uptime_seconds();
            node_inner.ctime = node_inner.mtime;
        }

        if let Some(atime) = attr.atime {
            node_inner.atime = atime;
            inode.atime.store(atime, Ordering::Relaxed);
        }

        if let Some(mtime) = attr.mtime {
            node_inner.mtime = mtime;
            node_inner.ctime = mtime;
            inode.mtime.store(mtime, Ordering::Relaxed);
            inode.ctime.store(mtime, Ordering::Relaxed);
        }

        Ok(())
    }
}
