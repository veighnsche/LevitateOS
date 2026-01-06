//! TEAM_198: `ln` utility for LevitateOS
//!
//! Create links between files.
//! See `docs/specs/levbox/ln.md` for specification.

#![no_std]
#![no_main]

extern crate alloc;
extern crate ulib;

use alloc::vec::Vec;
use libsyscall::{println, symlinkat};

// ============================================================================
// Constants
// ============================================================================

const AT_FDCWD: i32 = -100;

// ============================================================================
// Help and Version
// ============================================================================

fn print_help() {
    println!("Usage: ln [OPTION]... TARGET LINK_NAME");
    println!("Create a link to TARGET with the name LINK_NAME.");
    println!();
    println!("  -s, --symbolic      make symbolic links instead of hard links");
    println!("  -f, --force         remove existing destination files");
    println!("      --help          display this help and exit");
    println!("      --version       output version information and exit");
    println!();
    println!("Note: Hard links are not yet implemented. Use -s for symbolic links.");
}

fn print_version() {
    println!("ln (LevitateOS levbox) 0.1.0");
}

// ============================================================================
// Core Logic
// ============================================================================

fn create_link(target: &str, linkpath: &str, symbolic: bool, force: bool) -> bool {
    if !symbolic {
        libsyscall::write(2, b"ln: hard links not implemented\n");
        libsyscall::write(2, b"Try 'ln -s' for symbolic links.\n");
        return false;
    }

    // If force flag, try to remove existing link first
    if force {
        // Ignore errors - file might not exist
        libsyscall::unlinkat(AT_FDCWD, linkpath, 0);
    }

    let ret = symlinkat(target, AT_FDCWD, linkpath);
    if ret < 0 {
        libsyscall::write(2, b"ln: failed to create symbolic link '");
        libsyscall::write(2, linkpath.as_bytes());
        libsyscall::write(2, b"': ");
        if ret == -17 {
            libsyscall::write(2, b"File exists\n");
        } else if ret == -30 {
            libsyscall::write(2, b"Read-only file system\n");
        } else {
            libsyscall::write(2, b"Error\n");
        }
        return false;
    }

    true
}

// ============================================================================
// Entry Point
// ============================================================================

#[no_mangle]
pub fn main() -> i32 {
    let mut symbolic = false;
    let mut force = false;
    let mut paths: Vec<alloc::string::String> = Vec::new();

    for arg in ulib::env::args().skip(1) {
        if arg == "--help" {
            print_help();
            return 0;
        } else if arg == "--version" {
            print_version();
            return 0;
        } else if arg == "-s" || arg == "--symbolic" {
            symbolic = true;
        } else if arg == "-f" || arg == "--force" {
            force = true;
        } else if arg.starts_with('-') {
            // Parse combined flags like -sf
            for c in arg.chars().skip(1) {
                match c {
                    's' => symbolic = true,
                    'f' => force = true,
                    _ => {
                        libsyscall::write(2, b"ln: invalid option -- '");
                        libsyscall::write(2, &[c as u8]);
                        libsyscall::write(2, b"'\n");
                        return 1;
                    }
                }
            }
        } else {
            paths.push(alloc::string::String::from(arg));
        }
    }

    if paths.len() < 2 {
        libsyscall::write(2, b"ln: missing file operand\n");
        libsyscall::write(2, b"Try 'ln --help' for more information.\n");
        return 1;
    }

    if paths.len() > 2 {
        libsyscall::write(2, b"ln: too many arguments\n");
        return 1;
    }

    let target = &paths[0];
    let linkpath = &paths[1];

    if create_link(target, linkpath, symbolic, force) {
        0
    } else {
        1
    }
}
