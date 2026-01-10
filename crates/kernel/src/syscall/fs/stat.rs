use crate::fs::vfs::dispatch::*;
// TEAM_413: Use new syscall helpers
// TEAM_421: Import SyscallResult
use crate::syscall::{Stat, get_fd, write_struct_to_user, SyscallResult};
use crate::task::fd_table::FdType;

/// TEAM_168: sys_fstat - Get file status.
/// TEAM_258: Updated to use Stat constructors for architecture independence.
/// TEAM_413: Updated to use write_struct_to_user helper.
/// TEAM_421: Updated to return SyscallResult.
pub fn sys_fstat(fd: usize, stat_buf: usize) -> SyscallResult {
    let task = crate::task::current_task();

    // TEAM_413: Use get_fd helper
    let entry = get_fd(fd)?;

    let stat = match &entry.fd_type {
        // TEAM_258: Use constructor for architecture independence
        FdType::Stdin | FdType::Stdout | FdType::Stderr => {
            Stat::new_device(crate::fs::mode::S_IFCHR | 0o666, 0)
        }
        FdType::VfsFile(file) => match vfs_fstat(file) {
            Ok(s) => s,
            Err(e) => return Err(e.to_errno()),
        },
        // TEAM_258: Use constructor for architecture independence
        FdType::PipeRead(_) | FdType::PipeWrite(_) => {
            Stat::new_pipe(crate::fs::pipe::PIPE_BUF_SIZE as i32)
        }
        // TEAM_258: Use constructor for architecture independence
        FdType::PtyMaster(_) | FdType::PtySlave(_) => {
            Stat::new_device(crate::fs::mode::S_IFCHR | 0o666, 0)
        }
        // TEAM_394: Epoll and EventFd are anonymous inodes
        FdType::Epoll(_) | FdType::EventFd(_) => {
            Stat::new_device(crate::fs::mode::S_IFCHR | 0o600, 0)
        }
    };

    // TEAM_413: Use write_struct_to_user helper
    write_struct_to_user(task.ttbr0, stat_buf, &stat)?;
    Ok(0)
}

/// TEAM_409: sys_fstatat - Get file status relative to directory fd.
/// TEAM_421: Updated to return SyscallResult.
/// Signature: fstatat(dirfd, pathname, statbuf, flags)
///
/// This is the "at" variant of stat, supporting AT_FDCWD for current directory.
/// TEAM_413: Updated to use resolve_at_path and write_struct_to_user helpers.
pub fn sys_fstatat(dirfd: i32, pathname: usize, stat_buf: usize, _flags: i32) -> SyscallResult {
    use crate::syscall::resolve_at_path;

    let task = crate::task::current_task();

    // TEAM_413: Use resolve_at_path helper for pathname resolution
    let path_str = resolve_at_path(dirfd, pathname)?;

    // Get file status via VFS
    let stat = match vfs_stat(&path_str) {
        Ok(s) => s,
        Err(e) => return Err(e.to_errno()),
    };

    // TEAM_413: Use write_struct_to_user helper
    write_struct_to_user(task.ttbr0, stat_buf, &stat)?;
    Ok(0)
}
