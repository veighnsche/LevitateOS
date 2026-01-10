//! TEAM_358: statx syscall implementation
//! TEAM_421: Returns SyscallResult, no scattered casts
//!
//! Extended file stat returning struct statx with additional fields.

use crate::memory::user as mm_user;
use crate::syscall::SyscallResult;
use linux_raw_sys::errno::{EBADF, EFAULT, ENOENT};

/// AT_EMPTY_PATH flag for statx
const AT_EMPTY_PATH: i32 = 0x1000;

/// TEAM_358: statx timestamp (16 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct StatxTimestamp {
    pub tv_sec: i64,
    pub tv_nsec: u32,
    pub __reserved: i32,
}

/// TEAM_358: struct statx (256 bytes)
/// Linux extended file stat structure.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Statx {
    pub stx_mask: u32,
    pub stx_blksize: u32,
    pub stx_attributes: u64,
    pub stx_nlink: u32,
    pub stx_uid: u32,
    pub stx_gid: u32,
    pub stx_mode: u16,
    pub __spare0: [u16; 1],
    pub stx_ino: u64,
    pub stx_size: u64,
    pub stx_blocks: u64,
    pub stx_attributes_mask: u64,
    pub stx_atime: StatxTimestamp,
    pub stx_btime: StatxTimestamp,
    pub stx_ctime: StatxTimestamp,
    pub stx_mtime: StatxTimestamp,
    pub stx_rdev_major: u32,
    pub stx_rdev_minor: u32,
    pub stx_dev_major: u32,
    pub stx_dev_minor: u32,
    pub stx_mnt_id: u64,
    pub stx_dio_mem_align: u32,
    pub stx_dio_offset_align: u32,
    pub __spare3: [u64; 12],
}

// Statx mask flags
const STATX_BASIC_STATS: u32 = 0x07FF;

/// TEAM_358: sys_statx - Get extended file status.
/// TEAM_421: Returns SyscallResult
///
/// # Arguments
/// * `dirfd` - Directory file descriptor (or AT_FDCWD)
/// * `pathname` - Path to file (user pointer, may be empty if AT_EMPTY_PATH)
/// * `flags` - Flags (AT_EMPTY_PATH, AT_SYMLINK_NOFOLLOW, etc.)
/// * `mask` - What fields to return
/// * `statxbuf` - User buffer for struct statx
pub fn sys_statx(dirfd: i32, pathname: usize, flags: i32, _mask: u32, statxbuf: usize) -> SyscallResult {
    let task = crate::task::current_task();
    let statx_size = core::mem::size_of::<Statx>();

    // Validate output buffer
    if mm_user::validate_user_buffer(task.ttbr0, statxbuf, statx_size, true).is_err() {
        return Err(EFAULT);
    }

    // Handle AT_EMPTY_PATH: use dirfd as the file descriptor
    if flags & AT_EMPTY_PATH != 0 {
        return statx_by_fd(dirfd as usize, statxbuf, task.ttbr0);
    }

    // Otherwise, resolve pathname
    let mut path_buf = [0u8; 256];
    let path = crate::syscall::read_user_cstring(task.ttbr0, pathname, &mut path_buf)?;

    // Use existing VFS stat function
    use crate::fs::vfs::dispatch::vfs_stat;

    let stat = match vfs_stat(path) {
        Ok(s) => s,
        Err(_) => return Err(ENOENT),
    };

    // Convert Stat to Statx
    let statx = stat_to_statx(&stat);

    // Copy to user buffer
    copy_statx_to_user(task.ttbr0, statxbuf, &statx)
}

/// Get statx by file descriptor
/// TEAM_421: Returns SyscallResult
fn statx_by_fd(fd: usize, statxbuf: usize, ttbr0: usize) -> SyscallResult {
    use crate::fs::vfs::dispatch::vfs_fstat;
    use crate::task::fd_table::FdType;

    let task = crate::task::current_task();
    let fd_table = task.fd_table.lock();

    let entry = match fd_table.get(fd) {
        Some(e) => e,
        None => return Err(EBADF),
    };

    let stat = match entry.fd_type {
        FdType::Stdin | FdType::Stdout | FdType::Stderr => {
            crate::arch::Stat::new_device(crate::fs::mode::S_IFCHR | 0o666, 0)
        }
        FdType::VfsFile(ref file) => match vfs_fstat(file) {
            Ok(s) => s,
            Err(_) => return Err(EBADF),
        },
        FdType::PipeRead(_) | FdType::PipeWrite(_) => {
            crate::arch::Stat::new_pipe(crate::fs::pipe::PIPE_BUF_SIZE as i32)
        }
        FdType::PtyMaster(_) | FdType::PtySlave(_) => {
            crate::arch::Stat::new_device(crate::fs::mode::S_IFCHR | 0o666, 0)
        }
        // TEAM_394: Epoll and EventFd are anonymous inodes
        FdType::Epoll(_) | FdType::EventFd(_) => {
            crate::arch::Stat::new_device(crate::fs::mode::S_IFCHR | 0o600, 0)
        }
    };

    let statx = stat_to_statx(&stat);
    copy_statx_to_user(ttbr0, statxbuf, &statx)
}

/// Convert Stat to Statx
fn stat_to_statx(stat: &crate::arch::Stat) -> Statx {
    Statx {
        stx_mask: STATX_BASIC_STATS,
        stx_blksize: stat.st_blksize as u32,
        stx_attributes: 0,
        stx_nlink: stat.st_nlink,
        stx_uid: stat.st_uid,
        stx_gid: stat.st_gid,
        stx_mode: stat.st_mode as u16,
        __spare0: [0],
        stx_ino: stat.st_ino,
        stx_size: stat.st_size as u64,
        stx_blocks: stat.st_blocks as u64,
        stx_attributes_mask: 0,
        stx_atime: StatxTimestamp {
            tv_sec: stat.st_atime,
            tv_nsec: 0,
            __reserved: 0,
        },
        stx_btime: StatxTimestamp::default(), // Birth time not tracked
        stx_ctime: StatxTimestamp {
            tv_sec: stat.st_ctime,
            tv_nsec: 0,
            __reserved: 0,
        },
        stx_mtime: StatxTimestamp {
            tv_sec: stat.st_mtime,
            tv_nsec: 0,
            __reserved: 0,
        },
        stx_rdev_major: (stat.st_rdev >> 8) as u32,
        stx_rdev_minor: (stat.st_rdev & 0xFF) as u32,
        stx_dev_major: (stat.st_dev >> 8) as u32,
        stx_dev_minor: (stat.st_dev & 0xFF) as u32,
        stx_mnt_id: 0,
        stx_dio_mem_align: 0,
        stx_dio_offset_align: 0,
        __spare3: [0; 12],
    }
}

/// Copy Statx struct to user buffer
/// TEAM_421: Returns SyscallResult
///
/// # Safety
/// Caller must have validated the user buffer with `validate_user_buffer` first.
fn copy_statx_to_user(ttbr0: usize, statxbuf: usize, statx: &Statx) -> SyscallResult {
    let statx_size = core::mem::size_of::<Statx>();

    // TEAM_416: Replace unwrap() with proper error handling for panic safety
    let dest = match mm_user::user_va_to_kernel_ptr(ttbr0, statxbuf) {
        Some(p) => p,
        None => return Err(EFAULT),
    };
    unsafe {
        core::ptr::copy_nonoverlapping(statx as *const Statx as *const u8, dest, statx_size);
    }

    Ok(0)
}
