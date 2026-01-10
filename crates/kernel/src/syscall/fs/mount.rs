// TEAM_206: Mount syscalls
// TEAM_421: Returns SyscallResult, no scattered casts

use crate::syscall::SyscallResult;
use linux_raw_sys::errno::{EACCES, EBUSY, EFAULT, EINVAL, ENOENT};
use core::convert::TryFrom;

// TEAM_206: Mount a filesystem
/// TEAM_421: Returns SyscallResult
pub fn sys_mount(
    source_ptr: usize,
    target_ptr: usize,
    fstype_ptr: usize,
    flags: usize,
    _data_ptr: usize,
) -> SyscallResult {
    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;

    // Read source string
    let source = match crate::syscall::sys::read_user_string(ttbr0, source_ptr, 256) {
        Ok(s) => s,
        Err(_) => return Err(EFAULT),
    };

    // Read target string
    let target = match crate::syscall::sys::read_user_string(ttbr0, target_ptr, 256) {
        Ok(s) => s,
        Err(_) => return Err(EFAULT),
    };

    // Read fstype string
    let fstype_str = match crate::syscall::sys::read_user_string(ttbr0, fstype_ptr, 64) {
        Ok(s) => s,
        Err(_) => return Err(EFAULT),
    };

    // Convert fstype
    let fstype = match crate::fs::mount::FsType::try_from(fstype_str.as_str()) {
        Ok(t) => t,
        Err(_) => return Err(EINVAL),
    };

    // Convert flags (simplified)
    let mount_flags = if (flags & 1) != 0 {
        crate::fs::mount::MountFlags::readonly()
    } else {
        crate::fs::mount::MountFlags::new()
    };

    match crate::fs::mount::mount(
        crate::fs::path::Path::new(&target),
        fstype,
        mount_flags,
        &source,
    ) {
        Ok(_) => Ok(0),
        Err(e) => match e {
            crate::fs::mount::MountError::AlreadyMounted => Err(EBUSY),
            crate::fs::mount::MountError::NotMounted => Err(EINVAL),
            crate::fs::mount::MountError::InvalidMountpoint => Err(ENOENT),
            crate::fs::mount::MountError::UnsupportedFsType => Err(EINVAL),
            crate::fs::mount::MountError::PermissionDenied => Err(EACCES),
        },
    }
}

// TEAM_206: Unmount a filesystem
/// TEAM_421: Returns SyscallResult
pub fn sys_umount(target_ptr: usize, _flags: usize) -> SyscallResult {
    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;

    // Read target string
    let target = match crate::syscall::sys::read_user_string(ttbr0, target_ptr, 256) {
        Ok(s) => s,
        Err(_) => return Err(EFAULT),
    };

    match crate::fs::mount::umount(crate::fs::path::Path::new(&target)) {
        Ok(_) => Ok(0),
        Err(_) => Err(EINVAL),
    }
}
