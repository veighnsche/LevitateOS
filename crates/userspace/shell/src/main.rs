//! `TEAM_081`: `LevitateOS` Shell (`lsh`)
//!
//! Interactive shell for `LevitateOS` Phase 8b.
//! Supports builtin commands: `echo`, `help`, `clear`, `exit`
//!
//! `TEAM_118`: Refactored to use `libsyscall`.
//!
//! TEAM_158: Behavior IDs [SH1]-[SH7] for traceability.

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

    // [SH7] Builtin: exit command terminates shell
    // exit         -> graceful shutdown (minimal output)
    // exit --verbose -> graceful shutdown with golden file output
    if bytes_eq(cmd, b"exit") {
        println!("Goodbye!");
        libsyscall::shutdown(libsyscall::shutdown_flags::NORMAL); // [SH7]
    }
    if bytes_eq(cmd, b"exit --verbose") {
        println!("Goodbye! (verbose shutdown for golden file)");
        libsyscall::shutdown(libsyscall::shutdown_flags::VERBOSE); // [SH7]
    }

    // Builtin: test (POSIX: exit 0 if no args, used for shell scripting)
    if bytes_eq(cmd, b"test") {
        // No output, exit 0 (implicit return in shell)
        return;
    }

    // [SH6] Builtin: help command shows available commands
    if bytes_eq(cmd, b"help") {
        println!("LevitateOS Shell (lsh) v0.1");
        println!("Commands:");
        println!("  echo <text>    - Print text");
        println!("  help           - Show this help");
        println!("  clear          - Clear screen");
        println!("  exit           - Shutdown system");
        println!("  exit --verbose - Shutdown with detailed output");
        println!("  test           - Exit 0 (for scripting)");
        return; // [SH6]
    }

    // Builtin: clear (ANSI escape)
    if bytes_eq(cmd, b"clear") {
        print!("\x1b[2J\x1b[H");
        return;
    }

    // [SH5] Builtin: echo command outputs text
    if starts_with(cmd, b"echo ") {
        if let Ok(s) = core::str::from_utf8(&cmd[5..]) {
            println!("{}", s); // [SH5] echo outputs text
        }
        return;
    }
    if bytes_eq(cmd, b"echo") {
        println!(); // [SH5] empty echo
        return;
    }

    // [SH8] External command execution with argument passing
    // TEAM_186: Parse command line and spawn with arguments
    println!("[DEBUG] execute: splitting args...");
    let mut result = -1isize;
    let (parts, argc) = split_args(cmd);
    println!("[DEBUG] execute: argc={}", argc);
    if argc > 0 {
        // Convert parts to str slices
        let mut argv_strs: [&str; 16] = [""; 16];
        let mut valid = true;
        for i in 0..argc {
            match core::str::from_utf8(parts[i]) {
                Ok(s) => argv_strs[i] = s,
                Err(_) => {
                    valid = false;
                    break;
                }
            }
        }

        if valid {
            // First part is the command name
            let cmd_name = argv_strs[0];
            println!("[DEBUG] execute: cmd_name='{}'", cmd_name);

            // Build path (prepend / if needed)
            // Use a static buffer since we can't allocate
            static mut PATH_BUF: [u8; 64] = [0; 64];
            let path = if cmd_name.starts_with('/') {
                cmd_name
            } else {
                unsafe {
                    PATH_BUF[0] = b'/';
                    let len = cmd_name.len().min(62);
                    PATH_BUF[1..1 + len].copy_from_slice(&cmd_name.as_bytes()[..len]);
                    // SAFETY: We just wrote valid UTF-8 bytes
                    core::str::from_utf8_unchecked(&PATH_BUF[..1 + len])
                }
            };

            // Spawn with all arguments
            println!("[DEBUG] execute: spawning {}...", path);
            result = libsyscall::spawn_args(path, &argv_strs[..argc]);
            println!("[DEBUG] execute: spawn result={}", result);
            if result >= 0 {
                // Process spawned successfully with PID = result
                // TEAM_188: Wait for child process to complete
                let child_pid = result as i32;

                // TEAM_220: Set child as foreground
                libsyscall::set_foreground(child_pid as usize);

                let mut status: i32 = 0;
                libsyscall::waitpid(child_pid, Some(&mut status));

                // TEAM_220: Restore shell as foreground
                libsyscall::set_foreground(libsyscall::getpid() as usize);
                return;
            }
        }
    }

    // Unknown command
    print!("Unknown: ");
    if let Ok(s) = core::str::from_utf8(cmd) {
        println!("{} (err={})", s, result);
    } else {
        println!("<invalid utf8> (err={})", result);
    }
}

/// TEAM_186: Split command line into whitespace-separated parts
fn split_args(cmd: &[u8]) -> ([&[u8]; 16], usize) {
    let mut parts: [&[u8]; 16] = [&[]; 16];
    let mut count = 0;
    let mut i = 0;

    while i < cmd.len() && count < 16 {
        // Skip whitespace
        while i < cmd.len() && (cmd[i] == b' ' || cmd[i] == b'\t') {
            i += 1;
        }
        if i >= cmd.len() {
            break;
        }

        // Find end of word
        let start = i;
        while i < cmd.len() && cmd[i] != b' ' && cmd[i] != b'\t' {
            i += 1;
        }

        parts[count] = &cmd[start..i];
        count += 1;
    }

    (parts, count)
}

/// [SH1] Entry point for the shell - prints banner on startup.
/// [SH2] Prints # prompt.
/// [SH3] Reads input line.
/// [SH1] Entry point for the shell - prints banner on startup.
/// [SH2] Prints # prompt.
/// [SH3] Reads input line.
#[no_mangle]
pub extern "C" fn shell_entry() -> ! {
    // [SH1] Shell prints banner on startup
    println!();
    println!("LevitateOS Shell (lsh) v0.1");
    println!("Type 'help' for commands.");
    println!();

    // TEAM_220: Ignore Ctrl+C in shell itself
    // TEAM_244: Use sigreturn_trampoline for proper signal return
    extern "C" fn sigint_handler(_sig: i32) {
        // Just print a newline and a new prompt if we're idle?
        // For now, doing nothing is better than exiting.
    }
    // TEAM_244: Local trampoline that calls sigreturn after handler
    extern "C" fn sigreturn_trampoline() -> ! {
        libsyscall::sigreturn()
    }
    libsyscall::sigaction(
        libsyscall::SIGINT as i32,
        sigint_handler as *const () as usize,
        sigreturn_trampoline as *const () as usize,
    );

    // TEAM_220: Set shell as foreground on startup
    libsyscall::set_foreground(libsyscall::getpid() as usize);

    let mut buf = [0u8; 256];

    loop {
        print!("# "); // [SH2] Shell prints # prompt
        let mut line_len = 0; // [SH3] reads input line
        'inner: loop {
            let mut c_buf = [0u8; 1];
            let n = libsyscall::read(0, &mut c_buf);
            if n > 0 {
                let b = c_buf[0];

                if b == b'\n' || b == b'\r' {
                    // Echo newline and execute
                    libsyscall::write(1, b"\n");
                    if line_len > 0 {
                        execute(&buf[..line_len]);
                    }
                    break 'inner;
                } else if b == 0x08 || b == 0x7f {
                    // Backspace: erase character from buffer and screen
                    if line_len > 0 {
                        line_len -= 1;
                        // Move back, overwrite with space, move back again
                        libsyscall::write(1, b"\x08 \x08");
                    }
                } else if line_len < buf.len() {
                    // Normal character: add to buffer and echo
                    buf[line_len] = b;
                    line_len += 1;
                    libsyscall::write(1, &c_buf[..1]);
                }
            } else if n == 0 {
                // [SH7] EOF (Ctrl+D) - exit shell
                println!("EOF");
                libsyscall::shutdown(libsyscall::shutdown_flags::NORMAL);
            } else {
                // Error reading
                println!("Read error: {}", n);
                libsyscall::shutdown(libsyscall::shutdown_flags::NORMAL);
            }
        }
    }
}

/// TEAM_299: Naked entry point to align stack for x86_64.
/// The kernel might not align the stack to 16 bytes when jumping to userspace.
/// This trampoline ensures alignment to prevent SIMD/Rust ABI issues.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "xor rbp, rbp",      // Clear frame pointer
        "mov rdi, rsp",      // Save original stack pointer (ignoring args for now)
        "and rsp, -16",      // Align stack to 16 bytes
        "call {entry}",      // Call Rust entry point
        "ud2",               // Should not return
        entry = sym shell_entry,
    )
}

/// TEAM_304: Naked entry point for aarch64.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "mov x29, #0",       // Clear frame pointer
        "mov x30, #0",       // Clear link register
        "mov x0, sp",        // Preserve original SP
        "and sp, x0, #-16",  // Align stack to 16 bytes
        "b {entry}",         // Tail call to Rust entry
        entry = sym shell_entry,
    )
}
