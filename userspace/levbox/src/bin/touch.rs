//! TEAM_198: `touch` utility for LevitateOS
//!
//! Change file timestamps or create empty files.
//! See `docs/specs/levbox/touch.md` for specification.

#![no_std]
#![no_main]

extern crate alloc;
extern crate ulib;

use alloc::vec::Vec;
use core::panic::PanicInfo;
use libsyscall::{common_panic_handler, openat, utimensat, println, Timespec, UTIME_NOW};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

// ============================================================================
// Constants
// ============================================================================

const AT_FDCWD: i32 = -100;
const O_CREAT: u32 = 0o100;
const O_WRONLY: u32 = 0o1;

// ============================================================================
// Help and Version
// ============================================================================

fn print_help() {
    println!("Usage: touch [OPTION]... FILE...");
    println!("Update the access and modification times of each FILE to the current time.");
    println!();
    println!("A FILE argument that does not exist is created empty, unless -c is supplied.");
    println!();
    println!("  -a                  change only the access time");
    println!("  -c, --no-create     do not create any files");
    println!("  -m                  change only the modification time");
    println!("      --help          display this help and exit");
    println!("      --version       output version information and exit");
}

fn print_version() {
    println!("touch (LevitateOS levbox) 0.1.0");
}

// ============================================================================
// Core Logic
// ============================================================================

fn touch_file(path: &str, no_create: bool, atime_only: bool, mtime_only: bool) -> bool {
    // First, try to update timestamps on existing file
    let times = [
        Timespec {
            tv_sec: 0,
            tv_nsec: if mtime_only { libsyscall::UTIME_OMIT } else { UTIME_NOW },
        },
        Timespec {
            tv_sec: 0,
            tv_nsec: if atime_only { libsyscall::UTIME_OMIT } else { UTIME_NOW },
        },
    ];

    let ret = utimensat(AT_FDCWD, path, Some(&times), 0);
    
    if ret == 0 {
        return true; // Success - file existed and timestamps updated
    }

    // File doesn't exist - check if we should create it
    if no_create {
        return true; // -c flag: don't create, but don't error
    }

    // Try to create the file
    let ret = openat(AT_FDCWD, path, O_CREAT | O_WRONLY, 0o644);
    if ret < 0 {
        libsyscall::write(2, b"touch: cannot touch '");
        libsyscall::write(2, path.as_bytes());
        libsyscall::write(2, b"': ");
        if ret == -30 {
            libsyscall::write(2, b"Read-only file system\n");
        } else {
            libsyscall::write(2, b"Error\n");
        }
        return false;
    }

    // Close the file
    libsyscall::close(ret as usize);
    true
}

// ============================================================================
// Entry Point
// ============================================================================

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let sp: *const usize;
    unsafe {
        core::arch::asm!("mov {}, sp", out(reg) sp);
        ulib::env::init_args(sp);
    }

    let mut no_create = false;
    let mut atime_only = false;
    let mut mtime_only = false;
    let mut files: Vec<&str> = Vec::new();

    for arg in ulib::env::args().skip(1) {
        if arg == "--help" {
            print_help();
            libsyscall::exit(0);
        } else if arg == "--version" {
            print_version();
            libsyscall::exit(0);
        } else if arg == "-c" || arg == "--no-create" {
            no_create = true;
        } else if arg == "-a" {
            atime_only = true;
        } else if arg == "-m" {
            mtime_only = true;
        } else if arg.starts_with('-') {
            // Parse combined flags like -ac
            for c in arg.chars().skip(1) {
                match c {
                    'a' => atime_only = true,
                    'c' => no_create = true,
                    'm' => mtime_only = true,
                    _ => {
                        libsyscall::write(2, b"touch: invalid option -- '");
                        libsyscall::write(2, &[c as u8]);
                        libsyscall::write(2, b"'\n");
                        libsyscall::exit(1);
                    }
                }
            }
        } else {
            files.push(arg);
        }
    }

    if files.is_empty() {
        libsyscall::write(2, b"touch: missing file operand\n");
        libsyscall::write(2, b"Try 'touch --help' for more information.\n");
        libsyscall::exit(1);
    }

    let mut exit_code = 0;
    for file in files {
        if !touch_file(file, no_create, atime_only, mtime_only) {
            exit_code = 1;
        }
    }

    libsyscall::exit(exit_code);
}
