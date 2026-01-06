//! TEAM_192: `ls` utility for LevitateOS
//!
//! Lists directory contents.
//! See `docs/specs/levbox/ls.md` for specification.

#![no_std]
#![no_main]

extern crate alloc;
extern crate ulib;

use alloc::string::String;
use alloc::vec::Vec;
use libsyscall::println;
use ulib::fs::{read_dir, File, FileType};

// ============================================================================
// Options
// ============================================================================

struct Options {
    all: bool,            // -a
    almost_all: bool,     // -A
    classify: bool,       // -F
    one_per_line: bool,   // -1
    long_format: bool,    // -l
    human_readable: bool, // -h
    recursive: bool,      // -R
    color: bool,          // --color
}

impl Default for Options {
    fn default() -> Self {
        Self {
            all: false,
            almost_all: false,
            classify: false,
            one_per_line: true, // Defaulting to 1 per line for now as we don't have column display
            long_format: false,
            human_readable: false,
            recursive: false,
            color: false,
        }
    }
}

// ============================================================================
// TEAM_193: ANSI Color Codes
// ============================================================================

const ANSI_RESET: &[u8] = b"\x1b[0m";
const ANSI_BLUE: &[u8] = b"\x1b[34m"; // Directories
const ANSI_CYAN: &[u8] = b"\x1b[36m"; // Symlinks
                                      // const ANSI_GREEN: &[u8] = b"\x1b[32m";  // Executables (not detected yet)

// ============================================================================
// Help and Version
// ============================================================================

fn print_help() {
    println!("Usage: ls [OPTION]... [FILE]...");
    println!("List information about the FILEs (the current directory by default).");
    println!();
    println!("  -a, --all            do not ignore entries starting with .");
    println!("  -A, --almost-all     do not list implied . and ..");
    println!("  -F, --classify       append indicator (one of */=>@|) to entries");
    println!("  -h, --human-readable with -l, print sizes like 1K 234M 2G");
    println!("  -l                   use a long listing format");
    println!("  -R, --recursive      list subdirectories recursively");
    println!("  -1                   list one file per line");
    println!("      --color          colorize the output");
    println!("      --help           display this help and exit");
    println!("      --version        output version information and exit");
}

fn print_version() {
    println!("ls (LevitateOS levbox) 0.1.0");
}

// ============================================================================
// TEAM_193: Formatting Helpers
// ============================================================================

/// TEAM_193: Format file size in human-readable format (1K, 234M, 2G)
fn format_size_human(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if size >= GB {
        let whole = size / GB;
        let frac = (size % GB) * 10 / GB;
        if frac > 0 {
            alloc::format!("{}{}G", whole, frac)
        } else {
            alloc::format!("{}G", whole)
        }
    } else if size >= MB {
        let whole = size / MB;
        let frac = (size % MB) * 10 / MB;
        if frac > 0 {
            alloc::format!("{}{}M", whole, frac)
        } else {
            alloc::format!("{}M", whole)
        }
    } else if size >= KB {
        let whole = size / KB;
        let frac = (size % KB) * 10 / KB;
        if frac > 0 {
            alloc::format!("{}{}K", whole, frac)
        } else {
            alloc::format!("{}K", whole)
        }
    } else {
        alloc::format!("{}", size)
    }
}

/// TEAM_193: Get file type character for long listing
fn file_type_char(ft: FileType) -> char {
    match ft {
        FileType::Directory => 'd',
        FileType::File => '-',
        FileType::Symlink => 'l',
        FileType::Other => '?',
    }
}

/// TEAM_193: Build full path from directory and file name
fn join_path(dir: &str, name: &str) -> String {
    if dir == "." {
        String::from(name)
    } else if dir.ends_with('/') {
        alloc::format!("{}{}", dir, name)
    } else {
        alloc::format!("{}/{}", dir, name)
    }
}

// ============================================================================
// Core Logic
// ============================================================================

fn list_dir(path: &str, opts: &Options) -> bool {
    let entries = match read_dir(path) {
        Ok(iter) => iter,
        Err(_) => {
            libsyscall::write(2, b"ls: cannot access '");
            libsyscall::write(2, path.as_bytes());
            libsyscall::write(2, b"': No such file or directory\n");
            return false;
        }
    };

    let mut collected = Vec::new();
    for entry in entries {
        match entry {
            Ok(e) => {
                let name = e.file_name();

                // Filtering
                if !opts.all && !opts.almost_all && name.starts_with('.') {
                    continue;
                }
                if opts.almost_all && (name == "." || name == "..") {
                    continue;
                }

                collected.push(e);
            }
            Err(_) => {
                libsyscall::write(2, b"ls: error reading directory\n");
                return false;
            }
        }
    }

    // Sorting (alphabetical)
    collected.sort_by(|a, b| a.file_name().cmp(b.file_name()));

    // TEAM_193: Collect subdirectories for recursive listing
    let mut subdirs: Vec<String> = Vec::new();

    for entry in collected {
        let name = entry.file_name();
        let file_type = entry.file_type();

        // TEAM_193: Long format output
        if opts.long_format {
            // File type character
            let type_char = file_type_char(file_type);
            libsyscall::write(1, &[type_char as u8]);

            // Permissions placeholder (we don't have full permission info)
            libsyscall::write(1, b"rw-r--r--");

            // Link count placeholder
            libsyscall::write(1, b"  1 ");

            // Owner/group placeholder
            libsyscall::write(1, b"root root ");

            // Get file size
            let full_path = join_path(path, name);
            let size = if let Ok(file) = File::open(&full_path) {
                if let Ok(meta) = file.metadata() {
                    meta.len()
                } else {
                    0
                }
            } else {
                0
            };

            // Size (human readable or bytes)
            let size_str = if opts.human_readable {
                format_size_human(size)
            } else {
                alloc::format!("{:>8}", size)
            };
            libsyscall::write(1, size_str.as_bytes());
            libsyscall::write(1, b" ");
        }

        // TEAM_193: File name with optional color
        if opts.color {
            match file_type {
                FileType::Directory => libsyscall::write(1, ANSI_BLUE),
                FileType::Symlink => libsyscall::write(1, ANSI_CYAN),
                _ => 0,
            };
        }
        libsyscall::write(1, name.as_bytes());
        if opts.color {
            libsyscall::write(1, ANSI_RESET);
        }

        if opts.classify {
            match file_type {
                FileType::Directory => libsyscall::write(1, b"/"),
                FileType::Symlink => libsyscall::write(1, b"@"),
                _ => 0,
            };
        }

        if opts.one_per_line || opts.long_format {
            libsyscall::write(1, b"\n");
        } else {
            libsyscall::write(1, b"  ");
        }

        // TEAM_193: Collect subdirectories for -R
        if opts.recursive && file_type == FileType::Directory && name != "." && name != ".." {
            subdirs.push(join_path(path, name));
        }
    }

    if !opts.one_per_line && !opts.long_format {
        libsyscall::write(1, b"\n");
    }

    // TEAM_193: Recursive listing
    let mut success = true;
    for subdir in subdirs {
        println!();
        println!("{}:", subdir);
        if !list_dir(&subdir, opts) {
            success = false;
        }
    }

    success
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
            } else if arg == "--all" {
                opts.all = true;
            } else if arg == "--almost-all" {
                opts.almost_all = true;
            } else if arg == "--classify" {
                opts.classify = true;
            } else if arg == "--human-readable" {
                opts.human_readable = true;
            } else if arg == "--recursive" {
                opts.recursive = true;
            } else if arg == "--color" || arg.starts_with("--color=") {
                opts.color = true;
            } else if arg.starts_with("--") {
                println!("ls: unrecognized option: {}", arg);
                return 2;
            } else if arg.starts_with('-') && arg.len() > 1 {
                for c in arg.chars().skip(1) {
                    match c {
                        'a' => opts.all = true,
                        'A' => opts.almost_all = true,
                        'F' => opts.classify = true,
                        'h' => opts.human_readable = true,
                        'l' => opts.long_format = true,
                        'R' => opts.recursive = true,
                        '1' => opts.one_per_line = true,
                        _ => {
                            println!("ls: invalid option -- '{}'", c);
                            return 2;
                        }
                    }
                }
            } else {
                files.push(arg);
            }
        }
    }

    let mut success = true;
    if files.is_empty() {
        if !list_dir(".", &opts) {
            success = false;
        }
    } else {
        let multi = files.len() > 1;
        for (i, file) in files.iter().enumerate() {
            if multi {
                if i > 0 {
                    println!();
                }
                println!("{}:", file);
            }
            if !list_dir(file, &opts) {
                success = false;
            }
        }
    }

    if success {
        0
    } else {
        1
    }
}
