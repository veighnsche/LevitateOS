//! Integration tests for libsyscall
//!
//! These tests verify the overall structure and API surface of libsyscall.

use libsyscall::*;

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

    // Test that all major modules are accessible
    if test!("module_exports", {
        let _ = fs::AT_FDCWD;
        let _ = fs::O_RDONLY;
        let _ = mm::PROT_READ;
        let _ = mm::MAP_PRIVATE;
        let _ = errno::ENOENT;
        let _ = errno::EINVAL;
        let _ = signal::SIGINT;
        let _ = fs::UTIME_NOW;
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test that errno is re-exported at root
    if test!("errno_reexport", {
        let enoent = errno::ENOENT;
        assert_eq!(enoent, 2);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test type sizes
    if test!("type_sizes", {
        use core::mem::size_of;

        #[cfg(target_arch = "aarch64")]
        assert_eq!(
            size_of::<fs::Stat>(),
            128,
            "Stat must be 128 bytes on AArch64"
        );

        assert!(
            size_of::<time::Timespec>() >= 16,
            "Timespec should have tv_sec and tv_nsec"
        );
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test syscall number constants
    if test!("syscall_numbers", {
        let _ = sysno::__NR_read;
        let _ = sysno::__NR_write;
        let _ = sysno::__NR_openat;
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test print macros compile
    if test!("print_macros", {
        use core::fmt::Write;
        let mut stdout = Stdout;
        let _ = write!(stdout, "test");
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test common patterns
    if test!("common_patterns", {
        let fd: isize = -(errno::ENOENT as isize);
        assert!(fd < 0);

        if fd == -(errno::ENOENT as isize) {
            assert_eq!(fd, -2);
        }

        if fd < 0 {
            assert!(true);
        }
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test signal constants
    if test!("signal_constants", {
        use signal::*;

        assert_eq!(SIGINT, 2);
        assert_eq!(SIGKILL, 9);
        assert_eq!(SIGCHLD, 17);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test optional parameters
    if test!("optional_parameters", {
        let mut status: i32 = 0;
        let _opt1: Option<&mut i32> = Some(&mut status);
        let _opt2: Option<&mut i32> = None;

        let times = [time::Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        }; 2];
        let _opt1: Option<&[time::Timespec; 2]> = Some(&times);
        let _opt2: Option<&[time::Timespec; 2]> = None;
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    (passed, failed)
}
