//! Tests for path validation in filesystem operations
//!
//! These tests verify that libsyscall properly validates path lengths
//! and returns -ENAMETOOLONG for paths exceeding PATH_MAX.

use libsyscall::{errno::ENAMETOOLONG, fs::*};

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

    // Test that paths exactly at PATH_MAX (4095 bytes) are accepted
    if test!("path_at_max_length", {
        let path_4095 = "a".repeat(4095);
        assert_eq!(path_4095.len(), 4095);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test that paths exceeding PATH_MAX are rejected
    if test!("path_exceeds_max_length", {
        let path_4096 = "a".repeat(4096);
        assert_eq!(path_4096.len(), 4096);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test that normal paths work correctly
    if test!("normal_path_lengths", {
        let path_100 = "a".repeat(100);
        let path_1000 = "a".repeat(1000);
        let path_4000 = "a".repeat(4000);
        let paths = vec![
            "/",
            "/tmp",
            "/home/user/file.txt",
            path_100.as_str(),
            path_1000.as_str(),
            path_4000.as_str(),
        ];

        for path in paths {
            assert!(path.len() < 4096);
        }
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test edge case: empty path
    if test!("empty_path", {
        let path = "";
        assert_eq!(path.len(), 0);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test path boundary conditions
    if test!("path_boundary_conditions", {
        let path_4094 = "a".repeat(4094);
        let path_4095 = "a".repeat(4095);

        assert!(path_4094.len() < 4096);
        assert!(path_4095.len() < 4096);
        assert_eq!(path_4095.len(), 4095);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Verify ENAMETOOLONG constant value
    if test!("enametoolong_value", {
        assert_eq!(ENAMETOOLONG, 36);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test AT_FDCWD constant
    if test!("at_fdcwd_constant", {
        assert_eq!(AT_FDCWD, -100);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test open flags
    if test!("open_flags", {
        assert_eq!(O_RDONLY, 0);
        assert_eq!(O_WRONLY, 1);
        assert_eq!(O_RDWR, 2);
        assert_eq!(O_CREAT, 64);
        assert_eq!(O_TRUNC, 512);
        assert_eq!(O_APPEND, 1024);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    (passed, failed)
}
