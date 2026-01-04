//! TEAM_073: HelloWorld userspace binary for LevitateOS.
//!
//! This is the first userspace program. It demonstrates:
//! - Running in EL0 (user mode)
//! - Making syscalls via SVC
//! - Printing to console
//! - Clean exit

#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// Syscall numbers (matching kernel's syscall.rs)
mod syscall {
    pub const SYS_WRITE: u64 = 1;
    pub const SYS_EXIT: u64 = 2;

    /// Write to a file descriptor.
    ///
    /// # Safety
    /// Uses inline assembly for syscall.
    #[inline(always)]
    pub fn write(fd: usize, buf: &[u8]) -> isize {
        let ret: i64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") SYS_WRITE,
                in("x0") fd,
                in("x1") buf.as_ptr(),
                in("x2") buf.len(),
                lateout("x0") ret,
                options(nostack)
            );
        }
        ret as isize
    }

    /// Exit the process.
    ///
    /// # Safety
    /// Uses inline assembly for syscall.
    #[inline(always)]
    pub fn exit(code: i32) -> ! {
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") SYS_EXIT,
                in("x0") code,
                options(noreturn, nostack)
            );
        }
    }
}

/// Print a string to stdout.
fn print(s: &str) {
    syscall::write(1, s.as_bytes());
}

/// Entry point for the userspace program.
#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    print("Hello from userspace!\n");
    print("LevitateOS Phase 8: Userspace support working!\n");
    syscall::exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("PANIC in userspace!\n");
    syscall::exit(1);
}
