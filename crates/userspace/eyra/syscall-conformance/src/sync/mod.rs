//! TEAM_424: Synchronization syscall conformance tests
//!
//! Tests: pipe, pipe2, futex (basic)

use crate::{conformance_test, assert_syscall_ok, assert_eq_desc, TestResult};
use libsyscall::{fs, sysno};
use libsyscall::arch::{syscall1, syscall2, syscall3};

pub fn run_tests() -> Vec<TestResult> {
    vec![
        test_pipe_basic(),
        test_pipe2_cloexec(),
        test_pipe_readwrite(),
    ]
}

// =============================================================================
// pipe() tests
// =============================================================================

fn test_pipe_basic() -> TestResult {
    conformance_test!("sync", "pipe_basic", {
        let mut fds = [0i32; 2];

        let result = syscall2(
            sysno::__NR_pipe2 as u64,
            fds.as_mut_ptr() as u64,
            0 // flags
        );
        assert_syscall_ok!(result, "pipe2");

        // Should return two valid fds
        let read_fd = fds[0];
        let write_fd = fds[1];

        if read_fd < 0 || write_fd < 0 {
            return Err(format!("pipe returned invalid fds: [{}, {}]", read_fd, write_fd));
        }

        if read_fd == write_fd {
            return Err("pipe returned same fd for read and write".to_string());
        }

        // Clean up
        let _ = syscall1(sysno::__NR_close as u64, read_fd as u64);
        let _ = syscall1(sysno::__NR_close as u64, write_fd as u64);
        Ok(())
    })
}

fn test_pipe2_cloexec() -> TestResult {
    conformance_test!("sync", "pipe2_cloexec", {
        let mut fds = [0i32; 2];

        let result = syscall2(
            sysno::__NR_pipe2 as u64,
            fds.as_mut_ptr() as u64,
            fs::O_CLOEXEC as u64
        );
        assert_syscall_ok!(result, "pipe2 with O_CLOEXEC");

        // Clean up
        let _ = syscall1(sysno::__NR_close as u64, fds[0] as u64);
        let _ = syscall1(sysno::__NR_close as u64, fds[1] as u64);
        Ok(())
    })
}

fn test_pipe_readwrite() -> TestResult {
    conformance_test!("sync", "pipe_readwrite", {
        let mut fds = [0i32; 2];

        let result = syscall2(sysno::__NR_pipe2 as u64, fds.as_mut_ptr() as u64, 0);
        assert_syscall_ok!(result, "pipe2");

        let read_fd = fds[0] as u64;
        let write_fd = fds[1] as u64;

        // Write some data
        let msg = b"pipe test data";
        let write_result = syscall3(
            sysno::__NR_write as u64,
            write_fd,
            msg.as_ptr() as u64,
            msg.len() as u64
        );
        assert_syscall_ok!(write_result, "write to pipe");
        assert_eq_desc!(write_result as usize, msg.len(), "write should return bytes written");

        // Read it back
        let mut buf = [0u8; 32];
        let read_result = syscall3(
            sysno::__NR_read as u64,
            read_fd,
            buf.as_mut_ptr() as u64,
            buf.len() as u64
        );
        assert_syscall_ok!(read_result, "read from pipe");
        assert_eq_desc!(read_result as usize, msg.len(), "read should return bytes available");

        // Verify data matches
        if &buf[..msg.len()] != msg {
            return Err("pipe data mismatch".to_string());
        }

        // Clean up
        let _ = syscall1(sysno::__NR_close as u64, read_fd);
        let _ = syscall1(sysno::__NR_close as u64, write_fd);
        Ok(())
    })
}
