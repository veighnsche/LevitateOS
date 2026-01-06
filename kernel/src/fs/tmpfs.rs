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

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use los_utils::Spinlock;

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

/// TEAM_194: A node in the tmpfs tree (file or directory)
/// Note: No Debug derive to avoid Spinlock<TmpfsNode> Debug requirement in FdType
pub struct TmpfsNode {
    /// Unique inode number
    pub ino: u64,
    /// Node name (not full path)
    pub name: String,
    /// Node type
    pub node_type: TmpfsNodeType,
    /// File content (for files only)
    pub data: Vec<u8>,
    /// Child nodes (for directories only)
    pub children: Vec<Arc<Spinlock<TmpfsNode>>>,
    /// TEAM_198: Access time (seconds since boot)
    pub atime: u64,
    /// TEAM_198: Modification time (seconds since boot)
    pub mtime: u64,
    /// TEAM_198: Creation time (seconds since boot)
    pub ctime: u64,
}

impl TmpfsNode {
    /// TEAM_194: Create a new file node
    /// TEAM_198: Added timestamp fields
    pub fn new_file(ino: u64, name: &str) -> Self {
        let now = crate::syscall::time::uptime_seconds();
        Self {
            ino,
            name: String::from(name),
            node_type: TmpfsNodeType::File,
            data: Vec::new(),
            children: Vec::new(),
            atime: now,
            mtime: now,
            ctime: now,
        }
    }

    /// TEAM_194: Create a new directory node
    /// TEAM_198: Added timestamp fields
    pub fn new_dir(ino: u64, name: &str) -> Self {
        let now = crate::syscall::time::uptime_seconds();
        Self {
            ino,
            name: String::from(name),
            node_type: TmpfsNodeType::Directory,
            data: Vec::new(),
            children: Vec::new(),
            atime: now,
            mtime: now,
            ctime: now,
        }
    }

    /// TEAM_198: Create a new symlink node
    pub fn new_symlink(ino: u64, name: &str, target: &str) -> Self {
        let now = crate::syscall::time::uptime_seconds();
        Self {
            ino,
            name: String::from(name),
            node_type: TmpfsNodeType::Symlink,
            data: target.as_bytes().to_vec(), // Store target path in data
            children: Vec::new(),
            atime: now,
            mtime: now,
            ctime: now,
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

    /// TEAM_194: Get file size
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// TEAM_194: The tmpfs filesystem state
pub struct Tmpfs {
    /// Root directory node
    root: Arc<Spinlock<TmpfsNode>>,
    /// Next inode number
    next_ino: AtomicU64,
    /// Total bytes used
    bytes_used: AtomicUsize,
}

impl Tmpfs {
    /// TEAM_194: Create a new tmpfs instance
    pub fn new() -> Self {
        Self {
            root: Arc::new(Spinlock::new(TmpfsNode::new_dir(1, ""))),
            next_ino: AtomicU64::new(2),
            bytes_used: AtomicUsize::new(0),
        }
    }

    /// TEAM_194: Allocate a new inode number
    fn alloc_ino(&self) -> u64 {
        self.next_ino.fetch_add(1, Ordering::SeqCst)
    }

    /// TEAM_194: Get total bytes used
    pub fn bytes_used(&self) -> usize {
        self.bytes_used.load(Ordering::SeqCst)
    }

    /// TEAM_194: Parse path into components (skip empty parts)
    fn parse_path(path: &str) -> Vec<&str> {
        path.split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .collect()
    }

    /// TEAM_194: Lookup a node by path, returning the node and its parent
    pub fn lookup(&self, path: &str) -> Option<Arc<Spinlock<TmpfsNode>>> {
        let components = Self::parse_path(path);
        
        if components.is_empty() {
            return Some(Arc::clone(&self.root));
        }

        let mut current = Arc::clone(&self.root);

        for component in components {
            let node = current.lock();
            if !node.is_dir() {
                return None;
            }

            let mut found = None;
            for child in &node.children {
                let child_node = child.lock();
                if child_node.name == component {
                    found = Some(Arc::clone(child));
                    break;
                }
            }
            drop(node);

            match found {
                Some(child) => current = child,
                None => return None,
            }
        }

        Some(current)
    }

    /// TEAM_194: Lookup parent directory and return (parent, basename)
    fn lookup_parent(&self, path: &str) -> Option<(Arc<Spinlock<TmpfsNode>>, String)> {
        let components = Self::parse_path(path);
        
        if components.is_empty() {
            return None; // Can't get parent of root
        }

        let basename = components.last()?.to_string();
        
        if components.len() == 1 {
            return Some((Arc::clone(&self.root), basename));
        }

        let parent_path: String = components[..components.len() - 1].join("/");
        let parent = self.lookup(&parent_path)?;
        
        Some((parent, basename))
    }

    /// TEAM_194: Create a file at the given path
    pub fn create_file(&self, path: &str) -> Result<Arc<Spinlock<TmpfsNode>>, TmpfsError> {
        let (parent, name) = self.lookup_parent(path).ok_or(TmpfsError::InvalidPath)?;
        
        let mut parent_node = parent.lock();
        if !parent_node.is_dir() {
            return Err(TmpfsError::NotADirectory);
        }

        // Check if already exists
        for child in &parent_node.children {
            let child_node = child.lock();
            if child_node.name == name {
                if child_node.is_file() {
                    // File exists, return it (for O_TRUNC, caller will truncate)
                    return Ok(Arc::clone(child));
                } else {
                    return Err(TmpfsError::AlreadyExists);
                }
            }
        }

        // Create new file
        let ino = self.alloc_ino();
        let new_file = Arc::new(Spinlock::new(TmpfsNode::new_file(ino, &name)));
        parent_node.children.push(Arc::clone(&new_file));

        Ok(new_file)
    }

    /// TEAM_194: Create a directory at the given path
    pub fn create_dir(&self, path: &str) -> Result<Arc<Spinlock<TmpfsNode>>, TmpfsError> {
        let (parent, name) = self.lookup_parent(path).ok_or(TmpfsError::InvalidPath)?;
        
        let mut parent_node = parent.lock();
        if !parent_node.is_dir() {
            return Err(TmpfsError::NotADirectory);
        }

        // Check if already exists
        for child in &parent_node.children {
            let child_node = child.lock();
            if child_node.name == name {
                return Err(TmpfsError::AlreadyExists);
            }
        }

        // Create new directory
        let ino = self.alloc_ino();
        let new_dir = Arc::new(Spinlock::new(TmpfsNode::new_dir(ino, &name)));
        parent_node.children.push(Arc::clone(&new_dir));

        Ok(new_dir)
    }

    /// TEAM_198: Create a symbolic link at the given path
    pub fn create_symlink(&self, path: &str, target: &str) -> Result<Arc<Spinlock<TmpfsNode>>, TmpfsError> {
        let (parent, name) = self.lookup_parent(path).ok_or(TmpfsError::InvalidPath)?;
        
        let mut parent_node = parent.lock();
        if !parent_node.is_dir() {
            return Err(TmpfsError::NotADirectory);
        }

        // Check if already exists
        for child in &parent_node.children {
            let child_node = child.lock();
            if child_node.name == name {
                return Err(TmpfsError::AlreadyExists);
            }
        }

        // Create new symlink
        let ino = self.alloc_ino();
        let new_symlink = Arc::new(Spinlock::new(TmpfsNode::new_symlink(ino, &name, target)));
        parent_node.children.push(Arc::clone(&new_symlink));

        Ok(new_symlink)
    }

    /// TEAM_194: Remove a file or empty directory
    pub fn remove(&self, path: &str, remove_dir: bool) -> Result<(), TmpfsError> {
        let (parent, name) = self.lookup_parent(path).ok_or(TmpfsError::InvalidPath)?;
        
        let mut parent_node = parent.lock();
        if !parent_node.is_dir() {
            return Err(TmpfsError::NotADirectory);
        }

        let mut found_idx = None;
        for (idx, child) in parent_node.children.iter().enumerate() {
            let child_node = child.lock();
            if child_node.name == name {
                if remove_dir {
                    // Removing directory
                    if !child_node.is_dir() {
                        return Err(TmpfsError::NotADirectory);
                    }
                    if !child_node.children.is_empty() {
                        return Err(TmpfsError::NotEmpty);
                    }
                } else {
                    // Removing file
                    if child_node.is_dir() {
                        return Err(TmpfsError::NotAFile);
                    }
                    // Update bytes used
                    self.bytes_used.fetch_sub(child_node.data.len(), Ordering::SeqCst);
                }
                found_idx = Some(idx);
                break;
            }
        }

        match found_idx {
            Some(idx) => {
                parent_node.children.remove(idx);
                Ok(())
            }
            None => Err(TmpfsError::NotFound),
        }
    }

    /// TEAM_194: Rename/move a node within tmpfs
    pub fn rename(&self, old_path: &str, new_path: &str) -> Result<(), TmpfsError> {
        // Find old node's parent and name
        let (old_parent, old_name) = self.lookup_parent(old_path).ok_or(TmpfsError::InvalidPath)?;
        let (new_parent, new_name) = self.lookup_parent(new_path).ok_or(TmpfsError::InvalidPath)?;

        // Find and remove from old parent
        let mut old_parent_node = old_parent.lock();
        let mut found_idx = None;
        let mut node_to_move = None;

        for (idx, child) in old_parent_node.children.iter().enumerate() {
            let child_node = child.lock();
            if child_node.name == old_name {
                found_idx = Some(idx);
                node_to_move = Some(Arc::clone(child));
                break;
            }
        }

        let idx = found_idx.ok_or(TmpfsError::NotFound)?;
        let node = node_to_move.ok_or(TmpfsError::NotFound)?;
        
        old_parent_node.children.remove(idx);
        drop(old_parent_node);

        // Update name
        {
            let mut node_inner = node.lock();
            node_inner.name = new_name;
        }

        // Add to new parent
        let mut new_parent_node = new_parent.lock();
        if !new_parent_node.is_dir() {
            return Err(TmpfsError::NotADirectory);
        }

        // Remove existing target if it exists
        let mut existing_idx = None;
        for (idx, child) in new_parent_node.children.iter().enumerate() {
            let child_node = child.lock();
            if child_node.name == node.lock().name {
                existing_idx = Some(idx);
                break;
            }
        }
        if let Some(idx) = existing_idx {
            new_parent_node.children.remove(idx);
        }

        new_parent_node.children.push(node);
        Ok(())
    }

    /// TEAM_194: Write data to a file
    pub fn write_file(
        &self,
        node: &Arc<Spinlock<TmpfsNode>>,
        offset: usize,
        data: &[u8],
    ) -> Result<usize, TmpfsError> {
        let mut node_inner = node.lock();
        
        if !node_inner.is_file() {
            return Err(TmpfsError::NotAFile);
        }

        let new_size = offset.saturating_add(data.len());
        
        // Check max file size
        if new_size > MAX_FILE_SIZE {
            return Err(TmpfsError::FileTooLarge);
        }

        // Check total space
        let old_size = node_inner.data.len();
        let size_delta = if new_size > old_size { new_size - old_size } else { 0 };
        let current_used = self.bytes_used.load(Ordering::SeqCst);
        
        if current_used + size_delta > MAX_TOTAL_SIZE {
            return Err(TmpfsError::NoSpace);
        }

        // Extend file if needed
        if new_size > node_inner.data.len() {
            node_inner.data.resize(new_size, 0);
            self.bytes_used.fetch_add(size_delta, Ordering::SeqCst);
        }

        // Write data
        node_inner.data[offset..offset + data.len()].copy_from_slice(data);

        // TEAM_198: Update mtime on write
        node_inner.mtime = crate::syscall::time::uptime_seconds();

        Ok(data.len())
    }

    /// TEAM_198: Update timestamps on a node
    pub fn update_timestamps(
        &self,
        node: &Arc<Spinlock<TmpfsNode>>,
        atime: Option<u64>,
        mtime: Option<u64>,
    ) -> Result<(), TmpfsError> {
        let mut node_inner = node.lock();
        if let Some(a) = atime {
            node_inner.atime = a;
        }
        if let Some(m) = mtime {
            node_inner.mtime = m;
        }
        Ok(())
    }

    /// TEAM_194: Read data from a file
    pub fn read_file(
        &self,
        node: &Arc<Spinlock<TmpfsNode>>,
        offset: usize,
        len: usize,
    ) -> Result<Vec<u8>, TmpfsError> {
        let node_inner = node.lock();
        
        if !node_inner.is_file() {
            return Err(TmpfsError::NotAFile);
        }

        if offset >= node_inner.data.len() {
            return Ok(Vec::new()); // EOF
        }

        let available = node_inner.data.len() - offset;
        let to_read = len.min(available);

        Ok(node_inner.data[offset..offset + to_read].to_vec())
    }

    /// TEAM_194: Truncate a file to zero length
    pub fn truncate(&self, node: &Arc<Spinlock<TmpfsNode>>) -> Result<(), TmpfsError> {
        let mut node_inner = node.lock();
        
        if !node_inner.is_file() {
            return Err(TmpfsError::NotAFile);
        }

        let old_size = node_inner.data.len();
        node_inner.data.clear();
        self.bytes_used.fetch_sub(old_size, Ordering::SeqCst);

        Ok(())
    }
}

impl Default for Tmpfs {
    fn default() -> Self {
        Self::new()
    }
}

/// TEAM_194: Global tmpfs instance
pub static TMPFS: Spinlock<Option<Tmpfs>> = Spinlock::new(None);

/// TEAM_194: Initialize the tmpfs
pub fn init() {
    let mut tmpfs = TMPFS.lock();
    *tmpfs = Some(Tmpfs::new());
    crate::verbose!("tmpfs initialized at /tmp");
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
