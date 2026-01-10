//! TEAM_413: Syscall Helper Abstractions
//!
//! This module provides common abstractions to reduce boilerplate across
//! syscall implementations while maintaining Linux ABI compatibility.
//!
//! ## Abstractions
//!
//! - `UserPtr<T>` / `UserSlice<T>`: Safe user-space memory access
//! - `get_fd()` / `get_vfs_file()`: FD lookup with proper error handling
//! - `write_struct_to_user()`: Generic struct copying to userspace
//! - `resolve_at_path()`: Proper dirfd resolution for *at() syscalls

extern crate alloc;

use alloc::format;
use alloc::string::String;
use core::marker::PhantomData;

use crate::fs::vfs::file::FileRef;
use crate::memory::user as mm_user;
use crate::syscall::{errno, fcntl, read_user_cstring};
use crate::task::fd_table::{FdEntry, FdType};
use crate::task::current_task;

// ============================================================================
// TEAM_413: UserPtr - Safe wrapper for user-space pointer access
// ============================================================================

/// TEAM_413: Validated pointer to user-space data of type T.
///
/// This provides safe read/write access to a single value in user memory.
/// All operations validate the memory mapping before access.
pub struct UserPtr<T: Copy> {
    ttbr0: usize,
    addr: usize,
    _marker: PhantomData<T>,
}

impl<T: Copy> UserPtr<T> {
    /// Create a new UserPtr from a user virtual address.
    ///
    /// Does NOT validate the address - validation happens on read/write.
    pub fn new(ttbr0: usize, addr: usize) -> Self {
        Self {
            ttbr0,
            addr,
            _marker: PhantomData,
        }
    }

    /// Read a value from user space.
    ///
    /// Returns EFAULT if the memory is not mapped or not readable.
    pub fn read(&self) -> Result<T, i64> {
        let size = core::mem::size_of::<T>();
        if mm_user::validate_user_buffer(self.ttbr0, self.addr, size, false).is_err() {
            return Err(errno::EFAULT);
        }

        let src = mm_user::user_va_to_kernel_ptr(self.ttbr0, self.addr)
            .ok_or(errno::EFAULT)?;

        // SAFETY: validate_user_buffer confirmed the memory is mapped and accessible.
        // We read size_of::<T>() bytes which is the exact size needed.
        let value = unsafe {
            core::ptr::read_unaligned(src as *const T)
        };

        Ok(value)
    }

    /// Write a value to user space.
    ///
    /// Returns EFAULT if the memory is not mapped or not writable.
    pub fn write(&self, val: T) -> Result<(), i64> {
        let size = core::mem::size_of::<T>();
        if mm_user::validate_user_buffer(self.ttbr0, self.addr, size, true).is_err() {
            return Err(errno::EFAULT);
        }

        let dest = mm_user::user_va_to_kernel_ptr(self.ttbr0, self.addr)
            .ok_or(errno::EFAULT)?;

        // SAFETY: validate_user_buffer confirmed the memory is mapped and writable.
        // We write size_of::<T>() bytes which is the exact size needed.
        unsafe {
            core::ptr::write_unaligned(dest as *mut T, val);
        }

        Ok(())
    }

    /// Check if the address is null (0).
    pub fn is_null(&self) -> bool {
        self.addr == 0
    }
}

// ============================================================================
// TEAM_413: UserSlice - Safe wrapper for user-space buffer access
// ============================================================================

/// TEAM_413: Validated slice in user-space.
///
/// This provides safe read/write access to a buffer in user memory.
/// All operations validate the memory mapping before access.
pub struct UserSlice<T: Copy> {
    ttbr0: usize,
    addr: usize,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T: Copy> UserSlice<T> {
    /// Create a new UserSlice from a user virtual address and length.
    ///
    /// The length is in number of elements (not bytes).
    /// Does NOT validate the address - validation happens on read/write.
    pub fn new(ttbr0: usize, addr: usize, len: usize) -> Self {
        Self {
            ttbr0,
            addr,
            len,
            _marker: PhantomData,
        }
    }

    /// Get the length in elements.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Copy data from user space into a kernel buffer.
    ///
    /// Returns the number of elements copied, or EFAULT on error.
    pub fn read_to(&self, buf: &mut [T]) -> Result<usize, i64> {
        let count = self.len.min(buf.len());
        if count == 0 {
            return Ok(0);
        }

        let byte_len = count * core::mem::size_of::<T>();
        if mm_user::validate_user_buffer(self.ttbr0, self.addr, byte_len, false).is_err() {
            return Err(errno::EFAULT);
        }

        let src = mm_user::user_va_to_kernel_ptr(self.ttbr0, self.addr)
            .ok_or(errno::EFAULT)?;

        // SAFETY: validate_user_buffer confirmed the memory is mapped.
        unsafe {
            core::ptr::copy_nonoverlapping(
                src as *const T,
                buf.as_mut_ptr(),
                count,
            );
        }

        Ok(count)
    }

    /// Copy data from a kernel buffer to user space.
    ///
    /// Returns the number of elements copied, or EFAULT on error.
    pub fn write_from(&self, buf: &[T]) -> Result<usize, i64> {
        let count = self.len.min(buf.len());
        if count == 0 {
            return Ok(0);
        }

        let byte_len = count * core::mem::size_of::<T>();
        if mm_user::validate_user_buffer(self.ttbr0, self.addr, byte_len, true).is_err() {
            return Err(errno::EFAULT);
        }

        let dest = mm_user::user_va_to_kernel_ptr(self.ttbr0, self.addr)
            .ok_or(errno::EFAULT)?;

        // SAFETY: validate_user_buffer confirmed the memory is mapped and writable.
        unsafe {
            core::ptr::copy_nonoverlapping(
                buf.as_ptr(),
                dest as *mut T,
                count,
            );
        }

        Ok(count)
    }
}

/// TEAM_413: Convenience constructor for byte slices.
impl UserSlice<u8> {
    /// Create a UserSlice for raw bytes.
    pub fn bytes(ttbr0: usize, addr: usize, len: usize) -> Self {
        Self::new(ttbr0, addr, len)
    }
}

// ============================================================================
// TEAM_413: FD Lookup Helpers
// ============================================================================

/// TEAM_413: Get a cloned FdEntry for the given fd.
///
/// This eliminates the common pattern of:
/// ```ignore
/// let task = current_task();
/// let fd_table = task.fd_table.lock();
/// let entry = match fd_table.get(fd) {
///     Some(e) => e.clone(),
///     None => return errno::EBADF,
/// };
/// drop(fd_table);
/// ```
///
/// Returns EBADF if the fd is not valid.
pub fn get_fd(fd: usize) -> Result<FdEntry, i64> {
    let task = current_task();
    let fd_table = task.fd_table.lock();
    fd_table.get(fd).cloned().ok_or(errno::EBADF)
}

/// TEAM_413: Get a VfsFile for the given fd.
///
/// This combines fd lookup with type checking, returning EBADF
/// if the fd is not a VfsFile (e.g., it's a pipe or special fd).
pub fn get_vfs_file(fd: usize) -> Result<FileRef, i64> {
    let entry = get_fd(fd)?;
    match &entry.fd_type {
        FdType::VfsFile(f) => Ok(f.clone()),
        _ => Err(errno::EBADF),
    }
}

/// TEAM_413: Check if an fd is valid without cloning the entry.
pub fn is_valid_fd(fd: usize) -> bool {
    let task = current_task();
    let fd_table = task.fd_table.lock();
    fd_table.get(fd).is_some()
}

// ============================================================================
// TEAM_413: Struct Writing Helper
// ============================================================================

/// TEAM_413: Write a struct to user space.
///
/// This is a generic helper for copying kernel structures to userspace.
/// The struct must be `#[repr(C)]` to ensure correct memory layout.
///
/// # Arguments
/// * `ttbr0` - User page table physical address
/// * `user_buf` - User virtual address to write to
/// * `value` - Reference to the struct to copy
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(EFAULT)` if the memory is not mapped or writable
///
/// # Example
/// ```ignore
/// let stat = Stat { ... };
/// write_struct_to_user(task.ttbr0, statbuf, &stat)?;
/// ```
pub fn write_struct_to_user<T: Copy>(ttbr0: usize, user_buf: usize, value: &T) -> Result<(), i64> {
    let size = core::mem::size_of::<T>();

    if mm_user::validate_user_buffer(ttbr0, user_buf, size, true).is_err() {
        return Err(errno::EFAULT);
    }

    let dest = mm_user::user_va_to_kernel_ptr(ttbr0, user_buf)
        .ok_or(errno::EFAULT)?;

    // SAFETY: validate_user_buffer confirmed the memory is mapped and writable.
    // We copy exactly size_of::<T>() bytes.
    unsafe {
        core::ptr::copy_nonoverlapping(
            value as *const T as *const u8,
            dest,
            size,
        );
    }

    Ok(())
}

/// TEAM_413: Read a struct from user space.
///
/// This is a generic helper for reading kernel structures from userspace.
/// The struct must be `#[repr(C)]` to ensure correct memory layout.
pub fn read_struct_from_user<T: Copy + Default>(ttbr0: usize, user_buf: usize) -> Result<T, i64> {
    let size = core::mem::size_of::<T>();

    if mm_user::validate_user_buffer(ttbr0, user_buf, size, false).is_err() {
        return Err(errno::EFAULT);
    }

    let src = mm_user::user_va_to_kernel_ptr(ttbr0, user_buf)
        .ok_or(errno::EFAULT)?;

    // SAFETY: validate_user_buffer confirmed the memory is mapped.
    let value = unsafe {
        core::ptr::read_unaligned(src as *const T)
    };

    Ok(value)
}

// ============================================================================
// TEAM_413: Path Resolution Helper
// ============================================================================

/// TEAM_413: Resolve a pathname relative to a directory fd.
///
/// This implements proper dirfd resolution for `*at()` syscalls:
/// - If path is absolute, dirfd is ignored
/// - If `dirfd == AT_FDCWD`, resolves relative to task's cwd
/// - Otherwise, resolves relative to the directory referred to by dirfd
///
/// # Arguments
/// * `dirfd` - Directory file descriptor or AT_FDCWD
/// * `pathname` - User pointer to null-terminated path string
///
/// # Returns
/// * `Ok(String)` - The resolved absolute path
/// * `Err(errno)` - EFAULT, EBADF, ENOTDIR, or ENAMETOOLONG
///
/// # Note
/// This currently only supports AT_FDCWD. Proper dirfd support requires
/// storing the path in FdEntry, which is a TODO for future work.
pub fn resolve_at_path(dirfd: i32, pathname: usize) -> Result<String, i64> {
    let task = current_task();

    // Read pathname from user space
    let mut path_buf = [0u8; 4096];
    let path_str = read_user_cstring(task.ttbr0, pathname, &mut path_buf)?;

    // Absolute paths ignore dirfd
    if path_str.starts_with('/') {
        return Ok(String::from(path_str));
    }

    // AT_FDCWD means use current working directory
    if dirfd == fcntl::AT_FDCWD {
        let cwd = task.cwd.lock();
        let base = cwd.trim_end_matches('/');
        if base.is_empty() {
            return Ok(format!("/{}", path_str));
        }
        return Ok(format!("{}/{}", base, path_str));
    }

    // TODO(TEAM_413): Proper dirfd support requires storing path in FdEntry.
    // For now, we return EBADF for non-AT_FDCWD dirfd with relative paths.
    // This matches the current behavior but is documented as a known limitation.
    //
    // To implement properly:
    // 1. Add `path: Option<String>` field to FdEntry or VfsFile
    // 2. Set the path when opening files
    // 3. Use it here to resolve relative paths

    let entry = get_fd(dirfd as usize)?;
    match &entry.fd_type {
        FdType::VfsFile(file) => {
            // Check if it's a directory
            if !file.inode.is_dir() {
                return Err(errno::ENOTDIR);
            }
            // For now, we can't get the path from the file
            // Log a warning and fall back to EBADF
            log::warn!(
                "[SYSCALL] resolve_at_path: dirfd {} not yet supported for relative paths",
                dirfd
            );
            Err(errno::EBADF)
        }
        _ => Err(errno::EBADF),
    }
}

/// TEAM_413: Read a path from userspace and handle AT_FDCWD.
///
/// Simpler version that just reads the path without full resolution.
/// Used when you only need the path string and will handle dirfd separately.
pub fn read_user_path(pathname: usize) -> Result<String, i64> {
    let task = current_task();
    let mut path_buf = [0u8; 4096];
    let path_str = read_user_cstring(task.ttbr0, pathname, &mut path_buf)?;
    Ok(String::from(path_str))
}

// ============================================================================
// TEAM_413: Result Extension for Syscalls
// ============================================================================

/// TEAM_413: Extension trait for converting VfsResult to syscall return.
///
/// This allows writing `vfs_operation().to_syscall_result(|| 0)` instead of
/// explicit match blocks.
pub trait SyscallResultExt<T> {
    /// Convert a VfsResult to a syscall return value.
    ///
    /// On success, calls the provided closure to compute the return value.
    /// On error, converts the VfsError to negative errno.
    fn to_syscall_result<F: FnOnce(T) -> i64>(self, on_success: F) -> i64;
}

impl<T> SyscallResultExt<T> for Result<T, crate::fs::vfs::error::VfsError> {
    fn to_syscall_result<F: FnOnce(T) -> i64>(self, on_success: F) -> i64 {
        match self {
            Ok(v) => on_success(v),
            Err(e) => e.into(),
        }
    }
}

// ============================================================================
// TEAM_415: Ioctl Helpers for FD operations
// ============================================================================

use crate::fs::tty::Termios;

/// TEAM_415: Write a termios struct to user space.
///
/// This helper encapsulates the common pattern in TCGETS ioctl handlers.
pub fn ioctl_get_termios(ttbr0: usize, arg: usize, termios: &Termios) -> i64 {
    match write_struct_to_user(ttbr0, arg, termios) {
        Ok(()) => 0,
        Err(e) => e,
    }
}

/// TEAM_415: Read a termios struct from user space.
///
/// This helper encapsulates the common pattern in TCSETS ioctl handlers.
/// Returns the Termios on success, or negative errno on failure.
pub fn ioctl_read_termios(ttbr0: usize, arg: usize) -> Result<Termios, i64> {
    read_struct_from_user(ttbr0, arg)
}

/// TEAM_415: Write an i32 value to user space for ioctl.
///
/// Used for TIOCGPGRP and similar ioctls that return an integer.
pub fn ioctl_write_i32(ttbr0: usize, arg: usize, value: i32) -> i64 {
    let ptr = UserPtr::<i32>::new(ttbr0, arg);
    match ptr.write(value) {
        Ok(()) => 0,
        Err(e) => e,
    }
}

/// TEAM_415: Read an i32 value from user space for ioctl.
///
/// Used for TIOCSPGRP and similar ioctls that take an integer.
pub fn ioctl_read_i32(ttbr0: usize, arg: usize) -> Result<i32, i64> {
    let ptr = UserPtr::<i32>::new(ttbr0, arg);
    ptr.read()
}

/// TEAM_415: Write a u32 value to user space for ioctl.
///
/// Used for TIOCGPTN and similar ioctls that return an unsigned integer.
pub fn ioctl_write_u32(ttbr0: usize, arg: usize, value: u32) -> i64 {
    let ptr = UserPtr::<u32>::new(ttbr0, arg);
    match ptr.write(value) {
        Ok(()) => 0,
        Err(e) => e,
    }
}

/// TEAM_415: Read a u32 value from user space for ioctl.
///
/// Used for TIOCSPTLCK and similar ioctls that take an unsigned integer.
pub fn ioctl_read_u32(ttbr0: usize, arg: usize) -> Result<u32, i64> {
    let ptr = UserPtr::<u32>::new(ttbr0, arg);
    ptr.read()
}

#[cfg(test)]
mod tests {
    // Unit tests would go here if we had a test harness
}
