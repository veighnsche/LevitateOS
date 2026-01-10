use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
use crate::fs::vfs::file::OpenFlags;
// TEAM_420: Direct linux_raw_sys imports, no shims
// TEAM_421: Import SyscallResult
use crate::syscall::{read_user_cstring, SyscallResult};
use crate::task::fd_table::FdType;
use linux_raw_sys::errno::{EACCES, EBADF, EEXIST, EINVAL, EIO, EMFILE, ENOMEM, ENOENT, ENOTDIR};
use linux_raw_sys::general::AT_FDCWD;

/// TEAM_345: sys_openat - Linux ABI compatible.
/// TEAM_421: Updated to return SyscallResult.
/// Signature: openat(dirfd, pathname, flags, mode)
///
/// TEAM_168: Original implementation.
/// TEAM_176: Updated to support opening directories for getdents.
/// TEAM_194: Updated to support tmpfs at /tmp with O_CREAT and O_TRUNC.
pub fn sys_openat(dirfd: i32, pathname: usize, flags: u32, _mode: u32) -> SyscallResult {
    let task = crate::task::current_task();

    // TEAM_345: Read null-terminated pathname (Linux ABI)
    let mut path_buf = [0u8; linux_raw_sys::general::PATH_MAX as usize];
    let path_str = read_user_cstring(task.ttbr0, pathname, &mut path_buf)?;

    // TEAM_345: Handle dirfd (AT_FDCWD means use cwd)
    // For now, we only support AT_FDCWD - relative paths with other dirfd not yet implemented
    if dirfd != AT_FDCWD && !path_str.starts_with('/') {
        // TODO(TEAM_345): Implement dirfd-relative path resolution
        log::warn!("[SYSCALL] openat: dirfd {} not yet supported for relative paths", dirfd);
        return Err(EBADF);
    }

    // TEAM_247: Handle PTY devices
    if path_str == "/dev/ptmx" {
        if let Some(pair) = crate::fs::tty::pty::allocate_pty() {
            let mut fd_table = task.fd_table.lock();
            match fd_table.alloc(FdType::PtyMaster(pair)) {
                Some(fd) => return Ok(fd as i64),
                None => return Err(EMFILE),
            }
        }
        return Err(ENOMEM);
    }

    if path_str.starts_with("/dev/pts/") {
        if let Ok(id) = path_str[9..].parse::<usize>() {
            if let Some(pair) = crate::fs::tty::pty::get_pty(id) {
                let mut fd_table = task.fd_table.lock();
                match fd_table.alloc(FdType::PtySlave(pair)) {
                    Some(fd) => return Ok(fd as i64),
                    None => return Err(EMFILE),
                }
            }
        }
        return Err(ENOENT);
    }

    // TEAM_205: All paths now go through generic vfs_open
    let vfs_flags = OpenFlags::new(flags);
    match vfs_open(path_str, vfs_flags, 0o666) {
        Ok(file) => {
            let mut fd_table = task.fd_table.lock();
            match fd_table.alloc(FdType::VfsFile(file)) {
                Some(fd) => Ok(fd as i64),
                None => Err(EMFILE),
            }
        }
        Err(VfsError::NotFound) => Err(ENOENT),
        Err(VfsError::AlreadyExists) => Err(EEXIST),
        Err(VfsError::NotADirectory) => Err(ENOTDIR),
        Err(VfsError::IsADirectory) => {
            Err(EIO) // Should not normally happen if vfs_open succeeded
        }
        Err(_) => Err(EIO),
    }
}

/// TEAM_168: sys_close - Close a file descriptor.
/// TEAM_421: Updated to return SyscallResult.
pub fn sys_close(fd: usize) -> SyscallResult {
    let task = crate::task::current_task();
    let mut fd_table = task.fd_table.lock();

    if fd < 3 {
        return Err(EINVAL);
    }

    if fd_table.close(fd) { Ok(0) } else { Err(EBADF) }
}

// ============================================================================
// TEAM_350: faccessat - Check file accessibility
// ============================================================================

/// TEAM_419: Access mode flags from linux-raw-sys
pub mod access_mode {
    pub use linux_raw_sys::general::{F_OK, X_OK, W_OK, R_OK};
}

/// TEAM_350: sys_faccessat - Check file accessibility.
/// TEAM_421: Updated to return SyscallResult.
///
/// Checks whether the calling process can access the file pathname.
/// For LevitateOS (single-user, root), we only check file existence.
///
/// # Arguments
/// * `dirfd` - Directory file descriptor (AT_FDCWD for cwd)
/// * `pathname` - Path to check
/// * `mode` - Access mode (F_OK, R_OK, W_OK, X_OK)
/// * `flags` - Flags (AT_SYMLINK_NOFOLLOW, etc.)
///
/// # Returns
/// Ok(0) if access is permitted, Err(errno) otherwise.
#[allow(unused_variables)]
pub fn sys_faccessat(dirfd: i32, pathname: usize, mode: i32, flags: i32) -> SyscallResult {
    use crate::fs::vfs::dispatch::vfs_access;

    let task = crate::task::current_task();

    // TEAM_418: Use PATH_MAX from SSOT
    let mut path_buf = [0u8; linux_raw_sys::general::PATH_MAX as usize];
    let path_str = read_user_cstring(task.ttbr0, pathname, &mut path_buf)?;

    log::trace!(
        "[SYSCALL] faccessat(dirfd={}, path='{}', mode=0x{:x}, flags=0x{:x})",
        dirfd,
        path_str,
        mode,
        flags
    );

    // Handle dirfd (AT_FDCWD means use cwd)
    if dirfd != AT_FDCWD && !path_str.starts_with('/') {
        log::warn!(
            "[SYSCALL] faccessat: dirfd {} not yet supported for relative paths",
            dirfd
        );
        return Err(EBADF);
    }

    // TEAM_350: For single-user OS, we only check existence
    // R_OK, W_OK, X_OK always succeed if file exists (we're root)
    match vfs_access(path_str, mode as u32) {
        Ok(_) => Ok(0), // File exists, access granted
        Err(crate::fs::vfs::error::VfsError::NotFound) => Err(ENOENT),
        Err(crate::fs::vfs::error::VfsError::NotADirectory) => Err(ENOTDIR),
        Err(_) => Err(EACCES),
    }
}
