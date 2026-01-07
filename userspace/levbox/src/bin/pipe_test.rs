#![no_std]
#![no_main]

//! TEAM_239: pipe_test - Verify pipe2/dup/dup3 syscalls work correctly.

extern crate ulib;
use libsyscall::println;

#[no_mangle]
pub fn main() -> i32 {
    println!("[pipe_test] Starting pipe/dup syscall tests...");

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Basic pipe creation
    println!("[pipe_test] Test 1: Basic pipe2");
    if test_basic_pipe() {
        println!("[pipe_test] Test 1: PASS");
        passed += 1;
    } else {
        println!("[pipe_test] Test 1: FAIL");
        failed += 1;
    }

    // Test 2: Pipe read/write
    println!("[pipe_test] Test 2: Pipe read/write");
    if test_pipe_readwrite() {
        println!("[pipe_test] Test 2: PASS");
        passed += 1;
    } else {
        println!("[pipe_test] Test 2: FAIL");
        failed += 1;
    }

    // Test 3: Pipe EOF detection
    println!("[pipe_test] Test 3: Pipe EOF");
    if test_pipe_eof() {
        println!("[pipe_test] Test 3: PASS");
        passed += 1;
    } else {
        println!("[pipe_test] Test 3: FAIL");
        failed += 1;
    }

    // Test 4: dup syscall
    println!("[pipe_test] Test 4: dup");
    if test_dup() {
        println!("[pipe_test] Test 4: PASS");
        passed += 1;
    } else {
        println!("[pipe_test] Test 4: FAIL");
        failed += 1;
    }

    // Test 5: dup3 syscall
    println!("[pipe_test] Test 5: dup3");
    if test_dup3() {
        println!("[pipe_test] Test 5: PASS");
        passed += 1;
    } else {
        println!("[pipe_test] Test 5: FAIL");
        failed += 1;
    }

    println!("[pipe_test] Results: {} passed, {} failed", passed, failed);

    if failed == 0 {
        println!("[pipe_test] All tests passed!");
        0
    } else {
        println!("[pipe_test] Some tests failed!");
        1
    }
}

/// Test that pipe2 creates a valid pipe pair
fn test_basic_pipe() -> bool {
    let mut fds = [0i32; 2];

    let res = libsyscall::pipe2(&mut fds, 0);
    if res < 0 {
        println!("[pipe_test]   pipe2 failed with error: {}", res);
        return false;
    }

    // fds should be valid (positive or zero for read, write)
    if fds[0] < 0 || fds[1] < 0 {
        println!(
            "[pipe_test]   invalid fds: read={}, write={}",
            fds[0], fds[1]
        );
        return false;
    }

    // fds should be different
    if fds[0] == fds[1] {
        println!("[pipe_test]   read and write fds are the same: {}", fds[0]);
        return false;
    }

    // Clean up
    libsyscall::close(fds[0] as usize);
    libsyscall::close(fds[1] as usize);

    true
}

/// Test writing to and reading from a pipe
fn test_pipe_readwrite() -> bool {
    let mut fds = [0i32; 2];

    if libsyscall::pipe2(&mut fds, 0) < 0 {
        println!("[pipe_test]   pipe2 failed");
        return false;
    }

    let read_fd = fds[0] as usize;
    let write_fd = fds[1] as usize;

    // Write data to pipe
    let msg = b"Hello, pipe!";
    let written = libsyscall::write(write_fd, msg);

    if written != msg.len() as isize {
        println!(
            "[pipe_test]   write returned {}, expected {}",
            written,
            msg.len()
        );
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        return false;
    }

    // Read data from pipe
    let mut buf = [0u8; 32];
    let read = libsyscall::read(read_fd, &mut buf);

    if read != msg.len() as isize {
        println!(
            "[pipe_test]   read returned {}, expected {}",
            read,
            msg.len()
        );
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        return false;
    }

    // Verify content
    if &buf[..msg.len()] != msg {
        println!("[pipe_test]   data mismatch");
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        return false;
    }

    libsyscall::close(read_fd);
    libsyscall::close(write_fd);
    true
}

/// Test that closing write end causes read to return 0 (EOF)
fn test_pipe_eof() -> bool {
    let mut fds = [0i32; 2];

    if libsyscall::pipe2(&mut fds, 0) < 0 {
        println!("[pipe_test]   pipe2 failed");
        return false;
    }

    let read_fd = fds[0] as usize;
    let write_fd = fds[1] as usize;

    // Write some data first
    let msg = b"data";
    libsyscall::write(write_fd, msg);

    // Read the data
    let mut buf = [0u8; 32];
    let read = libsyscall::read(read_fd, &mut buf);
    if read != 4 {
        println!("[pipe_test]   first read returned {}, expected 4", read);
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        return false;
    }

    // Close write end
    libsyscall::close(write_fd);

    // Read should return 0 (EOF) now
    let read2 = libsyscall::read(read_fd, &mut buf);
    if read2 != 0 {
        println!("[pipe_test]   EOF read returned {}, expected 0", read2);
        libsyscall::close(read_fd);
        return false;
    }

    libsyscall::close(read_fd);
    true
}

/// Test dup creates a copy of the fd
fn test_dup() -> bool {
    let mut fds = [0i32; 2];

    if libsyscall::pipe2(&mut fds, 0) < 0 {
        println!("[pipe_test]   pipe2 failed");
        return false;
    }

    let read_fd = fds[0] as usize;
    let write_fd = fds[1] as usize;

    // Dup the write end
    let dup_fd = libsyscall::dup(write_fd);
    if dup_fd < 0 {
        println!("[pipe_test]   dup failed with error: {}", dup_fd);
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        return false;
    }

    // Write via the duped fd
    let msg = b"via dup";
    let written = libsyscall::write(dup_fd as usize, msg);
    if written != msg.len() as isize {
        println!("[pipe_test]   write via dup failed");
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        libsyscall::close(dup_fd as usize);
        return false;
    }

    // Read from read end
    let mut buf = [0u8; 32];
    let read = libsyscall::read(read_fd, &mut buf);
    if read != msg.len() as isize {
        println!("[pipe_test]   read after dup write failed");
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        libsyscall::close(dup_fd as usize);
        return false;
    }

    // Verify content
    if &buf[..msg.len()] != msg {
        println!("[pipe_test]   data mismatch after dup");
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        libsyscall::close(dup_fd as usize);
        return false;
    }

    libsyscall::close(read_fd);
    libsyscall::close(write_fd);
    libsyscall::close(dup_fd as usize);
    true
}

/// Test dup3 duplicates to a specific fd number
fn test_dup3() -> bool {
    let mut fds = [0i32; 2];

    if libsyscall::pipe2(&mut fds, 0) < 0 {
        println!("[pipe_test]   pipe2 failed");
        return false;
    }

    let read_fd = fds[0] as usize;
    let write_fd = fds[1] as usize;

    // Pick a specific target fd (high number to avoid conflicts)
    let target_fd: usize = 50;

    // Dup write_fd to target_fd
    let res = libsyscall::dup3(write_fd, target_fd, 0);
    if res < 0 {
        println!("[pipe_test]   dup3 failed with error: {}", res);
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        return false;
    }

    // Verify the returned fd is target_fd
    if res as usize != target_fd {
        println!(
            "[pipe_test]   dup3 returned {}, expected {}",
            res, target_fd
        );
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        libsyscall::close(res as usize);
        return false;
    }

    // Write via target_fd
    let msg = b"via dup3";
    let written = libsyscall::write(target_fd, msg);
    if written != msg.len() as isize {
        println!("[pipe_test]   write via dup3 fd failed");
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        libsyscall::close(target_fd);
        return false;
    }

    // Read and verify
    let mut buf = [0u8; 32];
    let read = libsyscall::read(read_fd, &mut buf);
    if read != msg.len() as isize || &buf[..msg.len()] != msg {
        println!("[pipe_test]   data verification failed after dup3");
        libsyscall::close(read_fd);
        libsyscall::close(write_fd);
        libsyscall::close(target_fd);
        return false;
    }

    libsyscall::close(read_fd);
    libsyscall::close(write_fd);
    libsyscall::close(target_fd);
    true
}
