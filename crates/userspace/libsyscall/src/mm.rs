//! Memory management
//! TEAM_275: Refactored to use arch::syscallN

use crate::arch;
use crate::sysno::{__NR_brk, __NR_mmap, __NR_mprotect, __NR_munmap};

// ... constants unchanged ...

// TEAM_228: mmap protection flags
pub const PROT_NONE: u32 = 0;
pub const PROT_READ: u32 = 1;
pub const PROT_WRITE: u32 = 2;
pub const PROT_EXEC: u32 = 4;

// TEAM_228: mmap flags
pub const MAP_SHARED: u32 = 0x01;
pub const MAP_PRIVATE: u32 = 0x02;
pub const MAP_FIXED: u32 = 0x10;
pub const MAP_ANONYMOUS: u32 = 0x20;

/// Adjust program break (heap allocation).
#[inline]
pub fn sbrk(increment: isize) -> i64 {
    // Note: This maps to __NR_brk which assumes standard brk behavior.
    // LevitateOS kernel handler implementation determines if it accepts increment or addr.
    arch::syscall1(__NR_brk as u64, increment as u64)
}

/// TEAM_228: Map memory into process address space.
///
/// # Arguments
/// * `addr` - Hint address (can be 0 for system to choose)
/// * `len` - Length of mapping
/// * `prot` - Protection flags (PROT_READ | PROT_WRITE | PROT_EXEC)
/// * `flags` - Mapping flags (must include MAP_ANONYMOUS | MAP_PRIVATE)
/// * `fd` - File descriptor (-1 for anonymous)
/// * `offset` - File offset (0 for anonymous)
///
/// # Returns
/// * Virtual address of mapping, or negative error code.
#[inline]
pub fn mmap(addr: usize, len: usize, prot: u32, flags: u32, fd: i32, offset: usize) -> isize {
    arch::syscall6(
        __NR_mmap as u64,
        addr as u64,
        len as u64,
        prot as u64,
        flags as u64,
        fd as u64,
        offset as u64,
    ) as isize
}

/// TEAM_228: Unmap memory from process address space.
///
/// # Arguments
/// * `addr` - Start address of mapping (must be page-aligned)
/// * `len` - Length to unmap
///
/// # Returns
/// * 0 on success, negative error code on failure.
#[inline]
pub fn munmap(addr: usize, len: usize) -> isize {
    arch::syscall2(__NR_munmap as u64, addr as u64, len as u64) as isize
}

/// TEAM_228: Change protection on memory region.
///
/// # Arguments
/// * `addr` - Start address (must be page-aligned)
/// * `len` - Length of region
/// * `prot` - New protection flags
///
/// # Returns
/// * 0 on success, negative error code on failure.
#[inline]
pub fn mprotect(addr: usize, len: usize, prot: u32) -> isize {
    arch::syscall3(__NR_mprotect as u64, addr as u64, len as u64, prot as u64) as isize
}
