//! TEAM_250: Test suite for Core Utils
//!
//! Verifies: mkdir, ls, touch, rm, cat, cp, mv, etc.
//! Requires FD inheritance for output capturing.

#![no_std]
#![no_main]

extern crate alloc;
extern crate ulib;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use libsyscall::{
    close, dup2, exit, pipe2, println, read, shutdown, shutdown_flags, spawn_args, waitpid, write,
    O_CLOEXEC,
};

fn eprintln(msg: &str) {
    let _ = libsyscall::write(2, msg.as_bytes());
    let _ = libsyscall::write(2, b"\n");
}

/// Helper to run a command and return (exit_code, stdout)
fn run_capture(bin: &str, args: &[&str]) -> (i32, String) {
    let mut pipe_fds = [0i32; 2];
    if pipe2(&mut pipe_fds, 0) < 0 {
        eprintln("PIPE_FAIL");
        return (-1, String::from("PIPE_FAIL"));
    }
    let read_fd = pipe_fds[0] as usize;
    let write_fd = pipe_fds[1] as usize;

    // 0. Prepend binary name to args for argv[0]
    let mut full_args = alloc::vec::Vec::new();
    full_args.push(bin);
    full_args.extend_from_slice(args);
    let full_args_slice = full_args.as_slice();

    let pid = unsafe {
        println!("[TRACE] run_capture: starting spawn sequence");
        // We can't use fork(), so we rely on spawn_args inheriting FDs.
        // BUT spawn_args runs a NEW binary.
        // We need the CHILD to have stdout=write_fd.
        // Since we don't have fork(), we can't dup2 in the child before exec.
        //
        // WAIT. If we don't have fork(), how do we redirect stdout for the child?
        // `spawn` in LevitateOS just loads a binary and runs it.
        // It inherits FDs.
        // So we need to set OUR stdout to the pipe write end, then spawn, then restore our stdout.

        // 1. Save current stdout
        let saved_stdout = 10; // Must be < MAX_FDS (64)
        if libsyscall::dup2(1, saved_stdout) < 0 {
            let _ = close(read_fd);
            let _ = close(write_fd);
            eprintln("DUP_FAIL_SAVE");
            return (-1, String::from("DUP_FAIL_SAVE"));
        }

        // 2. Redirect stdout to pipe write
        if libsyscall::dup2(write_fd, 1) < 0 {
            let _ = close(read_fd);
            let _ = close(write_fd);
            let _ = close(saved_stdout);
            eprintln("DUP_FAIL_REDIRECT");
            return (-1, String::from("DUP_FAIL_REDIRECT"));
        }

        // 3. Spawn child (inherits stdout=pipe_write)
        let child_pid = spawn_args(bin, full_args_slice);

        // Restore stdout immediately after spawn
        let _ = libsyscall::dup2(saved_stdout, 1);
        let _ = close(saved_stdout);
        let _ = close(write_fd); // Close write end so read doesn't block forever

        if child_pid < 0 {
            eprintln("SPAWN ERROR");
            return (-1, String::from("SPAWN_FAIL"));
        }

        println!("[TRACE] run_capture: spawned pid {}", child_pid);
        child_pid
    };

    if pid < 0 {
        let _ = close(read_fd);
        return (-1, String::from("SPAWN_FAIL_CHECK"));
    }

    // Read output from pipe
    let mut output = Vec::new();
    let mut buf = [0u8; 1024];
    loop {
        let n = read(read_fd, &mut buf);
        if n > 0 {
            output.extend_from_slice(&buf[..n as usize]);
        } else {
            break;
        }
    }
    let _ = close(read_fd);

    // Wait for child
    let mut status = 0;
    let wait_ret = waitpid(pid as i32, Some(&mut status));
    if wait_ret < 0 {
        // Only print if waitpid failed unexpectedly
        // return (wait_ret as i32, String::from("WAITPID_FAIL"));
        // Actually, let's return -1 status and the error string
        return (-1, format!("WAITPID_FAIL: pid={} ret={}", pid, wait_ret));
    }

    // Debug status
    // println!("DEBUG: waitpid({}) -> status {}", pid, status);

    let output_str = String::from_utf8(output).unwrap_or(String::from("<invalid utf8>"));
    println!(
        "[TRACE] run_capture: finished. status={}, output len={}",
        status,
        output_str.len()
    );
    (status, output_str)
}

fn test_mkdir() -> bool {
    println!("[TEST] mkdir: Basic creation...");
    let (code, out) = run_capture("mkdir", &["/tmp/test_dir"]);
    if code != 0 {
        println!("FAIL: mkdir exit code {}", code);
        println!("OUTPUT: [{}]", out);
        return false;
    }

    // Verify it exists
    let fd = libsyscall::openat("/tmp/test_dir", 0);
    if fd < 0 {
        println!("FAIL: directory not created");
        return false;
    }
    libsyscall::close(fd as usize);

    println!("[TEST] mkdir: -p nested...");
    let (code, out) = run_capture("mkdir", &["-v", "-p", "/tmp/a/b/c"]);
    if code != 0 {
        println!("FAIL: mkdir -p exit code {}", code);
        println!("OUTPUT: {}", out);
        return false;
    }

    let fd = libsyscall::openat("/tmp/a/b/c", 0);
    if fd < 0 {
        println!("FAIL: nested directory not created");
        return false;
    }
    libsyscall::close(fd as usize);

    true
}

fn test_ls() -> bool {
    println!("[TEST] ls: Listing...");
    // Setup
    let _ = run_capture("mkdir", &["-p", "/tmp/ls_test/sub"]);
    let _ = run_capture("touch", &["/tmp/ls_test/file1"]);

    let (code, output) = run_capture("ls", &["/tmp/ls_test"]);
    if code != 0 {
        println!("FAIL: ls exit code {}", code);
        println!("OUTPUT: {}", output);
        return false;
    }

    if !output.contains("sub") || !output.contains("file1") {
        println!("FAIL: Output missing files. Got:\n{}", output);
        return false;
    }
    true
}

fn test_cat() -> bool {
    println!("[TEST] cat: File content...");
    // Setup - write via shell echo? No, we don't have echo binary exposed to spawn easily from here unless 'echo' is binary.
    // 'echo' is shell builtin.
    // We can use 'touch' to make empty file, but need content.
    // We can write content using syscalls directly from this test binary.

    let path = "/tmp/cat_test.txt";
    let fd = libsyscall::openat(path, libsyscall::O_CREAT | libsyscall::O_WRONLY);
    if fd < 0 {
        println!("FAIL: match setup failed");
        return false;
    }
    let content = "Hello World";
    libsyscall::write(fd as usize, content.as_bytes());
    libsyscall::close(fd as usize);

    let (code, output) = run_capture("cat", &[path]);
    if code != 0 {
        println!("FAIL: cat exit code {}", code);
        println!("OUTPUT: {}", output);
        return false;
    }

    if output != content {
        println!(
            "FAIL: Content mismatch. Expected '{}', got '{}'",
            content, output
        );
        return false;
    }
    true
}

fn test_touch() -> bool {
    println!("[TEST] touch: Create file...");
    let path = "/tmp/touch_test";

    // Ensure clean
    let _ = run_capture("rm", &[path]);

    let (code, _) = run_capture("touch", &[path]);
    if code != 0 {
        return false;
    }

    let fd = libsyscall::openat(path, 0);
    if fd < 0 {
        println!("FAIL: touch did not create file");
        return false;
    }
    libsyscall::close(fd as usize);
    true
}

fn test_rm() -> bool {
    println!("[TEST] rm: Remove file...");
    let path = "/tmp/rm_test";
    let _ = run_capture("touch", &[path]); // Setup

    let (code, _) = run_capture("rm", &[path]);
    if code != 0 {
        println!("FAIL: rm exit code {}", code);
        return false;
    }

    let fd = libsyscall::openat(path, 0);
    if fd >= 0 {
        libsyscall::close(fd as usize);
        println!("FAIL: File still exists");
        return false;
    }
    true
}

fn test_rmdir() -> bool {
    println!("[TEST] rmdir: Remove directory...");
    let path = "/tmp/rmdir_test";
    let _ = run_capture("mkdir", &[path]); // Setup

    let (code, _) = run_capture("rmdir", &[path]);
    if code != 0 {
        println!("FAIL: rmdir exit code {}", code);
        return false;
    }

    let fd = libsyscall::openat(path, 0);
    if fd >= 0 {
        libsyscall::close(fd as usize);
        println!("FAIL: Directory still exists");
        return false;
    }
    true
}

// Basic cp test
fn test_cp() -> bool {
    println!("[TEST] cp: Copy file...");
    let src = "/tmp/cp_src";
    let dst = "/tmp/cp_dst";

    // Setup src
    let fd = libsyscall::openat(src, libsyscall::O_CREAT | libsyscall::O_WRONLY);
    if fd < 0 {
        return false;
    }
    libsyscall::write(fd as usize, b"COPYDATA");
    libsyscall::close(fd as usize);

    // Run cp
    let (code, _) = run_capture("cp", &[src, dst]);
    if code != 0 {
        println!("FAIL: cp exit code {}", code);
        return false;
    }

    // Check dst content
    let (c, out) = run_capture("cat", &[dst]);
    if c != 0 || out != "COPYDATA" {
        println!("FAIL: Content mismatch. Got '{}'", out);
        return false;
    }
    true
}

// Basic mv test
fn test_mv() -> bool {
    println!("[TEST] mv: Move file...");
    let src = "/tmp/mv_src";
    let dst = "/tmp/mv_dst";

    // Setup src
    let fd = libsyscall::openat(src, libsyscall::O_CREAT | libsyscall::O_WRONLY);
    if fd < 0 {
        return false;
    }
    libsyscall::write(fd as usize, b"MOVEDATA");
    libsyscall::close(fd as usize);

    // Run mv
    let (code, _) = run_capture("mv", &[src, dst]);
    if code != 0 {
        println!("FAIL: mv exit code {}", code);
        return false;
    }

    // Check src gone
    let fd_src = libsyscall::openat(src, 0);
    if fd_src >= 0 {
        libsyscall::close(fd_src as usize);
        println!("FAIL: Source still exists");
        return false;
    }

    // Check dst content
    let (c, out) = run_capture("cat", &[dst]);
    if c != 0 || out != "MOVEDATA" {
        println!("FAIL: Content mismatch. Got '{}'", out);
        return false;
    }
    true
}

#[no_mangle]
pub fn main() -> i32 {
    println!("[SUITE] Core Utils Test Suite");

    let tests = [
        ("mkdir", test_mkdir as fn() -> bool),
        ("ls", test_ls),
        ("cat", test_cat),
        ("touch", test_touch),
        ("rm", test_rm),
        ("rmdir", test_rmdir),
        ("cp", test_cp),
        ("mv", test_mv),
    ];

    let mut failures = 0;
    for (name, func) in tests.iter() {
        println!("--------------------------------");
        if func() {
            println!("[PASS] {}", name);
        } else {
            println!("[FAIL] {}", name);
            failures += 1;
        }
    }
    println!("--------------------------------");

    if failures > 0 {
        println!("FAILED: {} tests failed", failures);
        1
    } else {
        println!("PASSED: All tests passed");
        0
    }
}
