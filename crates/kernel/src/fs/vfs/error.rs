//! TEAM_202: VFS Error Types
//!
//! Defines the error type for all VFS operations.

use core::fmt;

use crate::block::BlockError;
use crate::fs::FsError;

/// TEAM_202: VFS Error codes
///
/// These map to standard POSIX errno values for userspace compatibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VfsError {
    /// Operation not permitted (EPERM = 1)
    PermissionDenied,
    /// No such file or directory (ENOENT = 2)
    NotFound,
    /// I/O error (EIO = 5)
    IoError,
    /// Bad file descriptor (EBADF = 9)
    BadFd,
    /// Resource temporarily unavailable (EAGAIN = 11)
    WouldBlock,
    /// Out of memory (ENOMEM = 12)
    OutOfMemory,
    /// Permission denied (EACCES = 13)
    AccessDenied,
    /// Bad address (EFAULT = 14)
    BadAddress,
    /// Device or resource busy (EBUSY = 16)
    Busy,
    /// File exists (EEXIST = 17)
    AlreadyExists,
    /// Not a directory (ENOTDIR = 20)
    NotADirectory,
    /// Is a directory (EISDIR = 21)
    IsADirectory,
    /// Invalid argument (EINVAL = 22)
    InvalidArgument,
    /// Too many open files in system (ENFILE = 23)
    TooManyOpenFiles,
    /// File too large (EFBIG = 27)
    FileTooLarge,
    /// No space left on device (ENOSPC = 28)
    NoSpace,
    /// Read-only file system (EROFS = 30)
    ReadOnlyFs,
    /// Too many links (EMLINK = 31)
    TooManyLinks,
    /// Directory not empty (ENOTEMPTY = 39)
    DirectoryNotEmpty,
    /// No data available (ENODATA = 61)
    NoData,
    /// Operation not supported (EOPNOTSUPP = 95)
    NotSupported,
    /// Name too long (ENAMETOOLONG = 36)
    NameTooLong,
    /// Stale file handle (ESTALE = 116)
    StaleHandle,
    /// Invalid cross-device link (EXDEV = 18)
    CrossDevice,
    /// Not a symbolic link (EINVAL = 22)
    NotASymlink,
    /// Internal kernel error (EIO = 5)
    InternalError,
}

impl VfsError {
    /// TEAM_202: Convert to POSIX errno value (negative)
    pub fn to_errno(self) -> i64 {
        match self {
            VfsError::PermissionDenied => -1,
            VfsError::NotFound => -2,
            VfsError::IoError => -5,
            VfsError::BadFd => -9,
            VfsError::WouldBlock => -11,
            VfsError::OutOfMemory => -12,
            VfsError::AccessDenied => -13,
            VfsError::BadAddress => -14,
            VfsError::Busy => -16,
            VfsError::AlreadyExists => -17,
            VfsError::NotADirectory => -20,
            VfsError::IsADirectory => -21,
            VfsError::InvalidArgument => -22,
            VfsError::TooManyOpenFiles => -23,
            VfsError::FileTooLarge => -27,
            VfsError::NoSpace => -28,
            VfsError::ReadOnlyFs => -30,
            VfsError::TooManyLinks => -31,
            VfsError::DirectoryNotEmpty => -39,
            VfsError::NoData => -61,
            VfsError::NotSupported => -95,
            VfsError::NameTooLong => -36,
            VfsError::StaleHandle => -116,
            VfsError::CrossDevice => -18,
            VfsError::NotASymlink => -22,
            VfsError::InternalError => -5,
        }
    }

    /// TEAM_202: Get error name
    pub fn name(&self) -> &'static str {
        match self {
            VfsError::PermissionDenied => "EPERM",
            VfsError::NotFound => "ENOENT",
            VfsError::IoError => "EIO",
            VfsError::BadFd => "EBADF",
            VfsError::WouldBlock => "EAGAIN",
            VfsError::OutOfMemory => "ENOMEM",
            VfsError::AccessDenied => "EACCES",
            VfsError::BadAddress => "EFAULT",
            VfsError::Busy => "EBUSY",
            VfsError::AlreadyExists => "EEXIST",
            VfsError::NotADirectory => "ENOTDIR",
            VfsError::IsADirectory => "EISDIR",
            VfsError::InvalidArgument => "EINVAL",
            VfsError::TooManyOpenFiles => "ENFILE",
            VfsError::FileTooLarge => "EFBIG",
            VfsError::NoSpace => "ENOSPC",
            VfsError::ReadOnlyFs => "EROFS",
            VfsError::TooManyLinks => "EMLINK",
            VfsError::DirectoryNotEmpty => "ENOTEMPTY",
            VfsError::NoData => "ENODATA",
            VfsError::NotSupported => "EOPNOTSUPP",
            VfsError::NameTooLong => "ENAMETOOLONG",
            VfsError::StaleHandle => "ESTALE",
            VfsError::CrossDevice => "EXDEV",
            VfsError::NotASymlink => "EINVAL",
            VfsError::InternalError => "EIO",
        }
    }
}

impl fmt::Display for VfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            VfsError::PermissionDenied => "Operation not permitted",
            VfsError::NotFound => "No such file or directory",
            VfsError::IoError => "I/O error",
            VfsError::BadFd => "Bad file descriptor",
            VfsError::WouldBlock => "Resource temporarily unavailable",
            VfsError::OutOfMemory => "Out of memory",
            VfsError::AccessDenied => "Permission denied",
            VfsError::BadAddress => "Bad address",
            VfsError::Busy => "Device or resource busy",
            VfsError::AlreadyExists => "File exists",
            VfsError::NotADirectory => "Not a directory",
            VfsError::IsADirectory => "Is a directory",
            VfsError::InvalidArgument => "Invalid argument",
            VfsError::TooManyOpenFiles => "Too many open files",
            VfsError::FileTooLarge => "File too large",
            VfsError::NoSpace => "No space left on device",
            VfsError::ReadOnlyFs => "Read-only file system",
            VfsError::TooManyLinks => "Too many links",
            VfsError::DirectoryNotEmpty => "Directory not empty",
            VfsError::NoData => "No data available",
            VfsError::NotSupported => "Operation not supported",
            VfsError::NameTooLong => "File name too long",
            VfsError::StaleHandle => "Stale file handle",
            VfsError::CrossDevice => "Invalid cross-device link",
            VfsError::NotASymlink => "Not a symbolic link",
            VfsError::InternalError => "Internal kernel error",
        };
        write!(f, "{} ({})", msg, self.name())
    }
}

// TEAM_219: Error Mappings
impl From<BlockError> for VfsError {
    fn from(err: BlockError) -> Self {
        match err {
            BlockError::NotInitialized => VfsError::IoError,
            BlockError::ReadFailed => VfsError::IoError,
            BlockError::WriteFailed => VfsError::IoError,
            BlockError::InvalidBufferSize => VfsError::InternalError,
        }
    }
}

impl From<FsError> for VfsError {
    fn from(err: FsError) -> Self {
        match err {
            // General FS errors
            FsError::VolumeOpen => VfsError::IoError,
            FsError::DirOpen => VfsError::IoError,
            FsError::FileOpen => VfsError::NotFound,
            FsError::ReadError => VfsError::IoError,
            FsError::WriteError => VfsError::IoError,
            FsError::NotMounted => VfsError::IoError,
            // Wrap inner BlockError
            FsError::BlockError(e) => e.into(),
        }
    }
}

/// TEAM_202: Result type for VFS operations
pub type VfsResult<T> = Result<T, VfsError>;
