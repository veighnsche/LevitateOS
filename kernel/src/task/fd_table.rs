//! TEAM_168: File Descriptor Table for LevitateOS.
//!
//! Per-process file descriptor management for syscalls.
//! Supports stdin/stdout/stderr (fd 0/1/2), initramfs files, and tmpfs files.

use alloc::vec::Vec;
use los_hal::IrqSafeLock;

/// TEAM_168: Maximum number of open file descriptors per process.
pub const MAX_FDS: usize = 64;

use crate::fs::pipe::PipeRef;
use crate::fs::vfs::file::FileRef;

/// TEAM_168: Type of file descriptor entry.
/// TEAM_194: Removed Copy derive to support Arc<> for tmpfs nodes.
/// TEAM_195: Removed Debug derive since Mutex<TmpfsNode> doesn't implement Debug.
/// TEAM_203: Added VfsFile variant and removed legacy Tmpfs variants.
/// TEAM_233: Added PipeRead and PipeWrite variants for pipe support.
#[derive(Clone)]
pub enum FdType {
    /// Standard input (keyboard)
    Stdin,
    /// Standard output (console)
    Stdout,
    /// Standard error (console)
    Stderr,
    /// TEAM_203: Generic VFS file (used for tmpfs, FAT32, etc.)
    VfsFile(FileRef),
    /// TEAM_233: Read end of a pipe
    PipeRead(PipeRef),
    /// TEAM_233: Write end of a pipe
    PipeWrite(PipeRef),
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
        entries.push(Some(FdEntry::new(FdType::Stdin))); // fd 0
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
    /// TEAM_240: Now properly closes pipe read/write ends.
    ///
    /// Returns true if fd was valid and closed, false otherwise.
    pub fn close(&mut self, fd: usize) -> bool {
        if let Some(slot) = self.entries.get_mut(fd) {
            if let Some(entry) = slot.take() {
                // TEAM_240: Notify pipe when closing pipe fds
                match &entry.fd_type {
                    FdType::PipeRead(pipe) => pipe.close_read(),
                    FdType::PipeWrite(pipe) => pipe.close_write(),
                    _ => {}
                }
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

    /// TEAM_233: Duplicate a file descriptor to the lowest available slot.
    ///
    /// Returns the new fd number on success, or None if oldfd is invalid or table is full.
    pub fn dup(&mut self, oldfd: usize) -> Option<usize> {
        // Get the FdType from oldfd
        let fd_type = self.get(oldfd)?.fd_type.clone();
        // Allocate a new fd with the same type
        self.alloc(fd_type)
    }

    /// TEAM_233: Duplicate a file descriptor to a specific slot.
    ///
    /// If newfd is already open, it is closed first.
    /// Returns newfd on success, or None if oldfd is invalid.
    pub fn dup_to(&mut self, oldfd: usize, newfd: usize) -> Option<usize> {
        if oldfd == newfd {
            return None; // dup3 returns EINVAL for this
        }
        if newfd >= MAX_FDS {
            return None;
        }

        // Get the FdType from oldfd
        let fd_type = self.get(oldfd)?.fd_type.clone();

        // Ensure entries vector is large enough
        while self.entries.len() <= newfd {
            self.entries.push(None);
        }

        // TEAM_240: Close newfd if open (with proper pipe cleanup)
        if let Some(entry) = self.entries[newfd].take() {
            match &entry.fd_type {
                FdType::PipeRead(pipe) => pipe.close_read(),
                FdType::PipeWrite(pipe) => pipe.close_write(),
                _ => {}
            }
        }

        // Set newfd to point to same fd_type
        self.entries[newfd] = Some(FdEntry::new(fd_type));
        Some(newfd)
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
