// TEAM_394: Brush shell wrapper for LevitateOS
//
// This binary wraps brush-shell to run on LevitateOS via Eyra std support.
// Brush provides POSIX/Bash-compatible shell functionality including:
// - Script execution (.sh files)
// - Variables, loops, conditionals
// - Tab completion and history (when reedline feature enabled)

// Required for Eyra std replacement
extern crate eyra;

fn main() {
    // TEAM_394: For now, just print a message to verify the binary loads
    // Full brush integration requires tokio runtime which needs epoll
    println!("Brush shell starting on LevitateOS...");
    
    // TODO: Initialize brush shell once tokio works
    // The full implementation would be:
    // brush_shell::run().unwrap_or_else(|e| {
    //     eprintln!("brush: {}", e);
    //     std::process::exit(1);
    // });
    
    // For now, run a simple REPL loop
    simple_repl();
}

/// TEAM_394: Simple REPL for initial testing
/// This will be replaced by brush_shell::run() once tokio works
fn simple_repl() {
    use std::io::{self, BufRead, Write};
    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    loop {
        // Print prompt
        print!("brush$ ");
        let _ = stdout.flush();
        
        // Read line
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                // EOF
                println!();
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("brush: read error: {}", e);
                break;
            }
        }
        
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Handle builtins
        match line {
            "exit" | "quit" => break,
            "help" => {
                println!("Brush shell (minimal mode)");
                println!("Builtins: exit, help, pwd, cd, echo");
                println!("Full brush features pending tokio integration");
            }
            "pwd" => {
                match std::env::current_dir() {
                    Ok(path) => println!("{}", path.display()),
                    Err(e) => eprintln!("pwd: {}", e),
                }
            }
            _ if line.starts_with("cd ") => {
                let path = &line[3..];
                if let Err(e) = std::env::set_current_dir(path) {
                    eprintln!("cd: {}: {}", path, e);
                }
            }
            _ if line.starts_with("echo ") => {
                println!("{}", &line[5..]);
            }
            _ => {
                // Try to execute as external command
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }
                
                let cmd = parts[0];
                let args = &parts[1..];
                
                match std::process::Command::new(cmd).args(args).status() {
                    Ok(status) => {
                        if !status.success() {
                            if let Some(code) = status.code() {
                                eprintln!("brush: {} exited with code {}", cmd, code);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("brush: {}: {}", cmd, e);
                    }
                }
            }
        }
    }
}
