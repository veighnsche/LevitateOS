//! TEAM_081: LevitateOS Shell (lsh)
//!
//! Interactive shell for LevitateOS Phase 8b.
//! Supports builtin commands: echo, help, clear, exit

#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// Syscall numbers (matching kernel's syscall.rs)
mod syscall {
    pub const SYS_READ: u64 = 0;
    pub const SYS_WRITE: u64 = 1;
    pub const SYS_EXIT: u64 = 2;

    /// Read from a file descriptor.
    #[inline(always)]
    pub fn read(fd: usize, buf: &mut [u8]) -> isize {
        let ret: i64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") SYS_READ,
                in("x0") fd,
                in("x1") buf.as_mut_ptr(),
                in("x2") buf.len(),
                lateout("x0") ret,
                options(nostack)
            );
        }
        ret as isize
    }

    /// Write to a file descriptor.
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

/// Print a string followed by newline.
fn println(s: &str) {
    print(s);
    print("\n");
}

/// Read a line from stdin into buffer. Returns number of bytes read.
fn read_line(buf: &mut [u8]) -> usize {
    let n = syscall::read(0, buf);
    if n < 0 {
        0
    } else {
        n as usize
    }
}

/// Trim whitespace from both ends of a byte slice.
fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = s.len();
    while start < end && matches!(s[start], b' ' | b'\t' | b'\n' | b'\r') {
        start += 1;
    }
    while end > start && matches!(s[end - 1], b' ' | b'\t' | b'\n' | b'\r') {
        end -= 1;
    }
    &s[start..end]
}

/// Check if two byte slices are equal.
fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }
    true
}

/// Check if slice starts with prefix.
fn starts_with(s: &[u8], prefix: &[u8]) -> bool {
    s.len() >= prefix.len() && bytes_eq(&s[..prefix.len()], prefix)
}

/// Execute a command.
fn execute(line: &[u8]) {
    let cmd = trim(line);
    if cmd.is_empty() {
        return;
    }

    // Builtin: exit
    if bytes_eq(cmd, b"exit") {
        println("Goodbye!");
        syscall::exit(0);
    }

    // Builtin: help
    if bytes_eq(cmd, b"help") {
        println("LevitateOS Shell (lsh) v0.1");
        println("Commands: echo <text>, help, clear, exit");
        return;
    }

    // Builtin: clear (ANSI escape)
    if bytes_eq(cmd, b"clear") {
        print("\x1b[2J\x1b[H");
        return;
    }

    // Builtin: echo
    if starts_with(cmd, b"echo ") {
        if let Ok(s) = core::str::from_utf8(&cmd[5..]) {
            println(s);
        }
        return;
    }
    if bytes_eq(cmd, b"echo") {
        println("");
        return;
    }

    // Unknown command
    print("Unknown: ");
    if let Ok(s) = core::str::from_utf8(cmd) {
        println(s);
    }
}

/// Entry point for the shell.
#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    println("");
    println("LevitateOS Shell (lsh) v0.1");
    println("Type 'help' for commands.");
    println("");

    let mut buf = [0u8; 256];

    loop {
        print("# ");
        let mut line_len = 0;
        loop {
            let mut c_buf = [0u8; 1];
            let n = syscall::read(0, &mut c_buf);
            if n > 0 {
                let bytes = &c_buf[..n as usize];
                // Echo back to user
                syscall::write(1, bytes);

                for &b in bytes {
                    if b == b'\n' || b == b'\r' {
                        if line_len > 0 {
                            execute(&buf[..line_len]);
                        }
                        line_len = 0;
                        print("# ");
                    } else if b == 0x08 || b == 0x7f {
                        // Backspace
                        if line_len > 0 {
                            line_len -= 1;
                        }
                    } else if line_len < buf.len() {
                        buf[line_len] = b;
                        line_len += 1;
                    }
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("PANIC in shell!\n");
    syscall::exit(1);
}
