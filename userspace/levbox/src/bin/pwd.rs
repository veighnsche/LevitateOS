//! TEAM_192: `pwd` utility for LevitateOS
//!
//! Prints the current working directory.
//! See `docs/specs/levbox/pwd.md` for specification.

#![no_std]
#![no_main]

extern crate ulib;

use core::panic::PanicInfo;
use libsyscall::{common_panic_handler, getcwd, println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

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
pub extern "C" fn _start() -> ! {
    let sp: *const usize;
    unsafe {
        core::arch::asm!("mov {}, sp", out(reg) sp);
        ulib::env::init_args(sp);
    }

    let argc = ulib::env::args_len();
    for i in 1..argc {
        if let Some(arg) = ulib::env::arg(i) {
            if arg == "--help" {
                print_help();
                libsyscall::exit(0);
            } else if arg == "--version" {
                print_version();
                libsyscall::exit(0);
            } else if arg == "-L" || arg == "--logical" || arg == "-P" || arg == "--physical" {
                // Ignore these for now as we don't have symlink dirs in initramfs
            } else if arg.starts_with('-') {
                println!("pwd: invalid option -- '{}'", arg);
                libsyscall::exit(1);
            }
        }
    }

    let mut buf = [0u8; 256];
    let ret = getcwd(&mut buf);
    if ret < 0 {
        println!("pwd: error getting current directory");
        libsyscall::exit(1);
    }

    // getcwd returns length including null terminator
    let len = (ret as usize).saturating_sub(1);
    if let Ok(s) = core::str::from_utf8(&buf[..len]) {
        println!("{}", s);
    } else {
        println!("pwd: invalid UTF-8 in directory name");
        libsyscall::exit(1);
    }

    libsyscall::exit(0)
}
