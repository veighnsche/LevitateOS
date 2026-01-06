//! TEAM_182: `cat` utility for LevitateOS
//!
//! Concatenates and prints files to standard output.
//! See `docs/specs/levbox/cat.md` for specification.
//!
//! ## Usage
//! ```
//! cat [-u] [file...]
//! ```
//!
//! ## Behavior IDs
//! - [CAT1] Read file and output to stdout
//! - [CAT2] Read from stdin when no files or "-" operand
//! - [CAT3] Continue on error, report to stderr

#![no_std]
#![no_main]

extern crate alloc;
// TEAM_182: ulib provides #[global_allocator] and #[alloc_error_handler]
extern crate ulib;

use core::panic::PanicInfo;
use libsyscall::common_panic_handler;

// File descriptors
const STDIN: usize = 0;
const STDOUT: usize = 1;
const STDERR: usize = 2;

// Buffer size per spec recommendation
const BUF_SIZE: usize = 4096;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

// ============================================================================
// Error Handling
// ============================================================================

/// Write error message to stderr [CAT3]
fn eprint(msg: &str) {
    let _ = libsyscall::write(STDERR, msg.as_bytes());
}

fn eprintln(msg: &str) {
    eprint(msg);
    let _ = libsyscall::write(STDERR, b"\n");
}

// ============================================================================
// Core Cat Functions
// ============================================================================

/// [CAT2] Cat stdin to stdout until EOF
fn cat_stdin() -> bool {
    let mut buf = [0u8; BUF_SIZE];
    loop {
        let n = libsyscall::read(STDIN, &mut buf);
        if n < 0 {
            eprintln("cat: stdin: read error");
            return false;
        }
        if n == 0 {
            break; // EOF
        }
        let written = libsyscall::write(STDOUT, &buf[..n as usize]);
        if written < 0 {
            eprintln("cat: stdout: write error");
            return false;
        }
    }
    true
}

/// [CAT1] Cat a file to stdout
fn cat_file(path: &str) -> bool {
    let fd = libsyscall::openat(path, 0); // 0 = read-only
    if fd < 0 {
        eprint("cat: ");
        eprint(path);
        eprintln(": cannot open file");
        return false;
    }

    let mut buf = [0u8; BUF_SIZE];
    let fd_usize = fd as usize;
    let mut success = true;

    loop {
        let n = libsyscall::read(fd_usize, &mut buf);
        if n < 0 {
            eprint("cat: ");
            eprint(path);
            eprintln(": read error");
            success = false;
            break;
        }
        if n == 0 {
            break; // EOF
        }

        let written = libsyscall::write(STDOUT, &buf[..n as usize]);
        if written < 0 {
            eprint("cat: ");
            eprint(path);
            eprintln(": write error");
            success = false;
            break;
        }
    }

    let _ = libsyscall::close(fd_usize);
    success
}

// ============================================================================
// Entry Point
// ============================================================================

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize environment/args from stack (Linux ABI)
    // The stack pointer at entry contains argc, argv, envp
    let sp: *const usize;
    unsafe {
        core::arch::asm!("mov {}, sp", out(reg) sp);
        ulib::env::init_args(sp);
    }

    let mut exit_code = 0i32;
    let argc = ulib::env::args_len();

    if argc <= 1 {
        // [CAT2] No arguments: read from stdin
        if !cat_stdin() {
            exit_code = 1;
        }
    } else {
        // Process each file argument
        for i in 1..argc {
            if let Some(arg) = ulib::env::arg(i) {
                if arg == "-" {
                    // [CAT2] "-" means stdin
                    if !cat_stdin() {
                        exit_code = 1;
                    }
                } else if arg == "-u" {
                    // Unbuffered mode - no-op (we're already unbuffered)
                } else if arg.starts_with('-') {
                    // Unknown option
                    eprint("cat: invalid option: ");
                    eprintln(arg);
                    exit_code = 1;
                } else {
                    // [CAT1] Regular file
                    if !cat_file(arg) {
                        exit_code = 1;
                    }
                }
            }
        }
    }

    libsyscall::exit(exit_code)
}
