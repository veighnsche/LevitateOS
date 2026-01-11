//! TEAM_424: Memory syscall conformance tests
//!
//! Tests: mmap, munmap, mprotect, brk

use crate::{conformance_test, assert_syscall_ok, assert_eq_desc, TestResult};
use libsyscall::{errno, mm, sysno};
use libsyscall::arch::{syscall1, syscall6};

pub fn run_tests() -> Vec<TestResult> {
    vec![
        test_mmap_anonymous(),
        test_mmap_invalid_prot(),
        test_munmap_basic(),
        test_mprotect_basic(),
        test_brk_query(),
        test_brk_expand(),
    ]
}

// =============================================================================
// mmap() tests
// =============================================================================

fn test_mmap_anonymous() -> TestResult {
    conformance_test!("mem", "mmap_anonymous", {
        let page_size = 4096u64;

        // mmap anonymous private page
        let addr = syscall6(
            sysno::__NR_mmap as u64,
            0, // addr (let kernel choose)
            page_size,
            (mm::PROT_READ | mm::PROT_WRITE) as u64,
            (mm::MAP_ANONYMOUS | mm::MAP_PRIVATE) as u64,
            (-1i64) as u64, // fd (ignored for MAP_ANONYMOUS)
            0   // offset
        );

        if addr < 0 && addr > -4096 {
            return Err(format!("mmap failed with errno {}", -addr));
        }

        // Verify we can write to the page
        let ptr = addr as *mut u8;
        unsafe {
            *ptr = 42;
            if *ptr != 42 {
                return Err("mmap'd page not writable".to_string());
            }
        }

        // Clean up
        let result = syscall6(
            sysno::__NR_munmap as u64,
            addr as u64,
            page_size,
            0, 0, 0, 0
        );
        assert_syscall_ok!(result, "munmap");
        Ok(())
    })
}

fn test_mmap_invalid_prot() -> TestResult {
    conformance_test!("mem", "mmap_write_without_read", {
        // PROT_WRITE without PROT_READ is often invalid on many architectures
        // Some systems allow it, some don't - we just verify it doesn't crash
        let page_size = 4096u64;

        let addr = syscall6(
            sysno::__NR_mmap as u64,
            0,
            page_size,
            mm::PROT_WRITE as u64, // Write-only (unusual)
            (mm::MAP_ANONYMOUS | mm::MAP_PRIVATE) as u64,
            (-1i64) as u64,
            0
        );

        // Either succeeds or fails with EINVAL/EACCES - both are acceptable
        if addr >= 0 || addr == -(errno::EINVAL as i64) || addr == -(errno::EACCES as i64) {
            // Clean up if it succeeded
            if addr >= 0 {
                let _ = syscall6(sysno::__NR_munmap as u64, addr as u64, page_size, 0, 0, 0, 0);
            }
            Ok(())
        } else {
            Err(format!("unexpected mmap error: {}", addr))
        }
    })
}

// =============================================================================
// munmap() tests
// =============================================================================

fn test_munmap_basic() -> TestResult {
    conformance_test!("mem", "munmap_basic", {
        let page_size = 4096u64;

        // First mmap a page
        let addr = syscall6(
            sysno::__NR_mmap as u64,
            0,
            page_size,
            (mm::PROT_READ | mm::PROT_WRITE) as u64,
            (mm::MAP_ANONYMOUS | mm::MAP_PRIVATE) as u64,
            (-1i64) as u64,
            0
        );
        assert_syscall_ok!(addr, "mmap for munmap test");

        // Now unmap it
        let result = syscall6(
            sysno::__NR_munmap as u64,
            addr as u64,
            page_size,
            0, 0, 0, 0
        );
        assert_syscall_ok!(result, "munmap");
        Ok(())
    })
}

// =============================================================================
// mprotect() tests
// =============================================================================

fn test_mprotect_basic() -> TestResult {
    conformance_test!("mem", "mprotect_basic", {
        let page_size = 4096u64;

        // First mmap a writable page
        let addr = syscall6(
            sysno::__NR_mmap as u64,
            0,
            page_size,
            (mm::PROT_READ | mm::PROT_WRITE) as u64,
            (mm::MAP_ANONYMOUS | mm::MAP_PRIVATE) as u64,
            (-1i64) as u64,
            0
        );
        assert_syscall_ok!(addr, "mmap");

        // Write something to verify it's writable
        unsafe {
            let ptr = addr as *mut u8;
            *ptr = 0x42;
        }

        // Change to read-only - mprotect takes 3 args
        let result = syscall6(
            sysno::__NR_mprotect as u64,
            addr as u64,
            page_size,
            mm::PROT_READ as u64,
            0, 0, 0
        );
        assert_syscall_ok!(result, "mprotect to PROT_READ");

        // Verify we can still read
        unsafe {
            let ptr = addr as *const u8;
            if *ptr != 0x42 {
                return Err("data changed after mprotect".to_string());
            }
        }

        // Clean up
        let _ = syscall6(sysno::__NR_munmap as u64, addr as u64, page_size, 0, 0, 0, 0);
        Ok(())
    })
}

// =============================================================================
// brk() tests
// =============================================================================

fn test_brk_query() -> TestResult {
    conformance_test!("mem", "brk_query", {
        // brk(0) returns current program break
        let current_brk = syscall1(sysno::__NR_brk as u64, 0);

        // Should be a valid address (positive)
        if current_brk <= 0 {
            return Err(format!("brk(0) returned invalid address: {}", current_brk));
        }

        // Calling again should return same value
        let current_brk2 = syscall1(sysno::__NR_brk as u64, 0);
        assert_eq_desc!(current_brk, current_brk2, "brk(0) should be consistent");
        Ok(())
    })
}

fn test_brk_expand() -> TestResult {
    conformance_test!("mem", "brk_expand", {
        let page_size = 4096i64;

        // Get current brk
        let current_brk = syscall1(sysno::__NR_brk as u64, 0);
        assert_syscall_ok!(current_brk, "get current brk");

        // Expand by one page
        let new_brk_request = current_brk + page_size;
        let new_brk = syscall1(sysno::__NR_brk as u64, new_brk_request as u64);

        // Should succeed and return the new break
        if new_brk < current_brk {
            return Err(format!(
                "brk expansion failed: requested {}, got {}",
                new_brk_request, new_brk
            ));
        }

        // Write to the new memory (should not fault)
        unsafe {
            let ptr = current_brk as *mut u8;
            *ptr = 0xAB;
            if *ptr != 0xAB {
                return Err("couldn't write to expanded brk region".to_string());
            }
        }

        // Shrink back (optional - some systems don't shrink)
        let _ = syscall1(sysno::__NR_brk as u64, current_brk as u64);
        Ok(())
    })
}
