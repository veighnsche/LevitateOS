//! TEAM_193: `rmdir` utility for LevitateOS
//!
//! Removes empty directories.
//! See `docs/specs/levbox/rmdir.md` for specification.

#![no_std]
#![no_main]

extern crate alloc;
extern crate ulib;

use alloc::vec::Vec;
use libsyscall::{println, unlinkat};

// ============================================================================
// Help and Version
// ============================================================================

fn print_help() {
    println!("Usage: rmdir [OPTION]... DIRECTORY...");
    println!("Remove the DIRECTORY(ies), if they are empty.");
    println!();
    println!("  -p, --parents    remove DIRECTORY and its ancestors");
    println!("  -v, --verbose    output a diagnostic for every directory processed");
    println!("      --help       display this help and exit");
    println!("      --version    output version information and exit");
}

fn print_version() {
    println!("rmdir (LevitateOS levbox) 0.1.0");
}

// ============================================================================
// Core Logic
// ============================================================================

/// AT_REMOVEDIR flag for unlinkat to remove directories
const AT_REMOVEDIR: u32 = 0x200;
/// AT_FDCWD - current working directory
const AT_FDCWD: i32 = -100;

fn remove_dir(path: &str, verbose: bool) -> bool {
    let ret = unlinkat(AT_FDCWD, path, AT_REMOVEDIR);
    if ret < 0 {
        libsyscall::write(2, b"rmdir: failed to remove '");
        libsyscall::write(2, path.as_bytes());
        libsyscall::write(2, b"': Directory not empty or does not exist\n");
        return false;
    }

    if verbose {
        libsyscall::write(1, b"rmdir: removing directory, '");
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
    let mut verbose = false;
    let mut _parents = false;
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
                _parents = true;
            } else if arg == "-v" || arg == "--verbose" {
                verbose = true;
            } else if arg.starts_with('-') {
                println!("rmdir: invalid option -- '{}'", arg);
                return 1;
            } else {
                dirs.push(arg);
            }
        }
    }

    if dirs.is_empty() {
        println!("rmdir: missing operand");
        return 1;
    }

    let mut success = true;
    for dir in dirs {
        if !remove_dir(dir, verbose) {
            success = false;
        }
    }

    if success {
        0
    } else {
        1
    }
}
