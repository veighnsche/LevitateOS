use crate::memory::user as mm_user;

use crate::syscall::{errno, errno_file};
use core::convert::TryFrom;

// TEAM_206: Mount a filesystem
pub fn sys_mount(
    source_ptr: usize,
    target_ptr: usize,
    fstype_ptr: usize,
    flags: usize,
    _data_ptr: usize,
) -> i64 {
    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;

    // Read source string
    let source = match crate::syscall::sys::read_user_string(ttbr0, source_ptr, 256) {
        Ok(s) => s,
        Err(_) => return errno::EFAULT,
    };

    // Read target string
    let target = match crate::syscall::sys::read_user_string(ttbr0, target_ptr, 256) {
        Ok(s) => s,
        Err(_) => return errno::EFAULT,
    };

    // Read fstype string
    let fstype_str = match crate::syscall::sys::read_user_string(ttbr0, fstype_ptr, 64) {
        Ok(s) => s,
        Err(_) => return errno::EFAULT,
    };

    // Convert fstype
    let fstype = match crate::fs::mount::FsType::try_from(fstype_str.as_str()) {
        Ok(t) => t,
        Err(_) => return errno::EINVAL,
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
        Ok(_) => 0,
        Err(e) => match e {
            crate::fs::mount::MountError::AlreadyMounted => errno::EEXIST, // or EBUSY
            crate::fs::mount::MountError::NotMounted => errno::EINVAL,
            crate::fs::mount::MountError::InvalidMountpoint => errno::ENOENT,
            crate::fs::mount::MountError::UnsupportedFsType => errno::EINVAL, // or ENODEV
            crate::fs::mount::MountError::PermissionDenied => errno_file::EACCES,
        },
    }
}

// TEAM_206: Unmount a filesystem
pub fn sys_umount(target_ptr: usize, _flags: usize) -> i64 {
    let task = crate::task::current_task();
    let ttbr0 = task.ttbr0;

    // Read target string
    let target = match crate::syscall::sys::read_user_string(ttbr0, target_ptr, 256) {
        Ok(s) => s,
        Err(_) => return errno::EFAULT,
    };

    match crate::fs::mount::umount(crate::fs::path::Path::new(&target)) {
        Ok(_) => 0,
        Err(_) => errno::EINVAL, // Simplified mapping
    }
}
