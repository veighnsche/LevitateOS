//! TEAM_202: VFS Dispatch Layer
//!
//! High-level VFS operations that syscalls can call directly.
//! These handle path resolution, permission checks, and dispatch to
//! the appropriate filesystem.

extern crate alloc;

use alloc::sync::Arc;

use super::dentry::{Dentry, dcache};
use super::error::{VfsError, VfsResult};
use super::file::{File, FileRef, OpenFlags};
use super::ops::{DirEntry, SetAttr};
use crate::fs::mode;
use crate::syscall::Stat;

/// TEAM_202: Open a file by path
///
/// Resolves the path, checks permissions, and returns an open file handle.
pub fn vfs_open(path: &str, flags: OpenFlags, create_mode: u32) -> VfsResult<FileRef> {
    // Handle O_CREAT
    if flags.is_create() {
        return vfs_open_create(path, flags, create_mode);
    }

    // Look up the path
    let dentry = dcache().lookup(path)?;
    let inode = dentry.get_inode().ok_or(VfsError::NotFound)?;

    // Check O_DIRECTORY
    if flags.is_directory() && !inode.is_dir() {
        return Err(VfsError::NotADirectory);
    }

    // Handle O_TRUNC
    if flags.is_truncate() && flags.is_writable() && inode.is_file() {
        inode.truncate(0)?;
    }

    Ok(Arc::new(File::new(inode, flags)))
}

/// TEAM_202: Open with O_CREAT flag
fn vfs_open_create(path: &str, flags: OpenFlags, create_mode: u32) -> VfsResult<FileRef> {
    // Try to look up first
    match dcache().lookup(path) {
        Ok(dentry) => {
            // File exists
            if flags.is_exclusive() {
                return Err(VfsError::AlreadyExists);
            }

            let inode = dentry.get_inode().ok_or(VfsError::NotFound)?;

            if flags.is_directory() && !inode.is_dir() {
                return Err(VfsError::NotADirectory);
            }

            if flags.is_truncate() && flags.is_writable() && inode.is_file() {
                inode.truncate(0)?;
            }

            Ok(Arc::new(File::new(inode, flags)))
        }
        Err(VfsError::NotFound) => {
            // Need to create the file
            let (parent_dentry, name) = dcache().lookup_parent(path)?;
            let parent_inode = parent_dentry.get_inode().ok_or(VfsError::NotFound)?;

            if !parent_inode.is_dir() {
                return Err(VfsError::NotADirectory);
            }

            // Create the file
            let new_inode = parent_inode.create(name, mode::S_IFREG | (create_mode & 0o777))?;

            // Add to dentry cache
            let new_dentry = Arc::new(Dentry::new(
                alloc::string::String::from(name),
                Some(Arc::downgrade(&parent_dentry)),
                Some(new_inode.clone()),
            ));
            parent_dentry.add_child(new_dentry);

            Ok(Arc::new(File::new(new_inode, flags)))
        }
        Err(e) => Err(e),
    }
}

/// TEAM_202: Read from an open file
pub fn vfs_read(file: &File, buf: &mut [u8]) -> VfsResult<usize> {
    file.read(buf)
}

/// TEAM_202: Write to an open file
pub fn vfs_write(file: &File, buf: &[u8]) -> VfsResult<usize> {
    file.write(buf)
}

/// TEAM_202: Seek in an open file
pub fn vfs_seek(file: &File, offset: i64, whence: u32) -> VfsResult<u64> {
    let whence = super::ops::SeekWhence::from_u32(whence).ok_or(VfsError::InvalidArgument)?;
    file.seek(offset, whence)
}

/// TEAM_202: Get file status by path
pub fn vfs_stat(path: &str) -> VfsResult<Stat> {
    let dentry = dcache().lookup(path)?;
    let inode = dentry.get_inode().ok_or(VfsError::NotFound)?;
    Ok(inode.to_stat())
}

/// TEAM_202: Get file status by file descriptor
pub fn vfs_fstat(file: &File) -> VfsResult<Stat> {
    Ok(file.inode.to_stat())
}

/// TEAM_202: Create a directory
pub fn vfs_mkdir(path: &str, mode_bits: u32) -> VfsResult<()> {
    let (parent_dentry, name) = dcache().lookup_parent(path)?;
    let parent_inode = parent_dentry.get_inode().ok_or(VfsError::NotFound)?;

    if !parent_inode.is_dir() {
        return Err(VfsError::NotADirectory);
    }

    // Check if already exists
    if parent_dentry.lookup_child(name).is_some() {
        return Err(VfsError::AlreadyExists);
    }

    // Create the directory
    let new_inode = parent_inode.mkdir(name, mode::S_IFDIR | (mode_bits & 0o777))?;

    // Add to dentry cache
    let new_dentry = Arc::new(Dentry::new(
        alloc::string::String::from(name),
        Some(Arc::downgrade(&parent_dentry)),
        Some(new_inode),
    ));
    parent_dentry.add_child(new_dentry);

    Ok(())
}

/// TEAM_202: Remove a file
pub fn vfs_unlink(path: &str) -> VfsResult<()> {
    let (parent_dentry, name) = dcache().lookup_parent(path)?;
    let parent_inode = parent_dentry.get_inode().ok_or(VfsError::NotFound)?;

    if !parent_inode.is_dir() {
        return Err(VfsError::NotADirectory);
    }

    // Check that target exists and is not a directory
    let dentry = dcache().lookup(path)?;
    let inode = dentry.get_inode().ok_or(VfsError::NotFound)?;

    if inode.is_dir() {
        return Err(VfsError::IsADirectory);
    }

    // Remove from filesystem
    parent_inode.unlink(name)?;

    // Remove from dentry cache
    parent_dentry.remove_child(name);

    Ok(())
}

/// TEAM_202: Remove a directory
pub fn vfs_rmdir(path: &str) -> VfsResult<()> {
    let (parent_dentry, name) = dcache().lookup_parent(path)?;
    let parent_inode = parent_dentry.get_inode().ok_or(VfsError::NotFound)?;

    if !parent_inode.is_dir() {
        return Err(VfsError::NotADirectory);
    }

    // Check that target exists and is a directory
    let dentry = dcache().lookup(path)?;
    let inode = dentry.get_inode().ok_or(VfsError::NotFound)?;

    if !inode.is_dir() {
        return Err(VfsError::NotADirectory);
    }

    // Remove from filesystem
    parent_inode.rmdir(name)?;

    // Remove from dentry cache
    parent_dentry.remove_child(name);

    Ok(())
}

/// TEAM_202: Rename/move a file or directory
pub fn vfs_rename(old_path: &str, new_path: &str) -> VfsResult<()> {
    let (old_parent_dentry, old_name) = dcache().lookup_parent(old_path)?;
    let (new_parent_dentry, new_name) = dcache().lookup_parent(new_path)?;

    let old_parent_inode = old_parent_dentry.get_inode().ok_or(VfsError::NotFound)?;
    let new_parent_inode = new_parent_dentry.get_inode().ok_or(VfsError::NotFound)?;

    // Perform the rename in the filesystem
    old_parent_inode
        .ops
        .rename(&old_parent_inode, old_name, &new_parent_inode, new_name)?;

    // Update dentry cache
    if let Some(dentry) = old_parent_dentry.remove_child(old_name) {
        *dentry.name.write() = alloc::string::String::from(new_name);
        *dentry.parent.write() = Some(Arc::downgrade(&new_parent_dentry));
        new_parent_dentry.add_child(dentry);
    }

    Ok(())
}

/// TEAM_202: Create a symbolic link
pub fn vfs_symlink(target: &str, link_path: &str) -> VfsResult<()> {
    let (parent_dentry, name) = dcache().lookup_parent(link_path)?;
    let parent_inode = parent_dentry.get_inode().ok_or(VfsError::NotFound)?;

    if !parent_inode.is_dir() {
        return Err(VfsError::NotADirectory);
    }

    // Create the symlink
    let new_inode = parent_inode.ops.symlink(&parent_inode, name, target)?;

    // Add to dentry cache
    let new_dentry = Arc::new(Dentry::new(
        alloc::string::String::from(name),
        Some(Arc::downgrade(&parent_dentry)),
        Some(new_inode),
    ));
    parent_dentry.add_child(new_dentry);

    Ok(())
}

/// TEAM_202: Read directory entries
pub fn vfs_readdir(file: &File, offset: usize) -> VfsResult<Option<DirEntry>> {
    if !file.inode.is_dir() {
        return Err(VfsError::NotADirectory);
    }

    file.inode.ops.readdir(&file.inode, offset)
}

/// TEAM_202: Read symbolic link target
pub fn vfs_readlink(path: &str) -> VfsResult<alloc::string::String> {
    let dentry = dcache().lookup(path)?;
    let inode = dentry.get_inode().ok_or(VfsError::NotFound)?;

    if !inode.is_symlink() {
        return Err(VfsError::InvalidArgument);
    }

    inode.readlink()
}

/// TEAM_202: Truncate a file
pub fn vfs_truncate(path: &str, size: u64) -> VfsResult<()> {
    let dentry = dcache().lookup(path)?;
    let inode = dentry.get_inode().ok_or(VfsError::NotFound)?;

    if !inode.is_file() {
        return Err(VfsError::InvalidArgument);
    }

    inode.truncate(size)
}

/// TEAM_202: Check file access permissions
pub fn vfs_access(path: &str, _mode: u32) -> VfsResult<()> {
    let dentry = dcache().lookup(path)?;
    let _inode = dentry.get_inode().ok_or(VfsError::NotFound)?;

    // TODO: Implement proper permission checking based on uid/gid/mode
    // For now, just check existence
    Ok(())
}

/// TEAM_204: Set file attributes
pub fn vfs_setattr(path: &str, attr: &SetAttr) -> VfsResult<()> {
    let dentry = dcache().lookup(path)?;
    let inode = dentry.get_inode().ok_or(VfsError::NotFound)?;
    inode.ops.setattr(&inode, attr)
}

/// TEAM_204: Set file timestamps
pub fn vfs_utimes(path: &str, atime: Option<u64>, mtime: Option<u64>) -> VfsResult<()> {
    let attr = SetAttr {
        atime,
        mtime,
        ..Default::default()
    };
    vfs_setattr(path, &attr)
}

/// TEAM_209: Create a hard link
pub fn vfs_link(old_path: &str, new_path: &str) -> VfsResult<()> {
    // Look up the source
    let old_dentry = dcache().lookup(old_path)?;
    let old_inode = old_dentry.get_inode().ok_or(VfsError::NotFound)?;

    if old_inode.is_dir() {
        return Err(VfsError::IsADirectory);
    }

    // Resolve destination parent
    let (new_parent_dentry, new_name) = dcache().lookup_parent(new_path)?;
    let new_parent_inode = new_parent_dentry.get_inode().ok_or(VfsError::NotFound)?;

    if !new_parent_inode.is_dir() {
        return Err(VfsError::NotADirectory);
    }

    // Check if destination already exists
    if new_parent_dentry.lookup_child(new_name).is_some() {
        return Err(VfsError::AlreadyExists);
    }

    // Create the hard link in the filesystem
    new_parent_inode.ops.link(&new_parent_inode, new_name, &old_inode)?;

    // Add to dentry cache
    let new_dentry = Arc::new(Dentry::new(
        alloc::string::String::from(new_name),
        Some(Arc::downgrade(&new_parent_dentry)),
        Some(old_inode.clone()),
    ));
    new_parent_dentry.add_child(new_dentry);

    Ok(())
}
