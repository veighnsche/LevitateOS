//! LevitateOS Shell - A minimal no_std shell
//!
//! TEAM_435: Standalone no_std shell for LevitateOS.
//! No fallbacks, no compromises. Fail fast, fail loud.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libsyscall::{common_panic_handler, exit, print, println, read, spawn_args, waitpid};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

const VERSION: &str = "0.1.0";
const MAX_LINE: usize = 256;
const MAX_ARGS: usize = 16;
const MAX_ARG_LEN: usize = 64;

/// Fixed-size argument storage
struct Args {
    /// Storage for argument strings
    data: [[u8; MAX_ARG_LEN]; MAX_ARGS],
    /// Length of each argument
    lens: [usize; MAX_ARGS],
    /// Number of arguments
    count: usize,
}

impl Args {
    fn new() -> Self {
        Self {
            data: [[0u8; MAX_ARG_LEN]; MAX_ARGS],
            lens: [0; MAX_ARGS],
            count: 0,
        }
    }

    fn clear(&mut self) {
        self.count = 0;
    }

    fn push(&mut self, arg: &[u8]) -> bool {
        if self.count >= MAX_ARGS {
            return false;
        }
        let len = arg.len().min(MAX_ARG_LEN - 1);
        self.data[self.count][..len].copy_from_slice(&arg[..len]);
        self.data[self.count][len] = 0;
        self.lens[self.count] = len;
        self.count += 1;
        true
    }

    fn get(&self, i: usize) -> Option<&str> {
        if i >= self.count {
            return None;
        }
        // SAFETY: We only store valid UTF-8 from user input
        core::str::from_utf8(&self.data[i][..self.lens[i]]).ok()
    }

    fn len(&self) -> usize {
        self.count
    }
}

/// Read a line from stdin into buffer, returns length
fn read_line(buf: &mut [u8; MAX_LINE]) -> usize {
    let mut pos = 0;

    loop {
        let mut byte = [0u8; 1];
        let n = read(0, &mut byte);

        if n <= 0 {
            break;
        }

        let ch = byte[0];

        match ch {
            // Enter - done
            b'\n' | b'\r' => {
                println!("");
                break;
            }
            // Backspace
            0x7f | 0x08 => {
                if pos > 0 {
                    pos -= 1;
                    print!("\x08 \x08");
                }
            }
            // Ctrl+C
            0x03 => {
                println!("^C");
                pos = 0;
                break;
            }
            // Ctrl+D at start of line = exit
            0x04 => {
                if pos == 0 {
                    println!("exit");
                    exit(0);
                }
            }
            // Regular character
            _ => {
                if pos < MAX_LINE - 1 && ch >= 0x20 && ch < 0x7f {
                    buf[pos] = ch;
                    pos += 1;
                    print!("{}", ch as char);
                }
            }
        }
    }

    pos
}

/// Parse command line into arguments
fn parse_line(line: &[u8], len: usize, args: &mut Args) {
    args.clear();

    let mut word_start = 0;
    let mut in_word = false;

    for i in 0..=len {
        let is_space = i == len || line[i] == b' ' || line[i] == b'\t';

        if in_word && is_space {
            args.push(&line[word_start..i]);
            in_word = false;
        } else if !in_word && !is_space {
            word_start = i;
            in_word = true;
        }
    }
}

/// Built-in: help
fn builtin_help() {
    println!("LevitateOS Shell v{}", VERSION);
    println!("");
    println!("Built-in commands:");
    println!("  help     - Show this help");
    println!("  exit     - Exit shell");
    println!("  cd DIR   - Change directory");
    println!("  pwd      - Print working directory");
    println!("");
    println!("External commands:");
    println!("  /path/cmd args  - Run program at path");
    println!("  cmd args        - Run /cmd");
    println!("");
    println!("Coreutils: cat echo head mkdir pwd rm tail touch");
}

/// Built-in: cd
fn builtin_cd(args: &Args) {
    if args.len() < 2 {
        println!("cd: missing directory");
        return;
    }
    if let Some(dir) = args.get(1) {
        let result = libsyscall::chdir(dir);
        if result < 0 {
            println!("cd: {}: error {}", dir, -result);
        }
    }
}

/// Built-in: pwd
fn builtin_pwd() {
    let mut buf = [0u8; 256];
    let result = libsyscall::getcwd(&mut buf);
    if result >= 0 {
        if let Ok(s) = core::str::from_utf8(&buf[..result as usize]) {
            println!("{}", s);
        }
    } else {
        println!("pwd: error {}", -result);
    }
}

/// Execute external command
fn exec_external(args: &Args) {
    if args.len() == 0 {
        return;
    }

    let cmd = match args.get(0) {
        Some(c) => c,
        None => return,
    };

    // Build path - if doesn't start with /, prepend /
    let mut path_buf = [0u8; 128];
    let path_len;

    if cmd.starts_with('/') {
        let len = cmd.len().min(127);
        path_buf[..len].copy_from_slice(&cmd.as_bytes()[..len]);
        path_len = len;
    } else {
        path_buf[0] = b'/';
        let len = cmd.len().min(126);
        path_buf[1..1 + len].copy_from_slice(&cmd.as_bytes()[..len]);
        path_len = 1 + len;
    }
    path_buf[path_len] = 0;

    let path = match core::str::from_utf8(&path_buf[..path_len]) {
        Ok(p) => p,
        Err(_) => {
            println!("{}: invalid path", cmd);
            return;
        }
    };

    // Build argv array for spawn_args
    // We need to create &str references that spawn_args expects
    let mut argv_storage: [&str; MAX_ARGS] = [""; MAX_ARGS];
    for i in 0..args.len() {
        if let Some(s) = args.get(i) {
            argv_storage[i] = s;
        }
    }
    let argv = &argv_storage[..args.len()];

    // Spawn the process
    let pid = spawn_args(path, argv);

    if pid < 0 {
        println!("{}: not found (error {})", cmd, -pid);
        return;
    }

    // Wait for it to complete
    let mut status: i32 = 0;
    let wait_result = waitpid(pid as i32, Some(&mut status));

    if wait_result < 0 {
        println!("{}: wait failed (error {})", cmd, -wait_result);
    } else if status != 0 {
        println!("[exit {}]", status);
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("LevitateOS Shell v{}", VERSION);
    println!("Type 'help' for commands.");
    println!("");

    let mut line_buf = [0u8; MAX_LINE];
    let mut args = Args::new();

    loop {
        print!("# ");

        let len = read_line(&mut line_buf);
        if len == 0 {
            continue;
        }

        parse_line(&line_buf, len, &mut args);
        if args.len() == 0 {
            continue;
        }

        let cmd = match args.get(0) {
            Some(c) => c,
            None => continue,
        };

        // Built-in commands
        match cmd {
            "help" | "?" => builtin_help(),
            "exit" | "quit" => {
                println!("Goodbye.");
                exit(0);
            }
            "cd" => builtin_cd(&args),
            "pwd" => builtin_pwd(),
            _ => exec_external(&args),
        }
    }
}
