//! TEAM_168: File Descriptor Table for LevitateOS.
//!
//! Per-process file descriptor management for syscalls.
//! Supports stdin/stdout/stderr (fd 0/1/2), initramfs files, and tmpfs files.

use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::fs::tmpfs::TmpfsNode;
use los_hal::IrqSafeLock;
use los_utils::Spinlock;

/// TEAM_168: Maximum number of open file descriptors per process.
pub const MAX_FDS: usize = 64;

/// TEAM_168: Type of file descriptor entry.
/// TEAM_194: Removed Copy derive to support Arc<> for tmpfs nodes.
/// TEAM_195: Removed Debug derive since Spinlock<TmpfsNode> doesn't implement Debug.
#[derive(Clone)]
pub enum FdType {
    /// Standard input (keyboard)
    Stdin,
    /// Standard output (console)
    Stdout,
    /// Standard error (console)
    Stderr,
    /// File from initramfs (read-only)
    InitramfsFile {
        /// Index into initramfs file table
        file_index: usize,
        /// Current read position
        offset: usize,
    },
    /// TEAM_176: Directory from initramfs for getdents
    InitramfsDir {
        /// Directory path (stored as index into a path for simplicity)
        /// 0 = root directory
        dir_index: usize,
        /// Current entry offset for iteration
        offset: usize,
    },
    /// TEAM_194: File from tmpfs (read-write)
    TmpfsFile {
        /// Reference to the tmpfs node
        node: Arc<Spinlock<TmpfsNode>>,
        /// Current read/write position
        offset: usize,
    },
    /// TEAM_194: Directory from tmpfs for getdents
    TmpfsDir {
        /// Reference to the tmpfs directory node
        node: Arc<Spinlock<TmpfsNode>>,
        /// Current entry offset for iteration
        offset: usize,
    },
}

/// TEAM_168: A single file descriptor entry.
/// TEAM_194: Removed Copy derive since FdType no longer implements Copy.
/// TEAM_195: Removed Debug derive since FdType no longer implements Debug.
#[derive(Clone)]
pub struct FdEntry {
    /// Type and state of this fd
    pub fd_type: FdType,
    /// Reference count (for future dup() support)
    #[allow(dead_code)]
    pub refcount: usize,
}

impl FdEntry {
    /// TEAM_168: Create a new fd entry.
    pub fn new(fd_type: FdType) -> Self {
        Self {
            fd_type,
            refcount: 1,
        }
    }
}

/// TEAM_168: Per-process file descriptor table.
/// TEAM_195: Removed Debug derive since FdEntry no longer implements Debug.
pub struct FdTable {
    /// Sparse array of file descriptors (None = unused slot)
    entries: Vec<Option<FdEntry>>,
}

impl FdTable {
    /// TEAM_168: Create a new fd table with stdin/stdout/stderr pre-populated.
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(MAX_FDS);
        
        // Pre-populate fd 0 (stdin), 1 (stdout), 2 (stderr)
        entries.push(Some(FdEntry::new(FdType::Stdin)));  // fd 0
        entries.push(Some(FdEntry::new(FdType::Stdout))); // fd 1
        entries.push(Some(FdEntry::new(FdType::Stderr))); // fd 2
        
        Self { entries }
    }

    /// TEAM_168: Allocate a new file descriptor (lowest available per Q2 decision).
    ///
    /// Returns the fd number on success, or None if table is full.
    pub fn alloc(&mut self, fd_type: FdType) -> Option<usize> {
        // Q2 decision: Always use lowest available (POSIX behavior)
        for (i, slot) in self.entries.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(FdEntry::new(fd_type));
                return Some(i);
            }
        }
        
        // No free slot in existing entries, try to extend
        if self.entries.len() < MAX_FDS {
            let fd = self.entries.len();
            self.entries.push(Some(FdEntry::new(fd_type)));
            return Some(fd);
        }
        
        None // Table full
    }

    /// TEAM_168: Get a file descriptor entry by number.
    pub fn get(&self, fd: usize) -> Option<&FdEntry> {
        self.entries.get(fd).and_then(|e| e.as_ref())
    }

    /// TEAM_168: Get a mutable file descriptor entry by number.
    #[allow(dead_code)]
    pub fn get_mut(&mut self, fd: usize) -> Option<&mut FdEntry> {
        self.entries.get_mut(fd).and_then(|e| e.as_mut())
    }

    /// TEAM_168: Close a file descriptor.
    ///
    /// Returns true if fd was valid and closed, false otherwise.
    pub fn close(&mut self, fd: usize) -> bool {
        if let Some(slot) = self.entries.get_mut(fd) {
            if slot.is_some() {
                *slot = None;
                return true;
            }
        }
        false
    }

    /// TEAM_168: Check if a file descriptor is valid.
    #[allow(dead_code)]
    pub fn is_valid(&self, fd: usize) -> bool {
        self.get(fd).is_some()
    }
}

impl Default for FdTable {
    fn default() -> Self {
        Self::new()
    }
}

/// TEAM_168: Thread-safe wrapper for FdTable.
pub type SharedFdTable = IrqSafeLock<FdTable>;

/// TEAM_168: Create a new shared fd table.
pub fn new_shared_fd_table() -> SharedFdTable {
    IrqSafeLock::new(FdTable::new())
}
