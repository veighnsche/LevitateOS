//! TEAM_193: `mv` utility for LevitateOS
//!
//! Moves (renames) files and directories.
//! See `docs/specs/levbox/mv.md` for specification.

#![no_std]
#![no_main]

extern crate alloc;
extern crate ulib;

use alloc::vec::Vec;
use core::panic::PanicInfo;
use libsyscall::{common_panic_handler, println, renameat};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

// ============================================================================
// Options
// ============================================================================

struct Options {
    force: bool,       // -f
    verbose: bool,     // -v
}

impl Default for Options {
    fn default() -> Self {
        Self {
            force: false,
            verbose: false,
        }
    }
}

// ============================================================================
// Help and Version
// ============================================================================

fn print_help() {
    println!("Usage: mv [OPTION]... SOURCE DEST");
    println!("       mv [OPTION]... SOURCE... DIRECTORY");
    println!("Rename SOURCE to DEST, or move SOURCE(s) to DIRECTORY.");
    println!();
    println!("  -f, --force      do not prompt before overwriting");
    println!("  -i, --interactive  prompt before overwrite (not implemented)");
    println!("  -v, --verbose    explain what is being done");
    println!("      --help       display this help and exit");
    println!("      --version    output version information and exit");
}

fn print_version() {
    println!("mv (LevitateOS levbox) 0.1.0");
}

// ============================================================================
// Core Logic
// ============================================================================

/// AT_FDCWD - current working directory
const AT_FDCWD: i32 = -100;

fn move_file(src: &str, dest: &str, opts: &Options) -> bool {
    let ret = renameat(AT_FDCWD, src, AT_FDCWD, dest);
    if ret < 0 {
        if !opts.force {
            libsyscall::write(2, b"mv: cannot move '");
            libsyscall::write(2, src.as_bytes());
            libsyscall::write(2, b"' to '");
            libsyscall::write(2, dest.as_bytes());
            libsyscall::write(2, b"': No such file or directory\n");
        }
        return opts.force;
    }

    if opts.verbose {
        libsyscall::write(1, b"renamed '");
        libsyscall::write(1, src.as_bytes());
        libsyscall::write(1, b"' -> '");
        libsyscall::write(1, dest.as_bytes());
        libsyscall::write(1, b"'\n");
    }

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

    let mut opts = Options::default();
    let mut files = Vec::new();

    let argc = ulib::env::args_len();
    for i in 1..argc {
        if let Some(arg) = ulib::env::arg(i) {
            if arg == "--help" {
                print_help();
                libsyscall::exit(0);
            } else if arg == "--version" {
                print_version();
                libsyscall::exit(0);
            } else if arg == "-f" || arg == "--force" {
                opts.force = true;
            } else if arg == "-i" || arg == "--interactive" {
                // Interactive mode not implemented
            } else if arg == "-v" || arg == "--verbose" {
                opts.verbose = true;
            } else if arg.starts_with('-') {
                println!("mv: invalid option -- '{}'", arg);
                libsyscall::exit(1);
            } else {
                files.push(arg);
            }
        }
    }

    if files.len() < 2 {
        println!("mv: missing file operand");
        println!("Try 'mv --help' for more information.");
        libsyscall::exit(1);
    }

    // Simple two-file rename for now
    if files.len() == 2 {
        let success = move_file(files[0], files[1], &opts);
        libsyscall::exit(if success { 0 } else { 1 })
    }

    // Multiple sources to directory - not implemented yet
    println!("mv: target '{}' is not a directory", files[files.len() - 1]);
    libsyscall::exit(1)
}
