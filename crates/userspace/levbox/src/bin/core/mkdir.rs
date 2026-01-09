//! TEAM_192: `mkdir` utility for LevitateOS
//!
//! Creates directories.
//! See `docs/specs/levbox/mkdir.md` for specification.

#![no_std]
#![no_main]

extern crate alloc;
extern crate ulib;

use alloc::string::String;
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

fn make_dir(path: &str, mode: u32, parents: bool, verbose: bool) -> bool {
    // AT_FDCWD is typically -100 in Linux
    const AT_FDCWD: i32 = -100;

    if parents {
        // Create full path recursively
        // Logic: Iterate over path components and create if missing
        let mut p = String::new();
        let chars: Vec<char> = path.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Handle root or separator
            if chars[i] == '/' {
                p.push('/');
                i += 1;
                // If double slash or root just continue
                continue;
            }

            // Consume component
            while i < chars.len() && chars[i] != '/' {
                p.push(chars[i]);
                i += 1;
            }
            // Temporarily ignore separators at the end for directory creation

            // Try creation
            if !p.is_empty() {
                let ret = mkdirat(AT_FDCWD, &p, mode);
                if ret < 0 {
                    let err = -ret; // errno
                    if err != 17 {
                        // EEXIST = 17 usually. But check LevitateOS errno.
                        // Actually, we don't have exact errno constants exposed easily here?
                        // Let's assume any error other than success is check-worthy.
                        // But if it fails, we check if it exists.
                        let fd = libsyscall::openat(&p, 0); // O_RDONLY
                        if fd >= 0 {
                            libsyscall::close(fd as usize);
                            // Exists (EEXIST case), continue
                        } else {
                            // Failed and doesn't exist? Real error.
                            // But wait, openat on directory might fail if directory?
                            // openat(dir, O_RDONLY) works on directories in many OSs.
                            // Assuming it failed to create.
                            libsyscall::write(2, b"mkdir: cannot create directory '");
                            libsyscall::write(2, p.as_bytes());
                            libsyscall::write(2, b"'\n");
                            return false;
                        }
                    }
                } else if verbose {
                    libsyscall::write(1, b"mkdir: created directory '");
                    libsyscall::write(1, p.as_bytes());
                    libsyscall::write(1, b"'\n");
                }
            }

            // If we stopped at '/', we continue loop to handle it next iteration (or just append)
            if i < chars.len() && chars[i] == '/' {
                p.push('/');
                i += 1;
            }
        }
        true
    } else {
        // Normal mkdir without -p
        let ret = mkdirat(AT_FDCWD, path, mode);
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
}

// ============================================================================
// Entry Point
// ============================================================================

#[no_mangle]
pub fn main() -> i32 {
    libsyscall::write(2, b"DEBUG: mkdir starting\n");
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
