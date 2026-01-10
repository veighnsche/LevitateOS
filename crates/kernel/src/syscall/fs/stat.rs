use crate::memory::user as mm_user;

use crate::fs::vfs::dispatch::*;
use crate::syscall::{Stat, errno};
use crate::task::fd_table::FdType;

/// TEAM_168: sys_fstat - Get file status.
/// TEAM_258: Updated to use Stat constructors for architecture independence.
pub fn sys_fstat(fd: usize, stat_buf: usize) -> i64 {
    let task = crate::task::current_task();
    let stat_size = core::mem::size_of::<Stat>();
    if mm_user::validate_user_buffer(task.ttbr0, stat_buf, stat_size, true).is_err() {
        return errno::EFAULT;
    }

    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd) {
        Some(e) => e,
        None => return errno::EBADF,
    };

    let stat = match entry.fd_type {
        // TEAM_258: Use constructor for architecture independence
        FdType::Stdin | FdType::Stdout | FdType::Stderr => {
            Stat::new_device(crate::fs::mode::S_IFCHR | 0o666, 0)
        }
        FdType::VfsFile(ref file) => match vfs_fstat(file) {
            Ok(s) => s,
            Err(_) => return errno::EBADF,
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

    // SAFETY: validate_user_buffer already confirmed the entire buffer is accessible
    let dest = mm_user::user_va_to_kernel_ptr(task.ttbr0, stat_buf).unwrap();
    unsafe {
        core::ptr::copy_nonoverlapping(&stat as *const Stat as *const u8, dest, stat_size);
    }

    0
}
