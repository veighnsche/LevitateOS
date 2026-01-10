//! Tests for errno constants and error handling
//!
//! Verifies that errno values match Linux ABI and are correctly defined.

use libsyscall::errno::*;

macro_rules! test {
    ($name:expr, $test:expr) => {{
        std::print!("  {} ... ", $name);
        match std::panic::catch_unwind(|| $test) {
            Ok(_) => {
                std::println!("✓");
                true
            }
            Err(_) => {
                std::println!("✗");
                false
            }
        }
    }};
}

pub fn run_tests() -> (usize, usize) {
    let mut passed = 0;
    let mut failed = 0;

    // Test common errno values match Linux ABI
    if test!("common_errno_values", {
        assert_eq!(EPERM, 1, "Operation not permitted");
        assert_eq!(ENOENT, 2, "No such file or directory");
        assert_eq!(ESRCH, 3, "No such process");
        assert_eq!(EINTR, 4, "Interrupted system call");
        assert_eq!(EIO, 5, "I/O error");
        assert_eq!(EBADF, 9, "Bad file descriptor");
        assert_eq!(EAGAIN, 11, "Try again");
        assert_eq!(ENOMEM, 12, "Out of memory");
        assert_eq!(EACCES, 13, "Permission denied");
        assert_eq!(EFAULT, 14, "Bad address");
        assert_eq!(EEXIST, 17, "File exists");
        assert_eq!(EXDEV, 18, "Cross-device link");
        assert_eq!(ENOTDIR, 20, "Not a directory");
        assert_eq!(EISDIR, 21, "Is a directory");
        assert_eq!(EINVAL, 22, "Invalid argument");
        assert_eq!(EMFILE, 24, "Too many open files");
        assert_eq!(ENOSPC, 28, "No space left on device");
        assert_eq!(EROFS, 30, "Read-only file system");
        assert_eq!(ENAMETOOLONG, 36, "File name too long");
        assert_eq!(ENOSYS, 38, "Function not implemented");
        assert_eq!(ENOTEMPTY, 39, "Directory not empty");
        assert_eq!(ELOOP, 40, "Too many symbolic links");
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test that syscalls return negative errno values
    if test!("negative_errno_convention", {
        let error_enoent = -(ENOENT as isize);
        let error_einval = -(EINVAL as isize);

        assert_eq!(error_enoent, -2);
        assert_eq!(error_einval, -22);
        assert!(error_enoent < 0);
        assert!(error_einval < 0);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test errno range validity
    if test!("errno_ranges", {
        assert!(EPERM > 0 && EPERM < 200);
        assert!(ENOENT > 0 && ENOENT < 200);
        assert!(ENAMETOOLONG > 0 && ENAMETOOLONG < 200);
        assert!(ENOSYS > 0 && ENOSYS < 200);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test that EAGAIN and EWOULDBLOCK are the same
    if test!("eagain_ewouldblock", {
        assert_eq!(EAGAIN, 11);
        assert_eq!(EWOULDBLOCK, 11);
        assert_eq!(EAGAIN, EWOULDBLOCK);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    (passed, failed)
}
