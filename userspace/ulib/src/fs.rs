//! TEAM_168: File system abstractions for LevitateOS userspace.
//!
//! Provides a `File` type similar to `std::fs::File`.
//! TEAM_176: Added `ReadDir` and `DirEntry` for directory iteration.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use crate::io::{Error, ErrorKind, Read, Result};

/// TEAM_168: An open file handle.
///
/// When dropped, the file is automatically closed.
///
/// # Example
/// ```rust
/// use ulib::fs::File;
///
/// let file = File::open("hello.txt")?;
/// let mut buf = [0u8; 100];
/// let n = file.read(&mut buf)?;
/// ```
pub struct File {
    fd: usize,
}

impl File {
    /// TEAM_168: Open a file for reading.
    ///
    /// # Arguments
    /// * `path` - Path to the file (in initramfs)
    ///
    /// # Returns
    /// The opened file, or an error.
    pub fn open(path: &str) -> Result<Self> {
        let fd = libsyscall::openat(path, 0); // 0 = read-only
        if fd < 0 {
            return Err(Error::from_errno(fd));
        }
        Ok(Self { fd: fd as usize })
    }

    /// TEAM_168: Get the file descriptor number.
    pub fn as_raw_fd(&self) -> usize {
        self.fd
    }

    /// TEAM_168: Get file metadata.
    pub fn metadata(&self) -> Result<Metadata> {
        let mut stat = libsyscall::Stat::default();
        let ret = libsyscall::fstat(self.fd, &mut stat);
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }
        Ok(Metadata {
            size: stat.st_size,
            mode: stat.st_mode,
        })
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // TEAM_178: Call kernel syscall for file read
        let ret = libsyscall::read(self.fd, buf);
        if ret < 0 {
            Err(Error::from_errno(ret))
        } else {
            Ok(ret as usize)
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        // Ignore errors on close - nothing we can do
        let _ = libsyscall::close(self.fd);
    }
}

/// TEAM_168: File metadata.
#[derive(Debug, Clone, Copy)]
pub struct Metadata {
    /// File size in bytes
    pub size: u64,
    /// TEAM_182: File mode (1=file, 2=dir/device)
    mode: u32,
}

impl Metadata {
    /// TEAM_168: Get the file size.
    pub fn len(&self) -> u64 {
        self.size
    }

    /// TEAM_168: Check if file is empty.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// TEAM_168: Check if this is a regular file.
    pub fn is_file(&self) -> bool {
        self.mode == 1
    }

    /// TEAM_182: Check if this is a directory.
    pub fn is_dir(&self) -> bool {
        self.mode == 2
    }
}

// ============================================================================
// Directory Iteration (TEAM_176)
// ============================================================================

/// TEAM_176: File type from directory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Regular file
    File,
    /// Directory
    Directory,
    /// Symbolic link
    Symlink,
    /// Other (device, socket, etc.)
    Other,
}

impl FileType {
    /// TEAM_176: Parse from dirent d_type field.
    fn from_d_type(d_type: u8) -> Self {
        match d_type {
            libsyscall::d_type::DT_REG => Self::File,
            libsyscall::d_type::DT_DIR => Self::Directory,
            libsyscall::d_type::DT_LNK => Self::Symlink,
            _ => Self::Other,
        }
    }

    /// Check if this is a regular file.
    pub fn is_file(&self) -> bool {
        matches!(self, Self::File)
    }

    /// Check if this is a directory.
    pub fn is_dir(&self) -> bool {
        matches!(self, Self::Directory)
    }

    /// Check if this is a symbolic link.
    pub fn is_symlink(&self) -> bool {
        matches!(self, Self::Symlink)
    }
}

/// TEAM_176: A single directory entry.
#[derive(Debug, Clone)]
pub struct DirEntry {
    name: String,
    file_type: FileType,
    ino: u64,
}

impl DirEntry {
    /// Get the file name (without path).
    pub fn file_name(&self) -> &str {
        &self.name
    }

    /// Get the file type.
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    /// Get the inode number.
    pub fn ino(&self) -> u64 {
        self.ino
    }
}

/// TEAM_176: Iterator over directory entries.
///
/// # Example
/// ```rust
/// use ulib::fs::read_dir;
///
/// for entry in read_dir("/")? {
///     let entry = entry?;
///     println!("{}: {:?}", entry.file_name(), entry.file_type());
/// }
/// ```
pub struct ReadDir {
    fd: usize,
    buf: Vec<u8>,
    pos: usize,
    end: usize,
    finished: bool,
}

/// TEAM_176: Buffer size for ReadDir (Q1 decision: 4096 bytes).
const READDIR_BUF_SIZE: usize = 4096;

impl ReadDir {
    /// TEAM_176: Open a directory for iteration.
    pub fn open(path: &str) -> Result<Self> {
        let fd = libsyscall::openat(path, 0);
        if fd < 0 {
            return Err(Error::from_errno(fd));
        }

        Ok(Self {
            fd: fd as usize,
            buf: alloc::vec![0u8; READDIR_BUF_SIZE],
            pos: 0,
            end: 0,
            finished: false,
        })
    }

    /// TEAM_176: Fetch more entries from kernel.
    fn fill_buffer(&mut self) -> Result<()> {
        let ret = libsyscall::getdents(self.fd, &mut self.buf);
        if ret < 0 {
            return Err(Error::from_errno(ret));
        }
        self.pos = 0;
        self.end = ret as usize;
        if ret == 0 {
            self.finished = true;
        }
        Ok(())
    }

    /// TEAM_176: Parse next entry from buffer.
    fn parse_entry(&mut self) -> Option<Result<DirEntry>> {
        if self.pos >= self.end {
            return None;
        }

        // Read dirent64 header (packed struct: 8 + 8 + 2 + 1 = 19 bytes)
        if self.pos + 19 > self.end {
            return None;
        }

        let buf = &self.buf[self.pos..];
        
        // Parse header fields (careful with packed struct alignment)
        let d_ino = u64::from_le_bytes([
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ]);
        let d_reclen = u16::from_le_bytes([buf[16], buf[17]]) as usize;
        let d_type = buf[18];

        if self.pos + d_reclen > self.end || d_reclen < 19 {
            return None;
        }

        // Parse name (starts at offset 19, null-terminated)
        let name_start = 19;
        let name_end = d_reclen;
        let name_bytes = &buf[name_start..name_end];
        
        // Find null terminator
        let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len());
        let name = match core::str::from_utf8(&name_bytes[..name_len]) {
            Ok(s) => String::from(s),
            Err(_) => return Some(Err(Error::new(ErrorKind::InvalidArgument))),
        };

        self.pos += d_reclen;

        Some(Ok(DirEntry {
            name,
            file_type: FileType::from_d_type(d_type),
            ino: d_ino,
        }))
    }
}

impl Iterator for ReadDir {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to parse from current buffer
            if let Some(entry) = self.parse_entry() {
                return Some(entry);
            }

            // Buffer exhausted, try to fetch more
            if self.finished {
                return None;
            }

            if let Err(e) = self.fill_buffer() {
                return Some(Err(e));
            }

            if self.finished {
                return None;
            }
        }
    }
}

impl Drop for ReadDir {
    fn drop(&mut self) {
        let _ = libsyscall::close(self.fd);
    }
}

/// TEAM_176: Convenience function to read a directory.
///
/// # Example
/// ```rust
/// use ulib::fs::read_dir;
///
/// for entry in read_dir("/")? {
///     let entry = entry?;
///     println!("{}", entry.file_name());
/// }
/// ```
pub fn read_dir(path: &str) -> Result<ReadDir> {
    ReadDir::open(path)
}
