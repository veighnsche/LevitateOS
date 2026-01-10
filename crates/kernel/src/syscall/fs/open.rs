use crate::fs::vfs::dispatch::*;
use crate::fs::vfs::error::VfsError;
use crate::fs::vfs::file::OpenFlags;
use crate::syscall::{errno, fcntl, read_user_cstring};
use crate::task::fd_table::FdType;

/// TEAM_345: sys_openat - Linux ABI compatible.
/// Signature: openat(dirfd, pathname, flags, mode)
///
/// TEAM_168: Original implementation.
/// TEAM_176: Updated to support opening directories for getdents.
/// TEAM_194: Updated to support tmpfs at /tmp with O_CREAT and O_TRUNC.
pub fn sys_openat(dirfd: i32, pathname: usize, flags: u32, _mode: u32) -> i64 {
    let task = crate::task::current_task();

    // TEAM_345: Read null-terminated pathname (Linux ABI)
    let mut path_buf = [0u8; crate::syscall::constants::PATH_MAX];
    let path_str = match read_user_cstring(task.ttbr0, pathname, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };

    // TEAM_345: Handle dirfd (AT_FDCWD means use cwd)
    // For now, we only support AT_FDCWD - relative paths with other dirfd not yet implemented
    if dirfd != fcntl::AT_FDCWD && !path_str.starts_with('/') {
        // TODO(TEAM_345): Implement dirfd-relative path resolution
        log::warn!("[SYSCALL] openat: dirfd {} not yet supported for relative paths", dirfd);
        return errno::EBADF;
    }

    // TEAM_247: Handle PTY devices
    if path_str == "/dev/ptmx" {
        if let Some(pair) = crate::fs::tty::pty::allocate_pty() {
            let mut fd_table = task.fd_table.lock();
            match fd_table.alloc(FdType::PtyMaster(pair)) {
                Some(fd) => return fd as i64,
                None => return errno::EMFILE,
            }
        }
        return errno::ENOMEM;
    }

    if path_str.starts_with("/dev/pts/") {
        if let Ok(id) = path_str[9..].parse::<usize>() {
            if let Some(pair) = crate::fs::tty::pty::get_pty(id) {
                let mut fd_table = task.fd_table.lock();
                match fd_table.alloc(FdType::PtySlave(pair)) {
                    Some(fd) => return fd as i64,
                    None => return errno::EMFILE,
                }
            }
        }
        return errno::ENOENT;
    }

    // TEAM_205: All paths now go through generic vfs_open
    let vfs_flags = OpenFlags::new(flags);
    match vfs_open(path_str, vfs_flags, 0o666) {
        Ok(file) => {
            let mut fd_table = task.fd_table.lock();
            match fd_table.alloc(FdType::VfsFile(file)) {
                Some(fd) => fd as i64,
                None => errno::EMFILE,
            }
        }
        Err(VfsError::NotFound) => errno::ENOENT,
        Err(VfsError::AlreadyExists) => errno::EEXIST,
        Err(VfsError::NotADirectory) => errno::ENOTDIR,
        Err(VfsError::IsADirectory) => {
            errno::EIO // Should not normally happen if vfs_open succeeded
        }
        Err(_) => errno::EIO,
    }
}

/// TEAM_168: sys_close - Close a file descriptor.
pub fn sys_close(fd: usize) -> i64 {
    let task = crate::task::current_task();
    let mut fd_table = task.fd_table.lock();

    if fd < 3 {
        return errno::EINVAL;
    }

    if fd_table.close(fd) { 0 } else { errno::EBADF }
}

// ============================================================================
// TEAM_350: faccessat - Check file accessibility
// ============================================================================

/// TEAM_350: Access mode flags for faccessat
pub mod access_mode {
    pub const F_OK: i32 = 0; // Test for existence
    pub const X_OK: i32 = 1; // Test for execute permission
    pub const W_OK: i32 = 2; // Test for write permission
    pub const R_OK: i32 = 4; // Test for read permission
}

/// TEAM_350: sys_faccessat - Check file accessibility.
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
/// 0 if access is permitted, negative errno otherwise.
#[allow(unused_variables)]
pub fn sys_faccessat(dirfd: i32, pathname: usize, mode: i32, flags: i32) -> i64 {
    use crate::fs::vfs::dispatch::vfs_access;

    let task = crate::task::current_task();

    // TEAM_418: Use PATH_MAX from SSOT
    let mut path_buf = [0u8; crate::syscall::constants::PATH_MAX];
    let path_str = match read_user_cstring(task.ttbr0, pathname, &mut path_buf) {
        Ok(s) => s,
        Err(e) => return e,
    };

    log::trace!(
        "[SYSCALL] faccessat(dirfd={}, path='{}', mode=0x{:x}, flags=0x{:x})",
        dirfd,
        path_str,
        mode,
        flags
    );

    // Handle dirfd (AT_FDCWD means use cwd)
    if dirfd != fcntl::AT_FDCWD && !path_str.starts_with('/') {
        log::warn!(
            "[SYSCALL] faccessat: dirfd {} not yet supported for relative paths",
            dirfd
        );
        return errno::EBADF;
    }

    // TEAM_350: For single-user OS, we only check existence
    // R_OK, W_OK, X_OK always succeed if file exists (we're root)
    match vfs_access(path_str, mode as u32) {
        Ok(_) => 0, // File exists, access granted
        Err(crate::fs::vfs::error::VfsError::NotFound) => errno::ENOENT,
        Err(crate::fs::vfs::error::VfsError::NotADirectory) => errno::ENOTDIR,
        Err(_) => errno::EACCES,
    }
}
