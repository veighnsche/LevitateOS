//! TEAM_193: `rm` utility for LevitateOS
//!
//! Removes files and directories.
//! See `docs/specs/levbox/rm.md` for specification.

#![no_std]
#![no_main]

extern crate alloc;
extern crate ulib;

use alloc::vec::Vec;
use core::panic::PanicInfo;
use libsyscall::{common_panic_handler, println, unlinkat};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

// ============================================================================
// Options
// ============================================================================

struct Options {
    force: bool,       // -f
    recursive: bool,   // -r, -R
    dir: bool,         // -d (remove empty directories)
    verbose: bool,     // -v
}

impl Default for Options {
    fn default() -> Self {
        Self {
            force: false,
            recursive: false,
            dir: false,
            verbose: false,
        }
    }
}

// ============================================================================
// Help and Version
// ============================================================================

fn print_help() {
    println!("Usage: rm [OPTION]... [FILE]...");
    println!("Remove (unlink) the FILE(s).");
    println!();
    println!("  -f, --force      ignore nonexistent files and arguments");
    println!("  -r, -R, --recursive  remove directories and their contents recursively");
    println!("  -d, --dir        remove empty directories");
    println!("  -v, --verbose    explain what is being done");
    println!("      --help       display this help and exit");
    println!("      --version    output version information and exit");
}

fn print_version() {
    println!("rm (LevitateOS levbox) 0.1.0");
}

// ============================================================================
// Core Logic
// ============================================================================

/// AT_REMOVEDIR flag for unlinkat to remove directories
const AT_REMOVEDIR: u32 = 0x200;
/// AT_FDCWD - current working directory
const AT_FDCWD: i32 = -100;

fn remove_file(path: &str, opts: &Options) -> bool {
    // Try to remove as a file first (flags = 0)
    let ret = unlinkat(AT_FDCWD, path, 0);
    if ret >= 0 {
        if opts.verbose {
            libsyscall::write(1, b"removed '");
            libsyscall::write(1, path.as_bytes());
            libsyscall::write(1, b"'\n");
        }
        return true;
    }

    // If -d flag, try to remove as empty directory
    if opts.dir {
        let ret = unlinkat(AT_FDCWD, path, AT_REMOVEDIR);
        if ret >= 0 {
            if opts.verbose {
                libsyscall::write(1, b"removed directory '");
                libsyscall::write(1, path.as_bytes());
                libsyscall::write(1, b"'\n");
            }
            return true;
        }
    }

    // TODO: Handle -r recursive removal

    if !opts.force {
        libsyscall::write(2, b"rm: cannot remove '");
        libsyscall::write(2, path.as_bytes());
        libsyscall::write(2, b"': No such file or directory\n");
    }

    opts.force // Return true if force mode (ignore errors)
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
            } else if arg == "-r" || arg == "-R" || arg == "--recursive" {
                opts.recursive = true;
            } else if arg == "-d" || arg == "--dir" {
                opts.dir = true;
            } else if arg == "-v" || arg == "--verbose" {
                opts.verbose = true;
            } else if arg == "-rf" || arg == "-fr" {
                opts.recursive = true;
                opts.force = true;
            } else if arg.starts_with('-') && arg.len() > 1 {
                // Handle combined short options like -rv
                for c in arg.chars().skip(1) {
                    match c {
                        'f' => opts.force = true,
                        'r' | 'R' => opts.recursive = true,
                        'd' => opts.dir = true,
                        'v' => opts.verbose = true,
                        _ => {
                            println!("rm: invalid option -- '{}'", c);
                            libsyscall::exit(1);
                        }
                    }
                }
            } else {
                files.push(arg);
            }
        }
    }

    if files.is_empty() {
        println!("rm: missing operand");
        libsyscall::exit(1);
    }

    let mut success = true;
    for file in files {
        if !remove_file(file, &opts) {
            success = false;
        }
    }

    libsyscall::exit(if success { 0 } else { 1 })
}
