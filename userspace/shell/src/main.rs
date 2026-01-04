//! TEAM_081: LevitateOS Shell (lsh)
//!
//! A minimal interactive shell for LevitateOS.
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

/// Trim whitespace from both ends of a string slice.
fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = s.len();
    
    while start < end && (s[start] == b' ' || s[start] == b'\t' || s[start] == b'\n' || s[start] == b'\r') {
        start += 1;
    }
    while end > start && (s[end - 1] == b' ' || s[end - 1] == b'\t' || s[end - 1] == b'\n' || s[end - 1] == b'\r') {
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
    if s.len() < prefix.len() {
        return false;
    }
    bytes_eq(&s[..prefix.len()], prefix)
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
        println("Available commands:");
        println("  echo <text>  - Print text to console");
        println("  help         - Show this help message");
        println("  clear        - Clear the screen");
        println("  exit         - Exit the shell");
        return;
    }

    // Builtin: clear (ANSI escape code)
    if bytes_eq(cmd, b"clear") {
        print("\x1b[2J\x1b[H"); // Clear screen and move cursor to home
        return;
    }

    // Builtin: echo
    if starts_with(cmd, b"echo ") {
        // Print everything after "echo "
        let text = &cmd[5..];
        if let Ok(s) = core::str::from_utf8(text) {
            println(s);
        }
        return;
    }
    if bytes_eq(cmd, b"echo") {
        println(""); // Empty echo
        return;
    }

    // Unknown command
    print("Unknown command: ");
    if let Ok(s) = core::str::from_utf8(cmd) {
        println(s);
    } else {
        println("<invalid utf8>");
    }
    println("Type 'help' for available commands.");
}

/// Entry point for the shell.
#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    // Print banner
    println("");
    println("LevitateOS Shell (lsh) v0.1");
    println("Type 'help' for available commands.");
    println("");

    // Input buffer
    let mut buf = [0u8; 256];

    loop {
        // Print prompt
        print("# ");

        // Read line
        let n = read_line(&mut buf);
        if n == 0 {
            continue;
        }

        // Execute command
        execute(&buf[..n]);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("PANIC in shell!\n");
    syscall::exit(1);
}
