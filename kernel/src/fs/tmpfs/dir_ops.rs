//! TEAM_203: Tmpfs Directory Operations
//!
//! TEAM_208: Refactored from tmpfs.rs into separate module.

extern crate alloc;

use alloc::string::ToString;
use alloc::sync::Arc;
use core::sync::atomic::Ordering;
use los_utils::Spinlock;

use crate::fs::mode;
use crate::fs::vfs::error::{VfsError, VfsResult};
use crate::fs::vfs::inode::Inode;
use crate::fs::vfs::ops::{DirEntry, InodeOps};

use super::TMPFS;
use super::node::{TmpfsNode, TmpfsNodeType, add_child};

/// TEAM_203: Tmpfs Directory Operations
pub(super) struct TmpfsDirOps;

impl InodeOps for TmpfsDirOps {
    fn lookup(&self, inode: &Inode, name: &str) -> VfsResult<Arc<Inode>> {
        let node = inode
            .private::<Arc<Spinlock<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        let node_inner = node.lock();
        if !node_inner.is_dir() {
            return Err(VfsError::NotADirectory);
        }

        for child in &node_inner.children {
            let child_node = child.lock();
            if child_node.name == name {
                let sb = inode.sb.upgrade().ok_or(VfsError::IoError)?;
                let tmpfs_lock = TMPFS.lock();
                let tmpfs = tmpfs_lock.as_ref().ok_or(VfsError::IoError)?;
                return Ok(tmpfs.make_inode(Arc::clone(child), Arc::downgrade(&sb)));
            }
        }

        Err(VfsError::NotFound)
    }

    fn readdir(&self, inode: &Inode, offset: usize) -> VfsResult<Option<DirEntry>> {
        let node = inode
            .private::<Arc<Spinlock<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        let node_inner = node.lock();
        if !node_inner.is_dir() {
            return Err(VfsError::NotADirectory);
        }

        // offsets 0 and 1 are . and ..
        if offset == 0 {
            return Ok(Some(DirEntry {
                ino: node_inner.ino,
                name: ".".to_string(),
                file_type: mode::S_IFDIR,
            }));
        }
        if offset == 1 {
            let parent_ino = if let Some(p) = node_inner.parent.upgrade() {
                p.lock().ino
            } else {
                node_inner.ino // root's parent is itself
            };
            return Ok(Some(DirEntry {
                ino: parent_ino,
                name: "..".to_string(),
                file_type: mode::S_IFDIR,
            }));
        }

        let child_idx = offset - 2;
        if child_idx < node_inner.children.len() {
            let child = &node_inner.children[child_idx];
            let child_node = child.lock();
            let de = DirEntry {
                ino: child_node.ino,
                name: child_node.name.clone(),
                file_type: mode::file_type(match child_node.node_type {
                    TmpfsNodeType::File => mode::S_IFREG,
                    TmpfsNodeType::Directory => mode::S_IFDIR,
                    TmpfsNodeType::Symlink => mode::S_IFLNK,
                }),
            };
            Ok(Some(de))
        } else {
            Ok(None)
        }
    }

    fn create(&self, inode: &Inode, name: &str, _mode: u32) -> VfsResult<Arc<Inode>> {
        let node = inode
            .private::<Arc<Spinlock<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        let sb = inode.sb.upgrade().ok_or(VfsError::IoError)?;
        let tmpfs_lock = TMPFS.lock();
        let tmpfs = tmpfs_lock.as_ref().ok_or(VfsError::IoError)?;

        let ino = tmpfs.alloc_ino();
        let new_node = Arc::new(Spinlock::new(TmpfsNode::new_file(ino, name)));
        add_child(node, Arc::clone(&new_node))?;

        Ok(tmpfs.make_inode(new_node, Arc::downgrade(&sb)))
    }

    fn mkdir(&self, inode: &Inode, name: &str, _mode: u32) -> VfsResult<Arc<Inode>> {
        let node = inode
            .private::<Arc<Spinlock<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        let sb = inode.sb.upgrade().ok_or(VfsError::IoError)?;
        let tmpfs_lock = TMPFS.lock();
        let tmpfs = tmpfs_lock.as_ref().ok_or(VfsError::IoError)?;

        let ino = tmpfs.alloc_ino();
        let new_node = Arc::new(Spinlock::new(TmpfsNode::new_dir(ino, name)));
        add_child(node, Arc::clone(&new_node))?;

        Ok(tmpfs.make_inode(new_node, Arc::downgrade(&sb)))
    }

    fn symlink(&self, inode: &Inode, name: &str, target: &str) -> VfsResult<Arc<Inode>> {
        let node = inode
            .private::<Arc<Spinlock<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        let sb = inode.sb.upgrade().ok_or(VfsError::IoError)?;
        let tmpfs_lock = TMPFS.lock();
        let tmpfs = tmpfs_lock.as_ref().ok_or(VfsError::IoError)?;

        let ino = tmpfs.alloc_ino();
        let new_node = Arc::new(Spinlock::new(TmpfsNode::new_symlink(ino, name, target)));
        add_child(node, Arc::clone(&new_node))?;

        Ok(tmpfs.make_inode(new_node, Arc::downgrade(&sb)))
    }

    fn rename(
        &self,
        old_dir: &Inode,
        old_name: &str,
        new_dir: &Inode,
        new_name: &str,
    ) -> VfsResult<()> {
        let old_node = old_dir
            .private::<Arc<Spinlock<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;
        let new_node = new_dir
            .private::<Arc<Spinlock<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        // TEAM_204: Rename cycle check
        {
            let old_node_locked = old_dir
                .private::<Arc<Spinlock<TmpfsNode>>>()
                .ok_or(VfsError::IoError)?
                .lock();
            let mut to_move = None;
            for child in &old_node_locked.children {
                if child.lock().name == old_name {
                    to_move = Some(child.clone());
                    break;
                }
            }
            if let Some(child) = to_move {
                let child_locked = child.lock();
                if child_locked.is_dir() {
                    let new_dir_node = new_node.lock();
                    if new_dir_node.is_descendant_of(child_locked.ino) {
                        return Err(VfsError::InvalidArgument); // Moving dir into its own subdir
                    }
                }
            } else {
                return Err(VfsError::NotFound);
            }
        }

        if Arc::ptr_eq(&old_node, &new_node) {
            let mut locked = old_node.lock();
            if !locked.is_dir() {
                return Err(VfsError::NotADirectory);
            }

            let mut found_idx = None;
            for (idx, child) in locked.children.iter().enumerate() {
                if child.lock().name == old_name {
                    found_idx = Some(idx);
                    break;
                }
            }
            let idx = found_idx.ok_or(VfsError::NotFound)?;

            // Check if target exists
            let mut target_idx = None;
            for (t_idx, child) in locked.children.iter().enumerate() {
                if child.lock().name == new_name {
                    target_idx = Some(t_idx);
                    break;
                }
            }

            if let Some(t_idx) = target_idx {
                if t_idx == idx {
                    // Renaming to same name, nothing to do
                    return Ok(());
                }
                let existing = locked.children.remove(t_idx);
                if existing.lock().is_dir() && !existing.lock().children.is_empty() {
                    locked.children.insert(t_idx, existing);
                    return Err(VfsError::DirectoryNotEmpty);
                }
                // Update bytes_used if it was a file/symlink
                let tmpfs_lock = TMPFS.lock();
                let tmpfs = tmpfs_lock.as_ref().ok_or(VfsError::IoError)?;
                if !existing.lock().is_dir() {
                    tmpfs
                        .bytes_used
                        .fetch_sub(existing.lock().data.len(), Ordering::SeqCst);
                }

                // Adjust index if needed since we removed an element
                let final_idx = if t_idx < idx { idx - 1 } else { idx };
                let to_move = locked.children.remove(final_idx);
                to_move.lock().name = new_name.to_string();
                locked.children.insert(t_idx, to_move); // Insert at the target's old position
            } else {
                let to_move = locked.children.remove(idx);
                to_move.lock().name = new_name.to_string();
                locked.children.push(to_move);
            }
        } else {
            let mut old_locked = old_node.lock();
            let mut new_locked = new_node.lock();

            if !old_locked.is_dir() || !new_locked.is_dir() {
                return Err(VfsError::NotADirectory);
            }

            let mut found_idx = None;
            for (idx, child) in old_locked.children.iter().enumerate() {
                if child.lock().name == old_name {
                    found_idx = Some(idx);
                    break;
                }
            }
            let to_move_arc = old_locked
                .children
                .remove(found_idx.ok_or(VfsError::NotFound)?);

            // Check if target exists and remove it
            let mut target_idx = None;
            for (idx, child) in new_locked.children.iter().enumerate() {
                if child.lock().name == new_name {
                    target_idx = Some(idx);
                    break;
                }
            }
            if let Some(idx) = target_idx {
                let existing_child = new_locked.children.remove(idx);
                // If it's a directory, it must be empty
                if existing_child.lock().is_dir() && !existing_child.lock().children.is_empty() {
                    // Put it back and return error
                    new_locked.children.insert(idx, existing_child);
                    old_locked.children.insert(found_idx.unwrap(), to_move_arc); // Put back original
                    return Err(VfsError::DirectoryNotEmpty);
                }
                // If it's a file/symlink, or an empty directory, it's replaced.
                // Update bytes_used if it was a file/symlink
                let tmpfs_lock = TMPFS.lock();
                let tmpfs = tmpfs_lock.as_ref().ok_or(VfsError::IoError)?;
                if !existing_child.lock().is_dir() {
                    tmpfs
                        .bytes_used
                        .fetch_sub(existing_child.lock().data.len(), Ordering::SeqCst);
                }
            }

            to_move_arc.lock().name = new_name.to_string();
            new_locked.children.push(to_move_arc);
        }

        Ok(())
    }

    fn unlink(&self, inode: &Inode, name: &str) -> VfsResult<()> {
        let node = inode
            .private::<Arc<Spinlock<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        let mut parent_node = node.lock();
        let mut found_idx = None;
        for (idx, child) in parent_node.children.iter().enumerate() {
            let child_node = (**child).lock();
            if child_node.name == name {
                if child_node.is_dir() {
                    return Err(VfsError::IsADirectory);
                }
                found_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = found_idx {
            let child = parent_node.children.remove(idx);
            
            // Decrement nlink in TmpfsNode
            let mut child_locked = child.lock();
            child_locked.nlink -= 1;
            
            // If it was the last link, decrement global bytes_used
            if child_locked.nlink == 0 {
                let tmpfs_lock = TMPFS.lock();
                let tmpfs = tmpfs_lock.as_ref().ok_or(VfsError::IoError)?;
                tmpfs
                    .bytes_used
                    .fetch_sub(child_locked.data.len(), Ordering::SeqCst);
            }
            Ok(())
        } else {
            Err(VfsError::NotFound)
        }
    }

    fn rmdir(&self, inode: &Inode, name: &str) -> VfsResult<()> {
        let node = inode
            .private::<Arc<Spinlock<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        let mut parent_node = node.lock();
        let mut found_idx = None;
        for (idx, child) in parent_node.children.iter().enumerate() {
            let child_node = (**child).lock();
            if child_node.name == name {
                if !child_node.is_dir() {
                    return Err(VfsError::NotADirectory);
                }
                if !child_node.children.is_empty() {
                    return Err(VfsError::DirectoryNotEmpty);
                }
                found_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = found_idx {
            let child = parent_node.children.remove(idx);
            child.lock().nlink -= 1; // self reference
            parent_node.nlink -= 1; // child's .. reference
            Ok(())
        } else {
            Err(VfsError::NotFound)
        }
    }

    fn link(&self, inode: &Inode, _name: &str, target: &Inode) -> VfsResult<()> {
        let parent_node_arc = inode
            .private::<Arc<Spinlock<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;
        
        let target_node_arc = target
            .private::<Arc<Spinlock<TmpfsNode>>>()
            .ok_or(VfsError::IoError)?;

        // Standard Unix restriction: no hard links to directories
        if target.is_dir() {
            return Err(VfsError::IsADirectory);
        }

        // Add child (handles name collision and setting parent weak ref)
        add_child(parent_node_arc, Arc::clone(target_node_arc))?;
        
        // Increment link count
        target_node_arc.lock().nlink += 1;
        target.inc_nlink();

        Ok(())
    }

    fn setattr(&self, inode: &Inode, attr: &crate::fs::vfs::ops::SetAttr) -> VfsResult<()> {
        let node = inode
            .private::<Arc<Spinlock<TmpfsNode>>>()
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
