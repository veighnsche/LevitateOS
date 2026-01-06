//! `TEAM_081`: `LevitateOS` Shell (`lsh`)
//!
//! Interactive shell for `LevitateOS` Phase 8b.
//! Supports builtin commands: `echo`, `help`, `clear`, `exit`
//!
//! `TEAM_118`: Refactored to use `libsyscall`.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libsyscall::{common_panic_handler, print, println};

// ============================================================================
// Panic Handler
// ============================================================================

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

// ============================================================================
// Shell Logic
// ============================================================================

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
        println!("Goodbye!");
        libsyscall::exit(0);
    }

    // Builtin: test (POSIX: exit 0 if no args, used for shell scripting)
    if bytes_eq(cmd, b"test") {
        // No output, exit 0 (implicit return in shell)
        return;
    }

    // Builtin: help
    if bytes_eq(cmd, b"help") {
        println!("LevitateOS Shell (lsh) v0.1");
        println!("Commands: echo <text>, help, clear, exit, test");
        return;
    }

    // Builtin: clear (ANSI escape)
    if bytes_eq(cmd, b"clear") {
        print!("\x1b[2J\x1b[H");
        return;
    }

    // Builtin: echo
    if starts_with(cmd, b"echo ") {
        if let Ok(s) = core::str::from_utf8(&cmd[5..]) {
            println!("{}", s);
        }
        return;
    }
    if bytes_eq(cmd, b"echo") {
        println!();
        return;
    }

    // Unknown command
    print!("Unknown: ");
    if let Ok(s) = core::str::from_utf8(cmd) {
        println!("{}", s);
    } else {
        println!("<invalid utf8>");
    }
}

/// Entry point for the shell.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!();
    println!("LevitateOS Shell (lsh) v0.1");
    println!("Type 'help' for commands.");
    println!();

    let mut buf = [0u8; 256];

    loop {
        print!("# ");
        let mut line_len = 0;
        'inner: loop {
            let mut c_buf = [0u8; 1];
            let n = libsyscall::read(0, &mut c_buf);
            if n > 0 {
                let bytes = &c_buf[..n as usize];
                // Echo back to user
                libsyscall::write(1, bytes);

                for &b in bytes {
                    if b == b'\n' || b == b'\r' {
                        if line_len > 0 {
                            execute(&buf[..line_len]);
                        }
                        break 'inner;
                    } else if b == 0x08 || b == 0x7f {
                        // Backspace
                        line_len = line_len.saturating_sub(1);
                    } else if line_len < buf.len() {
                        buf[line_len] = b;
                        line_len += 1;
                    }
                }
            }
        }
    }
}
