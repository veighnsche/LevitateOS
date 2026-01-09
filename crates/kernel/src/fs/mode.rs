//! TEAM_201: POSIX File Mode Constants
//!
//! Defines standard POSIX file type and permission bit constants for st_mode.

// ============================================================================
// File Type Constants (high bits of st_mode)
// ============================================================================

/// TEAM_201: Bit mask for extracting file type
pub const S_IFMT: u32 = 0o170000;

/// TEAM_201: Socket
pub const S_IFSOCK: u32 = 0o140000;

/// TEAM_201: Symbolic link
pub const S_IFLNK: u32 = 0o120000;

/// TEAM_201: Regular file
pub const S_IFREG: u32 = 0o100000;

/// TEAM_201: Block device
pub const S_IFBLK: u32 = 0o060000;

/// TEAM_201: Directory
pub const S_IFDIR: u32 = 0o040000;

/// TEAM_201: Character device
pub const S_IFCHR: u32 = 0o020000;

/// TEAM_201: FIFO (named pipe)
pub const S_IFIFO: u32 = 0o010000;

// ============================================================================
// Permission Bits (low bits of st_mode)
// ============================================================================

/// TEAM_201: Set user ID on execution
pub const S_ISUID: u32 = 0o4000;

/// TEAM_201: Set group ID on execution
pub const S_ISGID: u32 = 0o2000;

/// TEAM_201: Sticky bit
pub const S_ISVTX: u32 = 0o1000;

/// TEAM_201: Owner read/write/execute
pub const S_IRWXU: u32 = 0o0700;

/// TEAM_201: Owner read permission
pub const S_IRUSR: u32 = 0o0400;

/// TEAM_201: Owner write permission
pub const S_IWUSR: u32 = 0o0200;

/// TEAM_201: Owner execute permission
pub const S_IXUSR: u32 = 0o0100;

/// TEAM_201: Group read/write/execute
pub const S_IRWXG: u32 = 0o0070;

/// TEAM_201: Group read permission
pub const S_IRGRP: u32 = 0o0040;

/// TEAM_201: Group write permission
pub const S_IWGRP: u32 = 0o0020;

/// TEAM_201: Group execute permission
pub const S_IXGRP: u32 = 0o0010;

/// TEAM_201: Others read/write/execute
pub const S_IRWXO: u32 = 0o0007;

/// TEAM_201: Others read permission
pub const S_IROTH: u32 = 0o0004;

/// TEAM_201: Others write permission
pub const S_IWOTH: u32 = 0o0002;

/// TEAM_201: Others execute permission
pub const S_IXOTH: u32 = 0o0001;

// ============================================================================
// Helper Functions
// ============================================================================

/// TEAM_201: Check if mode indicates a regular file
#[inline]
pub const fn is_reg(mode: u32) -> bool {
    (mode & S_IFMT) == S_IFREG
}

/// TEAM_201: Check if mode indicates a directory
#[inline]
pub const fn is_dir(mode: u32) -> bool {
    (mode & S_IFMT) == S_IFDIR
}

/// TEAM_201: Check if mode indicates a symbolic link
#[inline]
pub const fn is_lnk(mode: u32) -> bool {
    (mode & S_IFMT) == S_IFLNK
}

/// TEAM_201: Check if mode indicates a character device
#[inline]
pub const fn is_chr(mode: u32) -> bool {
    (mode & S_IFMT) == S_IFCHR
}

/// TEAM_201: Check if mode indicates a block device
#[inline]
pub const fn is_blk(mode: u32) -> bool {
    (mode & S_IFMT) == S_IFBLK
}

/// TEAM_201: Check if mode indicates a FIFO
#[inline]
pub const fn is_fifo(mode: u32) -> bool {
    (mode & S_IFMT) == S_IFIFO
}

/// TEAM_201: Check if mode indicates a socket
#[inline]
pub const fn is_sock(mode: u32) -> bool {
    (mode & S_IFMT) == S_IFSOCK
}

/// TEAM_201: Extract just the file type from mode
#[inline]
pub const fn file_type(mode: u32) -> u32 {
    mode & S_IFMT
}

/// TEAM_201: Extract just the permission bits from mode
#[inline]
pub const fn permissions(mode: u32) -> u32 {
    mode & 0o7777
}

/// TEAM_201: Create a mode with file type and permissions
#[inline]
pub const fn make_mode(file_type: u32, perms: u32) -> u32 {
    (file_type & S_IFMT) | (perms & 0o7777)
}
