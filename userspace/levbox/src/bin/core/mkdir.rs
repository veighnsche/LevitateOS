//! TEAM_192: `mkdir` utility for LevitateOS
//!
//! Creates directories.
//! See `docs/specs/levbox/mkdir.md` for specification.

#![no_std]
#![no_main]

extern crate alloc;
extern crate ulib;

use alloc::vec::Vec;
use libsyscall::{mkdirat, println};

// ============================================================================
// Help and Version
// ============================================================================

fn print_help() {
    println!("Usage: mkdir [OPTION]... DIRECTORY...");
    println!("Create the DIRECTORY(ies), if they do not already exist.");
    println!();
    println!("  -m, --mode=MODE   set file mode (as in chmod), not a=rwx - umask");
    println!("  -p, --parents     no error if existing, make parent directories as needed");
    println!("  -v, --verbose     print a message for each created directory");
    println!("      --help        display this help and exit");
    println!("      --version     output version information and exit");
}

fn print_version() {
    println!("mkdir (LevitateOS levbox) 0.1.0");
}

// ============================================================================
// Core Logic
// ============================================================================

fn make_dir(path: &str, _mode: u32, _parents: bool, verbose: bool) -> bool {
    // AT_FDCWD is typically -100 in Linux
    const AT_FDCWD: i32 = -100;

    // TODO: support -p (parents)
    let ret = mkdirat(AT_FDCWD, path, 0o755);
    if ret < 0 {
        libsyscall::write(2, b"mkdir: cannot create directory '");
        libsyscall::write(2, path.as_bytes());
        libsyscall::write(2, b"': Read-only file system (or other error)\n");
        return false;
    }

    if verbose {
        libsyscall::write(1, b"mkdir: created directory '");
        libsyscall::write(1, path.as_bytes());
        libsyscall::write(1, b"'\n");
    }

    true
}

// ============================================================================
// Entry Point
// ============================================================================

#[no_mangle]
pub fn main() -> i32 {
    let mut parents = false;
    let mut verbose = false;
    let mode = 0o755;
    let mut dirs = Vec::new();

    let argc = ulib::env::args_len();
    for i in 1..argc {
        if let Some(arg) = ulib::env::arg(i) {
            if arg == "--help" {
                print_help();
                return 0;
            } else if arg == "--version" {
                print_version();
                return 0;
            } else if arg == "-p" || arg == "--parents" {
                parents = true;
            } else if arg == "-v" || arg == "--verbose" {
                verbose = true;
            } else if arg.starts_with("-m") {
                // Ignore mode for now
            } else if arg.starts_with('-') {
                println!("mkdir: invalid option -- '{}'", arg);
                return 1;
            } else {
                dirs.push(arg);
            }
        }
    }

    if dirs.is_empty() {
        println!("mkdir: missing operand");
        return 1;
    }

    let mut success = true;
    for dir in dirs {
        if !make_dir(dir, mode, parents, verbose) {
            success = false;
        }
    }

    if success {
        0
    } else {
        1
    }
}
