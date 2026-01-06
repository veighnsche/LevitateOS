//! TEAM_171: File system system calls.

use crate::syscall::{Stat, errno, errno_file, write_to_user_buf};
use los_hal::print;
use los_utils::cpio::CpioEntryType;

/// TEAM_081: sys_read - Read from a file descriptor.
/// TEAM_178: Refactored to dispatch by fd type, added InitramfsFile support.
pub fn sys_read(fd: usize, buf: usize, len: usize) -> i64 {
    if len == 0 {
        return 0;
    }

    let task = crate::task::current_task();
    
    // TEAM_178: Look up fd type and dispatch accordingly
    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd) {
        Some(e) => e.clone(),
        None => return errno::EBADF,
    };
    drop(fd_table);

    let ttbr0 = task.ttbr0;
    
    match entry.fd_type {
        crate::task::fd_table::FdType::Stdin => {
            read_stdin(buf, len, ttbr0)
        }
        crate::task::fd_table::FdType::InitramfsFile { file_index, offset } => {
            read_initramfs_file(fd, file_index, offset, buf, len, ttbr0)
        }
        // Q3/Q4: stdout, stderr, and directories return EBADF for read
        _ => errno::EBADF,
    }
}

/// TEAM_178: Read from stdin (keyboard/console input).
fn read_stdin(buf: usize, len: usize, ttbr0: usize) -> i64 {
    let max_read = len.min(4096);
    if crate::task::user_mm::validate_user_buffer(ttbr0, buf, max_read, true).is_err() {
        return errno::EFAULT;
    }

    let mut bytes_read = 0usize;

    loop {
        poll_input_devices(ttbr0, buf, &mut bytes_read, max_read);
        if bytes_read > 0 {
            break;
        }

        unsafe {
            los_hal::interrupts::enable();
        }
        let _ = los_hal::interrupts::disable();

        crate::task::yield_now();
    }

    bytes_read as i64
}

/// TEAM_178: Read from an initramfs file.
fn read_initramfs_file(
    fd: usize,
    file_index: usize,
    offset: usize,
    buf: usize,
    len: usize,
    ttbr0: usize,
) -> i64 {
    // Validate user buffer
    if crate::task::user_mm::validate_user_buffer(ttbr0, buf, len, true).is_err() {
        return errno::EFAULT;
    }

    // Get file data from initramfs
    let initramfs_guard = crate::fs::INITRAMFS.lock();
    let initramfs = match initramfs_guard.as_ref() {
        Some(i) => i,
        None => return errno::EBADF,
    };

    // Find the file entry by index
    let file_data = match initramfs.iter().nth(file_index) {
        Some(entry) => entry.data,
        None => return errno::EBADF,
    };

    let file_size = file_data.len();
    
    // Q1: If at or past EOF, return 0
    if offset >= file_size {
        return 0;
    }

    // Q2: Calculate bytes to read (partial read returns what's available)
    let available = file_size - offset;
    let to_read = len.min(available);

    // Copy data to userspace
    for i in 0..to_read {
        if !write_to_user_buf(ttbr0, buf, i, file_data[offset + i]) {
            return errno::EFAULT;
        }
    }

    drop(initramfs_guard);

    // Update offset in fd table
    let task = crate::task::current_task();
    let mut fd_table = task.fd_table.lock();
    if let Some(fd_entry) = fd_table.get_mut(fd) {
        if let crate::task::fd_table::FdType::InitramfsFile { offset: ref mut off, .. } = fd_entry.fd_type {
            *off = offset + to_read;
        }
    }

    to_read as i64
}

fn poll_input_devices(ttbr0: usize, user_buf: usize, bytes_read: &mut usize, max_read: usize) {
    crate::input::poll();

    while *bytes_read < max_read {
        if let Some(ch) = crate::input::read_char() {
            if !write_to_user_buf(ttbr0, user_buf, *bytes_read, ch as u8) {
                return;
            }
            *bytes_read += 1;
            if ch == '\n' {
                return;
            }
        } else {
            break;
        }
    }

    if *bytes_read < max_read {
        while let Some(byte) = los_hal::console::read_byte() {
            let byte = if byte == b'\r' { b'\n' } else { byte };
            if !write_to_user_buf(ttbr0, user_buf, *bytes_read, byte) {
                return;
            }
            *bytes_read += 1;
            if byte == b'\n' {
                return;
            }
            if *bytes_read >= max_read {
                return;
            }
        }
    }
}

/// TEAM_073: sys_write - Write to a file descriptor.
pub fn sys_write(fd: usize, buf: usize, len: usize) -> i64 {
    if fd != 1 && fd != 2 {
        return errno::EBADF;
    }

    let len = len.min(4096);
    let task = crate::task::current_task();
    if crate::task::user_mm::validate_user_buffer(task.ttbr0, buf, len, false).is_err() {
        return errno::EFAULT;
    }

    let slice = unsafe { core::slice::from_raw_parts(buf as *const u8, len) };

    if let Ok(s) = core::str::from_utf8(slice) {
        print!("{}", s);
    } else {
        for byte in slice {
            print!("{:02x}", byte);
        }
    }

    len as i64
}

/// TEAM_168: sys_openat - Open a file from initramfs.
/// TEAM_176: Updated to support opening directories for getdents.
pub fn sys_openat(path: usize, path_len: usize, _flags: u32) -> i64 {
    if path_len == 0 || path_len > 256 {
        return errno::EINVAL;
    }

    let task = crate::task::current_task();
    if crate::task::user_mm::validate_user_buffer(task.ttbr0, path, path_len, false).is_err() {
        return errno::EFAULT;
    }

    let mut path_buf = [0u8; 256];
    for i in 0..path_len {
        if let Some(ptr) = crate::task::user_mm::user_va_to_kernel_ptr(task.ttbr0, path + i) {
            path_buf[i] = unsafe { *ptr };
        } else {
            return errno::EFAULT;
        }
    }

    let path_str = match core::str::from_utf8(&path_buf[..path_len]) {
        Ok(s) => s,
        Err(_) => return errno::EINVAL,
    };

    let lookup_path = path_str.trim_start_matches('/');

    // TEAM_176: Check for root directory open
    if lookup_path.is_empty() || lookup_path == "." {
        let mut fd_table = task.fd_table.lock();
        return match fd_table.alloc(crate::task::fd_table::FdType::InitramfsDir {
            dir_index: 0, // 0 = root
            offset: 0,
        }) {
            Some(fd) => fd as i64,
            None => errno_file::EMFILE,
        };
    }

    let initramfs_guard = crate::fs::INITRAMFS.lock();
    let initramfs = match initramfs_guard.as_ref() {
        Some(i) => i,
        None => return errno_file::ENOENT,
    };

    let mut found_entry = None;
    let mut file_index = 0;
    for (idx, entry) in initramfs.iter().enumerate() {
        let entry_name = entry.name.trim_start_matches('/');
        if entry_name == lookup_path {
            found_entry = Some(entry.entry_type);
            file_index = idx;
            break;
        }
    }

    let entry_type = match found_entry {
        Some(t) => t,
        None => return errno_file::ENOENT,
    };

    drop(initramfs_guard);

    let mut fd_table = task.fd_table.lock();
    
    // TEAM_176: Allocate appropriate fd type based on entry type
    let fd_type = if entry_type == CpioEntryType::Directory {
        crate::task::fd_table::FdType::InitramfsDir {
            dir_index: file_index,
            offset: 0,
        }
    } else {
        crate::task::fd_table::FdType::InitramfsFile {
            file_index,
            offset: 0,
        }
    };

    match fd_table.alloc(fd_type) {
        Some(fd) => fd as i64,
        None => errno_file::EMFILE,
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

/// TEAM_168: sys_fstat - Get file status.
pub fn sys_fstat(fd: usize, stat_buf: usize) -> i64 {
    let task = crate::task::current_task();
    let stat_size = core::mem::size_of::<Stat>();
    if crate::task::user_mm::validate_user_buffer(task.ttbr0, stat_buf, stat_size, true).is_err() {
        return errno::EFAULT;
    }

    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd) {
        Some(e) => e,
        None => return errno::EBADF,
    };

    let stat = match entry.fd_type {
        crate::task::fd_table::FdType::Stdin
        | crate::task::fd_table::FdType::Stdout
        | crate::task::fd_table::FdType::Stderr => Stat {
            st_size: 0,
            st_mode: 2,
            _pad: 0,
        },
        crate::task::fd_table::FdType::InitramfsFile { file_index, .. } => {
            let initramfs_guard = crate::fs::INITRAMFS.lock();
            let initramfs = match initramfs_guard.as_ref() {
                Some(i) => i,
                None => return errno::EBADF,
            };

            let file_size = initramfs
                .iter()
                .nth(file_index)
                .map(|e| e.data.len())
                .unwrap_or(0);

            Stat {
                st_size: file_size as u64,
                st_mode: 1, // Regular file
                _pad: 0,
            }
        }
        // TEAM_176: Directory fd returns directory mode
        crate::task::fd_table::FdType::InitramfsDir { .. } => Stat {
            st_size: 0,
            st_mode: 2, // Directory
            _pad: 0,
        },
    };

    let stat_bytes =
        unsafe { core::slice::from_raw_parts(&stat as *const Stat as *const u8, stat_size) };

    for (i, &byte) in stat_bytes.iter().enumerate() {
        if let Some(ptr) = crate::task::user_mm::user_va_to_kernel_ptr(task.ttbr0, stat_buf + i) {
            unsafe { *ptr = byte };
        } else {
            return errno::EFAULT;
        }
    }

    0
}

/// TEAM_176: Dirent64 structure for getdents syscall.
/// Matches Linux ABI layout.
#[repr(C, packed)]
struct Dirent64 {
    d_ino: u64,      // Inode number
    d_off: i64,      // Offset to next entry
    d_reclen: u16,   // Length of this record
    d_type: u8,      // File type
    // d_name follows (null-terminated)
}

/// TEAM_176: File type constants for d_type
mod d_type {
    pub const DT_UNKNOWN: u8 = 0;
    pub const DT_DIR: u8 = 4;
    pub const DT_REG: u8 = 8;
    pub const DT_LNK: u8 = 10;
}

/// TEAM_176: sys_getdents - Read directory entries.
///
/// # Arguments
/// * `fd` - Directory file descriptor
/// * `buf` - User buffer to write Dirent64 records
/// * `buf_len` - Size of buffer in bytes
///
/// # Returns
/// * `> 0` - Number of bytes written
/// * `0` - End of directory
/// * `< 0` - Error code
pub fn sys_getdents(fd: usize, buf: usize, buf_len: usize) -> i64 {
    if buf_len == 0 {
        return 0;
    }

    let task = crate::task::current_task();
    if crate::task::user_mm::validate_user_buffer(task.ttbr0, buf, buf_len, true).is_err() {
        return errno::EFAULT;
    }

    // Get fd entry and check if it's a directory
    let fd_table = task.fd_table.lock();
    let entry = match fd_table.get(fd) {
        Some(e) => e.clone(),
        None => return errno::EBADF,
    };

    let (dir_offset, is_root) = match entry.fd_type {
        crate::task::fd_table::FdType::InitramfsDir { dir_index, offset } => {
            (offset, dir_index == 0)
        }
        _ => return errno_file::ENOTDIR,
    };
    drop(fd_table);

    // Get initramfs
    let initramfs_guard = crate::fs::INITRAMFS.lock();
    let initramfs = match initramfs_guard.as_ref() {
        Some(i) => i,
        None => return errno::EBADF,
    };

    // Collect directory entries (skip . and .. per Q2 decision)
    let entries: alloc::vec::Vec<_> = if is_root {
        // Root directory: entries without '/' in name
        initramfs
            .iter()
            .filter(|e| {
                let name = e.name;
                name != "." && name != ".." && !name.contains('/')
            })
            .collect()
    } else {
        // Non-root: would need path tracking, for now just return empty
        // TODO(TEAM_176): Support non-root directories
        alloc::vec::Vec::new()
    };

    drop(initramfs_guard);

    // Skip already-read entries
    let remaining_entries = entries.iter().skip(dir_offset);

    let mut bytes_written = 0usize;
    let mut entries_written = 0usize;

    for entry in remaining_entries {
        let name_bytes = entry.name.as_bytes();
        let name_len = name_bytes.len();
        
        // TEAM_183: Record size with checked arithmetic to prevent overflow
        // Header (19 bytes) + name + null + padding to 8-byte alignment
        let reclen = match (19usize)
            .checked_add(name_len)
            .and_then(|n| n.checked_add(1))
            .and_then(|n| n.checked_add(7))
            .map(|n| (n / 8) * 8)
        {
            Some(r) if r <= u16::MAX as usize => r,
            _ => continue, // Skip entry if reclen overflows or exceeds u16::MAX
        };
        
        if bytes_written + reclen > buf_len {
            break; // Buffer full
        }

        // Determine d_type from entry type
        let dtype = match entry.entry_type {
            CpioEntryType::File => d_type::DT_REG,
            CpioEntryType::Directory => d_type::DT_DIR,
            CpioEntryType::Symlink => d_type::DT_LNK,
            CpioEntryType::Other => d_type::DT_UNKNOWN,
        };

        // Write dirent64 header
        let dirent = Dirent64 {
            d_ino: entry.ino,
            d_off: (dir_offset + entries_written + 1) as i64,
            d_reclen: reclen as u16,
            d_type: dtype,
        };

        let dirent_bytes = unsafe {
            core::slice::from_raw_parts(
                &dirent as *const Dirent64 as *const u8,
                core::mem::size_of::<Dirent64>(),
            )
        };

        // Write header
        for (i, &byte) in dirent_bytes.iter().enumerate() {
            if !write_to_user_buf(task.ttbr0, buf, bytes_written + i, byte) {
                return errno::EFAULT;
            }
        }

        // Write name
        let name_offset = bytes_written + core::mem::size_of::<Dirent64>();
        for (i, &byte) in name_bytes.iter().enumerate() {
            if !write_to_user_buf(task.ttbr0, buf, name_offset + i, byte) {
                return errno::EFAULT;
            }
        }

        // Write null terminator
        if !write_to_user_buf(task.ttbr0, buf, name_offset + name_len, 0) {
            return errno::EFAULT;
        }

        // Zero-fill padding
        for i in (name_offset + name_len + 1)..(bytes_written + reclen) {
            if !write_to_user_buf(task.ttbr0, buf, i, 0) {
                return errno::EFAULT;
            }
        }

        bytes_written += reclen;
        entries_written += 1;
    }

    // Update offset in fd table
    let mut fd_table = task.fd_table.lock();
    if let Some(fd_entry) = fd_table.get_mut(fd) {
        if let crate::task::fd_table::FdType::InitramfsDir { offset, .. } = &mut fd_entry.fd_type {
            *offset = dir_offset + entries_written;
        }
    }

    bytes_written as i64
}
