//! TEAM_203: Tmpfs File Operations
//!
//! TEAM_208: Refactored from tmpfs.rs into separate module.

extern crate alloc;

use alloc::sync::Arc;
use core::sync::atomic::Ordering;
use los_utils::Mutex;

use crate::fs::vfs::error::{VfsError, VfsResult};
use crate::fs::vfs::inode::Inode;
use crate::fs::vfs::ops::InodeOps;

use super::TMPFS;
use super::node::{MAX_FILE_SIZE, MAX_TOTAL_SIZE, TmpfsNode};

/// TEAM_203: Tmpfs File Operations
pub(super) struct TmpfsFileOps;

impl InodeOps for TmpfsFileOps {
    fn read(&self, inode: &Inode, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let node = inode
            .private::<Arc<Mutex<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        let node_inner = node.lock();
        if !node_inner.is_file() {
            return Err(VfsError::IsADirectory);
        }

        if offset >= node_inner.data.len() as u64 {
            return Ok(0);
        }

        let available = node_inner.data.len() - offset as usize;
        let to_read = buf.len().min(available);
        buf[..to_read]
            .copy_from_slice(&node_inner.data[offset as usize..offset as usize + to_read]);

        Ok(to_read)
    }

    fn write(&self, inode: &Inode, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let node = inode
            .private::<Arc<Mutex<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        let mut node_inner = node.lock();
        if !node_inner.is_file() {
            return Err(VfsError::IsADirectory);
        }

        let offset = offset as usize;
        let new_size = offset.saturating_add(buf.len());

        // Check max file size
        if new_size > MAX_FILE_SIZE {
            return Err(VfsError::FileTooLarge);
        }

        // Tmpfs from Superblock
        let _sb_arc = inode.sb.upgrade().ok_or(VfsError::IoError)?;
        let tmpfs_lock = TMPFS.lock();
        let tmpfs = tmpfs_lock.as_ref().ok_or(VfsError::IoError)?;

        // Check total space
        let old_size = node_inner.data.len();
        let size_delta = if new_size > old_size {
            new_size - old_size
        } else {
            0
        };
        let current_used = tmpfs.bytes_used.load(Ordering::SeqCst);

        if current_used + size_delta > MAX_TOTAL_SIZE {
            return Err(VfsError::NoSpace);
        }

        // Extend file if needed
        if new_size > node_inner.data.len() {
            node_inner.data.resize(new_size, 0);
            tmpfs.bytes_used.fetch_add(size_delta, Ordering::SeqCst);
        }

        // Write data
        node_inner.data[offset..offset + buf.len()].copy_from_slice(buf);
        node_inner.mtime = crate::syscall::time::uptime_seconds();
        inode
            .size
            .store(node_inner.data.len() as u64, Ordering::Relaxed);
        inode.mtime.store(node_inner.mtime, Ordering::Relaxed);

        Ok(buf.len())
    }

    fn truncate(&self, inode: &Inode, size: u64) -> VfsResult<()> {
        let node = inode
            .private::<Arc<Mutex<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        let mut node_inner = node.lock();
        if !node_inner.is_file() {
            return Err(VfsError::IsADirectory);
        }

        let new_size = size as usize;

        // Check max file size
        if new_size > MAX_FILE_SIZE {
            return Err(VfsError::FileTooLarge);
        }

        let _sb_arc = inode.sb.upgrade().ok_or(VfsError::IoError)?;
        let tmpfs_lock = TMPFS.lock();
        let tmpfs = tmpfs_lock.as_ref().ok_or(VfsError::IoError)?;

        let old_size = node_inner.data.len();

        if new_size < old_size {
            // Shrink file
            let freed_bytes = old_size - new_size;
            node_inner.data.truncate(new_size);
            tmpfs.bytes_used.fetch_sub(freed_bytes, Ordering::SeqCst);
        } else if new_size > old_size {
            // Extend file
            let added_bytes = new_size - old_size;
            let current_used = tmpfs.bytes_used.load(Ordering::SeqCst);
            if current_used + added_bytes > MAX_TOTAL_SIZE {
                return Err(VfsError::NoSpace);
            }
            node_inner.data.resize(new_size, 0);
            tmpfs.bytes_used.fetch_add(added_bytes, Ordering::SeqCst);
        }

        node_inner.mtime = crate::syscall::time::uptime_seconds();
        inode
            .size
            .store(node_inner.data.len() as u64, Ordering::Relaxed);
        inode.mtime.store(node_inner.mtime, Ordering::Relaxed);

        Ok(())
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

        if let Some(size) = attr.size {
            drop(node_inner);
            self.truncate(inode, size)?;
        }

        Ok(())
    }
}
