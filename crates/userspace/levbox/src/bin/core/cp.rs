//! TEAM_193: `cp` utility for LevitateOS
//!
//! Copies files and directories.
//! See `docs/specs/levbox/cp.md` for specification.

#![no_std]
#![no_main]

extern crate alloc;
extern crate ulib;

use alloc::vec::Vec;
use libsyscall::println;
use ulib::fs::File;
use ulib::io::{Read, Write};

// ============================================================================
// Help and Version
// ============================================================================

fn print_help() {
    println!("Usage: cp [OPTION]... SOURCE DEST");
    println!("       cp [OPTION]... SOURCE... DIRECTORY");
    println!("Copy SOURCE to DEST, or multiple SOURCE(s) to DIRECTORY.");
    println!();
    println!("  -f, --force          if destination exists, remove it first");
    println!("  -i, --interactive    prompt before overwrite");
    println!("  -p, --preserve       preserve mode, ownership, timestamps");
    println!("  -R, -r, --recursive  copy directories recursively");
    println!("      --help           display this help and exit");
    println!("      --version        output version information and exit");
}

fn print_version() {
    println!("cp (LevitateOS levbox) 0.1.0");
}

// ============================================================================
// Options
// ============================================================================

struct Options {
    force: bool,
    interactive: bool,
    preserve: bool,
    recursive: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            force: false,
            interactive: false,
            preserve: false,
            recursive: false,
        }
    }
}

// ============================================================================
// Core Logic
// ============================================================================

fn copy_file(src: &str, dest: &str, _opts: &Options) -> bool {
    // Open source file
    let mut src_file = match File::open(src) {
        Ok(f) => f,
        Err(_) => {
            libsyscall::write(2, b"cp: cannot stat '");
            libsyscall::write(2, src.as_bytes());
            libsyscall::write(2, b"': No such file or directory\n");
            return false;
        }
    };

    // Read source file content
    let mut buf = [0u8; 4096];
    let mut content = Vec::new();
    loop {
        match src_file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => content.extend_from_slice(&buf[..n]),
            Err(_) => {
                libsyscall::write(2, b"cp: error reading '");
                libsyscall::write(2, src.as_bytes());
                libsyscall::write(2, b"'\n");
                return false;
            }
        }
    }

    // TEAM_256: Create destination file and write content
    let mut dest_file = match File::create(dest) {
        Ok(f) => f,
        Err(_) => {
            libsyscall::write(2, b"cp: cannot create '");
            libsyscall::write(2, dest.as_bytes());
            libsyscall::write(2, b"': Permission denied\n");
            return false;
        }
    };

    // Write content to destination
    match dest_file.write(&content) {
        Ok(_) => true,
        Err(_) => {
            libsyscall::write(2, b"cp: error writing '");
            libsyscall::write(2, dest.as_bytes());
            libsyscall::write(2, b"'\n");
            false
        }
    }
}

// ============================================================================
// Entry Point
// ============================================================================

#[no_mangle]
pub fn main() -> i32 {
    let mut opts = Options::default();
    let mut files = Vec::new();

    let argc = ulib::env::args_len();
    for i in 1..argc {
        if let Some(arg) = ulib::env::arg(i) {
            if arg == "--help" {
                print_help();
                return 0;
            } else if arg == "--version" {
                print_version();
                return 0;
            } else if arg == "-f" || arg == "--force" {
                opts.force = true;
            } else if arg == "-i" || arg == "--interactive" {
                opts.interactive = true;
            } else if arg == "-p" || arg == "--preserve" {
                opts.preserve = true;
            } else if arg == "-R" || arg == "-r" || arg == "--recursive" {
                opts.recursive = true;
            } else if arg.starts_with('-') {
                println!("cp: invalid option -- '{}'", arg);
                return 1;
            } else {
                files.push(arg);
            }
        }
    }

    if files.len() < 2 {
        println!("cp: missing file operand");
        println!("Try 'cp --help' for more information.");
        return 1;
    }

    // Simple two-file copy for now
    if files.len() == 2 {
        let success = copy_file(files[0], files[1], &opts);
        return if success { 0 } else { 1 };
    }

    // Multiple sources to directory
    println!("cp: target '{}' is not a directory", files[files.len() - 1]);
    1
}
