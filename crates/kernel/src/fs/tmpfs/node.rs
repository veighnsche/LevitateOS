//! TEAM_194: Tmpfs Node Types
//!
//! TEAM_208: Refactored from tmpfs.rs into separate module.

extern crate alloc;

use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use los_utils::Mutex;

use crate::fs::vfs::error::{VfsError, VfsResult};

/// TEAM_194: Maximum file size (16MB)
pub const MAX_FILE_SIZE: usize = 16 * 1024 * 1024;

/// TEAM_194: Maximum total tmpfs size (64MB)
pub const MAX_TOTAL_SIZE: usize = 64 * 1024 * 1024;

/// TEAM_194: Tmpfs error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TmpfsError {
    /// File or directory not found
    NotFound,
    /// File or directory already exists
    AlreadyExists,
    /// Not a directory
    NotADirectory,
    /// Not a file
    NotAFile,
    /// Directory not empty
    NotEmpty,
    /// No space left
    NoSpace,
    /// File too large
    FileTooLarge,
    /// Invalid path
    InvalidPath,
    /// Operation not supported
    NotSupported,
    /// Permission denied (cross-filesystem)
    CrossDevice,
}

/// TEAM_194: Node type enumeration
/// TEAM_198: Added Symlink variant
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TmpfsNodeType {
    File,
    Directory,
    Symlink,
}

/// TEAM_257: Directory entry in a tmpfs directory
pub struct TmpfsDirEntry {
    pub name: String,
    pub node: Arc<Mutex<TmpfsNode>>,
}

/// TEAM_194: A node in the tmpfs tree (file or directory)
/// Note: No Debug derive to avoid Mutex<TmpfsNode> Debug requirement in FdType
pub struct TmpfsNode {
    /// Unique inode number
    pub ino: u64,
    /// Node type
    pub node_type: TmpfsNodeType,
    /// File content (for files only)
    pub data: Vec<u8>,
    /// Child nodes (for directories only)
    pub children: Vec<TmpfsDirEntry>,
    /// TEAM_198: Access time (seconds since boot)
    pub atime: u64,
    /// TEAM_198: Modification time (seconds since boot)
    pub mtime: u64,
    /// TEAM_198: Creation time (seconds since boot)
    pub ctime: u64,
    /// Parent node (Weak reference to avoid cycles)
    pub parent: Weak<Mutex<TmpfsNode>>,
    /// TEAM_209: Number of hard links
    pub nlink: u32,
}

impl TmpfsNode {
    /// TEAM_194: Create a new file node
    /// TEAM_198: Added timestamp fields
    /// TEAM_209: Added nlink field
    pub fn new_file(ino: u64) -> Self {
        let now = crate::syscall::time::uptime_seconds();
        Self {
            ino,
            node_type: TmpfsNodeType::File,
            data: Vec::new(),
            children: Vec::new(),
            atime: now,
            mtime: now,
            ctime: now,
            parent: Weak::new(),
            nlink: 1,
        }
    }

    /// TEAM_194: Create a new directory node
    /// TEAM_198: Added timestamp fields
    /// TEAM_209: Added nlink field
    pub fn new_dir(ino: u64) -> Self {
        let now = crate::syscall::time::uptime_seconds();
        Self {
            ino,
            node_type: TmpfsNodeType::Directory,
            data: Vec::new(),
            children: Vec::new(),
            atime: now,
            mtime: now,
            ctime: now,
            parent: Weak::new(),
            nlink: 2, // self + parent
        }
    }

    /// TEAM_198: Create a new symlink node
    /// TEAM_209: Added nlink field
    pub fn new_symlink(ino: u64, target: &str) -> Self {
        let now = crate::syscall::time::uptime_seconds();
        Self {
            ino,
            node_type: TmpfsNodeType::Symlink,
            data: target.as_bytes().to_vec(), // Store target path in data
            children: Vec::new(),
            atime: now,
            mtime: now,
            ctime: now,
            parent: Weak::new(),
            nlink: 1,
        }
    }

    /// TEAM_194: Check if this is a file
    pub fn is_file(&self) -> bool {
        self.node_type == TmpfsNodeType::File
    }

    /// TEAM_194: Check if this is a directory
    pub fn is_dir(&self) -> bool {
        self.node_type == TmpfsNodeType::Directory
    }

    /// TEAM_198: Check if this is a symlink
    pub fn is_symlink(&self) -> bool {
        self.node_type == TmpfsNodeType::Symlink
    }

    /// TEAM_198: Get symlink target (returns None if not a symlink)
    pub fn symlink_target(&self) -> Option<&[u8]> {
        if self.is_symlink() {
            Some(&self.data)
        } else {
            None
        }
    }

    /// TEAM_194: Check if this is a symlink (alias)
    pub fn is_lnk(&self) -> bool {
        self.node_type == TmpfsNodeType::Symlink
    }

    /// TEAM_204: Check if this node is a descendant of the given node
    pub fn is_descendant_of(&self, other_ino: u64) -> bool {
        if self.ino == other_ino {
            return true;
        }

        let mut curr_node = self.parent.upgrade();
        while let Some(node) = curr_node {
            let locked = node.lock();
            if locked.ino == other_ino {
                return true;
            }
            curr_node = locked.parent.upgrade();
        }

        false
    }
}

/// TEAM_203: Shared logic for creating nodes
pub(super) fn add_child(
    parent: &Arc<Mutex<TmpfsNode>>,
    name: &str,
    child: Arc<Mutex<TmpfsNode>>,
) -> VfsResult<()> {
    let mut parent_node = parent.lock();
    if !parent_node.is_dir() {
        return Err(VfsError::NotADirectory);
    }
    for entry in &parent_node.children {
        if entry.name == name {
            return Err(VfsError::AlreadyExists);
        }
    }
    child.lock().parent = Arc::downgrade(parent);
    parent_node.children.push(TmpfsDirEntry {
        name: String::from(name),
        node: child,
    });
    Ok(())
}
