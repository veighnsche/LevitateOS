//! Tests for memory management syscall wrappers
//!
//! Verifies that mmap, munmap, and mprotect constants and types are correct.

use libsyscall::mm::*;

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

    // Test mmap protection flags
    if test!("prot_flags", {
        assert_eq!(PROT_NONE, 0);
        assert_eq!(PROT_READ, 1);
        assert_eq!(PROT_WRITE, 2);
        assert_eq!(PROT_EXEC, 4);
        assert_eq!(PROT_READ | PROT_WRITE, 3);
        assert_eq!(PROT_READ | PROT_WRITE | PROT_EXEC, 7);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test mmap flags
    if test!("map_flags", {
        assert_eq!(MAP_SHARED, 0x01);
        assert_eq!(MAP_PRIVATE, 0x02);
        assert_eq!(MAP_FIXED, 0x10);
        assert_eq!(MAP_ANONYMOUS, 0x20);
        assert_eq!(MAP_ANONYMOUS | MAP_PRIVATE, 0x22);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test that MAP_SHARED and MAP_PRIVATE are mutually exclusive
    if test!("map_flags_exclusive", {
        assert_ne!(MAP_SHARED & MAP_PRIVATE, 0);
        assert_ne!(MAP_SHARED, MAP_PRIVATE);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test typical mmap flag combinations
    if test!("typical_mmap_combinations", {
        assert_eq!(MAP_ANONYMOUS | MAP_PRIVATE, 0x22);
        assert_eq!(MAP_ANONYMOUS | MAP_SHARED, 0x21);
        assert_eq!(MAP_FIXED | MAP_ANONYMOUS | MAP_PRIVATE, 0x32);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test page alignment assumptions
    if test!("page_size_assumptions", {
        let page_4k = 4096;
        let page_2m = 2 * 1024 * 1024;
        assert_eq!(page_4k & (page_4k - 1), 0);
        assert_eq!(page_2m & (page_2m - 1), 0);
        assert_eq!(0x1000 % 4096, 0);
        assert_eq!(0x200000 % (2 * 1024 * 1024), 0);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test protection combinations
    if test!("prot_combinations", {
        assert_eq!(PROT_READ, 1);
        assert_eq!(PROT_READ | PROT_WRITE, 3);
        assert_eq!(PROT_READ | PROT_EXEC, 5);
        assert_eq!(PROT_READ | PROT_WRITE | PROT_EXEC, 7);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    // Test protection bit checks
    if test!("prot_bit_checks", {
        let rw = PROT_READ | PROT_WRITE;
        assert_ne!(rw & PROT_READ, 0);
        assert_ne!(rw & PROT_WRITE, 0);
        assert_eq!(rw & PROT_EXEC, 0);
    }) {
        passed += 1;
    } else {
        failed += 1;
    }

    (passed, failed)
}
