//! TEAM_192: `pwd` utility for LevitateOS
//!
//! Prints the current working directory.
//! See `docs/specs/levbox/pwd.md` for specification.

#![no_std]
#![no_main]

extern crate ulib;

use libsyscall::{getcwd, println};

// ============================================================================
// Help and Version
// ============================================================================

fn print_help() {
    println!("Usage: pwd [OPTION]...");
    println!("Print the full filename of the current working directory.");
    println!();
    println!("  -L, --logical     use PWD from environment, even if it contains symlinks");
    println!("  -P, --physical    avoid all symlinks");
    println!("      --help        display this help and exit");
    println!("      --version     output version information and exit");
}

fn print_version() {
    println!("pwd (LevitateOS levbox) 0.1.0");
}

// ============================================================================
// Entry Point
// ============================================================================

#[no_mangle]
pub fn main() -> i32 {
    let argc = ulib::env::args_len();
    for i in 1..argc {
        if let Some(arg) = ulib::env::arg(i) {
            if arg == "--help" {
                print_help();
                return 0;
            } else if arg == "--version" {
                print_version();
                return 0;
            } else if arg == "-L" || arg == "--logical" || arg == "-P" || arg == "--physical" {
                // Ignore these for now as we don't have symlink dirs in initramfs
            } else if arg.starts_with('-') {
                println!("pwd: invalid option -- '{}'", arg);
                return 1;
            }
        }
    }

    let mut buf = [0u8; 256];
    let ret = getcwd(&mut buf);
    if ret < 0 {
        println!("pwd: error getting current directory");
        return 1;
    }

    // getcwd returns length including null terminator
    let len = (ret as usize).saturating_sub(1);
    if let Ok(s) = core::str::from_utf8(&buf[..len]) {
        println!("{}", s);
    } else {
        println!("pwd: invalid UTF-8 in directory name");
        return 1;
    }

    0
}
