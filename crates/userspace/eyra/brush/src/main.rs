// TEAM_394: Brush shell wrapper for LevitateOS
// TEAM_404: Rewritten to use raw byte-by-byte I/O like lsh
//
// Uses Eyra std support but with raw fd reads for compatibility
// with LevitateOS kernel's TTY implementation.

use std::io::{Read, Write};
use std::os::fd::FromRawFd;

fn main() {
    println!();
    println!("LevitateOS Shell v0.2");
    println!("Type 'help' for commands.");
    println!();
    
    shell_loop();
}

fn shell_loop() {
    let mut buf = [0u8; 256];
    
    // Get raw stdin/stdout
    let mut stdin = unsafe { std::fs::File::from_raw_fd(0) };
    let mut stdout = unsafe { std::fs::File::from_raw_fd(1) };
    
    loop {
        // Print prompt
        let _ = stdout.write_all(b"# ");
        let _ = stdout.flush();
        
        // Read line character by character (like lsh does)
        let mut line_len = 0;
        loop {
            let mut c_buf = [0u8; 1];
            match stdin.read(&mut c_buf) {
                Ok(0) => {
                    // EOF
                    let _ = stdout.write_all(b"\nGoodbye!\n");
                    std::process::exit(0);
                }
                Ok(_) => {
                    let b = c_buf[0];
                    
                    if b == b'\n' || b == b'\r' {
                        // Newline - execute command
                        let _ = stdout.write_all(b"\n");
                        if line_len > 0 {
                            execute(&buf[..line_len], &mut stdout);
                        }
                        break;
                    } else if b == 0x08 || b == 0x7f {
                        // Backspace
                        if line_len > 0 {
                            line_len -= 1;
                            let _ = stdout.write_all(b"\x08 \x08");
                        }
                    } else if line_len < buf.len() {
                        // Normal character
                        buf[line_len] = b;
                        line_len += 1;
                        let _ = stdout.write_all(&c_buf[..1]);
                    }
                }
                Err(_) => {
                    let _ = stdout.write_all(b"\nRead error\n");
                    std::process::exit(1);
                }
            }
        }
    }
}

fn execute(cmd: &[u8], stdout: &mut std::fs::File) {
    let cmd = trim(cmd);
    if cmd.is_empty() {
        return;
    }
    
    // Convert to str for easier handling
    let cmd_str = match std::str::from_utf8(cmd) {
        Ok(s) => s,
        Err(_) => {
            let _ = stdout.write_all(b"Invalid UTF-8\n");
            return;
        }
    };
    
    match cmd_str {
        "exit" | "quit" => {
            let _ = stdout.write_all(b"Goodbye!\n");
            std::process::exit(0);
        }
        "help" => {
            let _ = stdout.write_all(b"LevitateOS Shell v0.2\n");
            let _ = stdout.write_all(b"Commands:\n");
            let _ = stdout.write_all(b"  echo <text>  - Print text\n");
            let _ = stdout.write_all(b"  help         - Show this help\n");
            let _ = stdout.write_all(b"  clear        - Clear screen\n");
            let _ = stdout.write_all(b"  exit         - Exit shell\n");
        }
        "clear" => {
            let _ = stdout.write_all(b"\x1b[2J\x1b[H");
        }
        _ if cmd_str.starts_with("echo ") => {
            let _ = stdout.write_all(cmd_str[5..].as_bytes());
            let _ = stdout.write_all(b"\n");
        }
        "echo" => {
            let _ = stdout.write_all(b"\n");
        }
        _ => {
            // External command - spawn process using libsyscall
            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            if parts.is_empty() {
                return;
            }
            
            let prog = parts[0];
            let path = if prog.starts_with('/') {
                prog.to_string()
            } else {
                format!("/{}", prog)
            };
            
            // TEAM_404: Use libsyscall for process spawning
            let pid = libsyscall::spawn_args(&path, &parts);
            if pid >= 0 {
                // Set child as foreground and wait
                libsyscall::set_foreground(pid as usize);
                let mut status: i32 = 0;
                libsyscall::waitpid(pid as i32, Some(&mut status));
                // Restore shell as foreground
                libsyscall::set_foreground(libsyscall::getpid() as usize);
            } else {
                let _ = stdout.write_all(b"Unknown: ");
                let _ = stdout.write_all(path.as_bytes());
                let _ = stdout.write_all(b"\n");
            }
        }
    }
}

fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = s.len();
    while start < end && matches!(s[start], b' ' | b'\t' | b'\n' | b'\r') {
        start += 1;
    }
    while end > start && matches!(s[end - 1], b' ' | b'\t' | b'\n' | b'\r') {
        end -= 1;
    }
    &s[start..end]
}
