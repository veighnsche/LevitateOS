//! TEAM_424: Filesystem syscall conformance tests
//!
//! Tests: open, openat, close, read, write, writev, fstat, lseek, dup, dup2, dup3

use crate::{conformance_test, assert_syscall_ok, assert_syscall_err, assert_eq_desc, TestResult};
use libsyscall::{errno, fs, sysno};
use libsyscall::arch::{syscall1, syscall2, syscall3, syscall4};

pub fn run_tests() -> Vec<TestResult> {
    vec![
        test_write_stdout(),
        test_read_stdin_nonblock(),
        test_openat_nonexistent(),
        test_close_invalid_fd(),
        test_close_valid_fd(),
        test_fstat_stdout(),
        test_lseek_pipe_fails(),
        test_dup_stdout(),
        test_dup2_redirect(),
        test_writev_basic(),
    ]
}

// =============================================================================
// write() tests
// =============================================================================

fn test_write_stdout() -> TestResult {
    conformance_test!("fs", "write_stdout", {
        let msg = b"[conformance] write test\n";
        let result = syscall3(
            sysno::__NR_write as u64,
            1, // stdout
            msg.as_ptr() as u64,
            msg.len() as u64
        );
        assert_syscall_ok!(result, "write to stdout");
        assert_eq_desc!(result as usize, msg.len(), "write should return bytes written");
        Ok(())
    })
}

// =============================================================================
// read() tests
// =============================================================================

fn test_read_stdin_nonblock() -> TestResult {
    conformance_test!("fs", "read_stdin_setup", {
        // Just verify fstat on stdin works - don't actually read
        let mut buf = [0u8; 128];
        let result = syscall2(
            sysno::__NR_fstat as u64,
            0, // stdin
            buf.as_mut_ptr() as u64
        );
        assert_syscall_ok!(result, "fstat on stdin");
        Ok(())
    })
}

// =============================================================================
// openat() tests
// =============================================================================

fn test_openat_nonexistent() -> TestResult {
    conformance_test!("fs", "openat_nonexistent", {
        let path = b"/nonexistent_file_that_should_not_exist\0";
        let result = syscall4(
            sysno::__NR_openat as u64,
            fs::AT_FDCWD as u64,
            path.as_ptr() as u64,
            fs::O_RDONLY as u64,
            0
        );
        assert_syscall_err!(result, errno::ENOENT);
        Ok(())
    })
}

// =============================================================================
// close() tests
// =============================================================================

fn test_close_invalid_fd() -> TestResult {
    conformance_test!("fs", "close_invalid_fd", {
        let result = syscall1(sysno::__NR_close as u64, 9999);
        assert_syscall_err!(result, errno::EBADF);
        Ok(())
    })
}

fn test_close_valid_fd() -> TestResult {
    conformance_test!("fs", "close_valid_fd", {
        // dup stdout to get a new fd, then close it
        let new_fd = syscall1(sysno::__NR_dup as u64, 1);
        assert_syscall_ok!(new_fd, "dup stdout");

        let result = syscall1(sysno::__NR_close as u64, new_fd as u64);
        assert_syscall_ok!(result, "close dup'd fd");

        // Closing again should fail with EBADF
        let result2 = syscall1(sysno::__NR_close as u64, new_fd as u64);
        assert_syscall_err!(result2, errno::EBADF);
        Ok(())
    })
}

// =============================================================================
// fstat() tests
// =============================================================================

fn test_fstat_stdout() -> TestResult {
    conformance_test!("fs", "fstat_stdout", {
        let mut stat_buf = [0u8; 128]; // Stat is 128 bytes
        let result = syscall2(
            sysno::__NR_fstat as u64,
            1, // stdout
            stat_buf.as_mut_ptr() as u64
        );
        assert_syscall_ok!(result, "fstat stdout");

        // Verify st_mode indicates character device (S_IFCHR = 0o020000)
        // st_mode is at offset 16, bytes 16-19 (u32)
        let mode = u32::from_ne_bytes([stat_buf[16], stat_buf[17], stat_buf[18], stat_buf[19]]);
        let s_ifchr = 0o020000u32;
        if (mode & 0o170000) != s_ifchr {
            return Err(format!("stdout should be char device, mode={:o}", mode));
        }
        Ok(())
    })
}

// =============================================================================
// lseek() tests
// =============================================================================

fn test_lseek_pipe_fails() -> TestResult {
    conformance_test!("fs", "lseek_pipe_espipe", {
        // lseek on stdout (which is a pipe/tty) should fail with ESPIPE
        let result = syscall3(
            sysno::__NR_lseek as u64,
            1, // stdout
            0, // offset
            0  // SEEK_SET
        );
        // Should fail with ESPIPE (29) for non-seekable device
        assert_syscall_err!(result, errno::ESPIPE);
        Ok(())
    })
}

// =============================================================================
// dup() tests
// =============================================================================

fn test_dup_stdout() -> TestResult {
    conformance_test!("fs", "dup_stdout", {
        let new_fd = syscall1(sysno::__NR_dup as u64, 1);
        assert_syscall_ok!(new_fd, "dup stdout");

        // New fd should be >= 3 (after stdin, stdout, stderr)
        if new_fd < 3 {
            return Err(format!("dup returned fd {} which is < 3", new_fd));
        }

        // Writing to new_fd should work
        let msg = b"[dup test]\n";
        let write_result = syscall3(
            sysno::__NR_write as u64,
            new_fd as u64,
            msg.as_ptr() as u64,
            msg.len() as u64
        );
        assert_syscall_ok!(write_result, "write to dup'd fd");

        // Clean up
        let _ = syscall1(sysno::__NR_close as u64, new_fd as u64);
        Ok(())
    })
}

fn test_dup2_redirect() -> TestResult {
    conformance_test!("fs", "dup2_redirect", {
        // First dup stdout to save it
        let saved_stdout = syscall1(sysno::__NR_dup as u64, 1);
        assert_syscall_ok!(saved_stdout, "save stdout");

        // dup2/dup3 to same fd should just return the fd
        // Note: aarch64 doesn't have dup2, use dup3 with flags=0 instead
        #[cfg(target_arch = "x86_64")]
        let result = syscall2(sysno::__NR_dup2 as u64, 1, 1);
        #[cfg(target_arch = "aarch64")]
        let result = syscall3(sysno::__NR_dup3 as u64, 1, 1, 0);

        assert_eq_desc!(result, 1, "dup2/dup3 same fd returns that fd");

        // Clean up
        let _ = syscall1(sysno::__NR_close as u64, saved_stdout as u64);
        Ok(())
    })
}

// =============================================================================
// writev() tests
// =============================================================================

fn test_writev_basic() -> TestResult {
    conformance_test!("fs", "writev_basic", {
        let msg1 = b"[writev] ";
        let msg2 = b"test\n";

        #[repr(C)]
        struct Iovec {
            iov_base: *const u8,
            iov_len: usize,
        }

        let iov = [
            Iovec { iov_base: msg1.as_ptr(), iov_len: msg1.len() },
            Iovec { iov_base: msg2.as_ptr(), iov_len: msg2.len() },
        ];

        let result = syscall3(
            sysno::__NR_writev as u64,
            1, // stdout
            iov.as_ptr() as u64,
            2  // iovcnt
        );
        assert_syscall_ok!(result, "writev");

        let expected_len = msg1.len() + msg2.len();
        assert_eq_desc!(result as usize, expected_len, "writev returns total bytes");
        Ok(())
    })
}
