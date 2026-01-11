//! TEAM_424: Process syscall conformance tests
//!
//! Tests: getpid, getppid, gettid, exit_group, clock_gettime, getrandom

use crate::{conformance_test, assert_syscall_ok, assert_eq_desc, TestResult};
use libsyscall::{sysno, time};
use libsyscall::arch::{syscall0, syscall2, syscall3};

pub fn run_tests() -> Vec<TestResult> {
    vec![
        test_getpid(),
        test_getppid(),
        test_gettid(),
        test_clock_gettime_monotonic(),
        test_clock_gettime_realtime(),
        test_getrandom(),
    ]
}

// =============================================================================
// getpid/getppid/gettid tests
// =============================================================================

fn test_getpid() -> TestResult {
    conformance_test!("process", "getpid", {
        let pid = syscall0(sysno::__NR_getpid as u64);

        // PID should be positive
        if pid <= 0 {
            return Err(format!("getpid returned invalid pid: {}", pid));
        }

        // Calling again should return same value
        let pid2 = syscall0(sysno::__NR_getpid as u64);
        assert_eq_desc!(pid, pid2, "getpid should be consistent");
        Ok(())
    })
}

fn test_getppid() -> TestResult {
    conformance_test!("process", "getppid", {
        let ppid = syscall0(sysno::__NR_getppid as u64);

        // PPID should be positive (or 0 for init, 1 for orphaned processes)
        if ppid < 0 {
            return Err(format!("getppid returned invalid ppid: {}", ppid));
        }
        Ok(())
    })
}

fn test_gettid() -> TestResult {
    conformance_test!("process", "gettid", {
        let tid = syscall0(sysno::__NR_gettid as u64);

        // TID should be positive
        if tid <= 0 {
            return Err(format!("gettid returned invalid tid: {}", tid));
        }

        // For single-threaded process, TID should equal PID
        let pid = syscall0(sysno::__NR_getpid as u64);
        assert_eq_desc!(tid, pid, "single-threaded: tid should equal pid");
        Ok(())
    })
}

// =============================================================================
// clock_gettime tests
// =============================================================================

fn test_clock_gettime_monotonic() -> TestResult {
    conformance_test!("process", "clock_gettime_monotonic", {
        let mut ts = time::Timespec { tv_sec: 0, tv_nsec: 0 };

        let result = syscall2(
            sysno::__NR_clock_gettime as u64,
            1, // CLOCK_MONOTONIC
            &mut ts as *mut time::Timespec as u64
        );
        assert_syscall_ok!(result, "clock_gettime MONOTONIC");

        // Time should be non-negative
        if ts.tv_sec < 0 {
            return Err(format!("monotonic time negative: {}", ts.tv_sec));
        }

        // Nanoseconds should be 0-999999999
        if ts.tv_nsec < 0 || ts.tv_nsec >= 1_000_000_000 {
            return Err(format!("invalid tv_nsec: {}", ts.tv_nsec));
        }

        // Get time again - should be >= previous
        let mut ts2 = time::Timespec { tv_sec: 0, tv_nsec: 0 };
        let _ = syscall2(
            sysno::__NR_clock_gettime as u64,
            1,
            &mut ts2 as *mut time::Timespec as u64
        );

        if ts2.tv_sec < ts.tv_sec || (ts2.tv_sec == ts.tv_sec && ts2.tv_nsec < ts.tv_nsec) {
            return Err("monotonic time went backwards".to_string());
        }
        Ok(())
    })
}

fn test_clock_gettime_realtime() -> TestResult {
    conformance_test!("process", "clock_gettime_realtime", {
        let mut ts = time::Timespec { tv_sec: 0, tv_nsec: 0 };

        let result = syscall2(
            sysno::__NR_clock_gettime as u64,
            0, // CLOCK_REALTIME
            &mut ts as *mut time::Timespec as u64
        );
        assert_syscall_ok!(result, "clock_gettime REALTIME");

        // Nanoseconds should be valid
        if ts.tv_nsec < 0 || ts.tv_nsec >= 1_000_000_000 {
            return Err(format!("invalid tv_nsec: {}", ts.tv_nsec));
        }
        Ok(())
    })
}

// =============================================================================
// getrandom tests
// =============================================================================

fn test_getrandom() -> TestResult {
    conformance_test!("process", "getrandom", {
        let mut buf = [0u8; 32];

        let result = syscall3(
            sysno::__NR_getrandom as u64,
            buf.as_mut_ptr() as u64,
            buf.len() as u64,
            0 // flags
        );
        assert_syscall_ok!(result, "getrandom");

        // Should return requested bytes
        assert_eq_desc!(result as usize, buf.len(), "getrandom should return requested bytes");

        // Buffer should not be all zeros (extremely unlikely for 32 random bytes)
        let all_zeros = buf.iter().all(|&b| b == 0);
        if all_zeros {
            return Err("getrandom returned all zeros (suspicious)".to_string());
        }

        // Get another batch - should be different
        let mut buf2 = [0u8; 32];
        let _ = syscall3(
            sysno::__NR_getrandom as u64,
            buf2.as_mut_ptr() as u64,
            buf2.len() as u64,
            0
        );

        if buf == buf2 {
            return Err("getrandom returned same bytes twice (suspicious)".to_string());
        }
        Ok(())
    })
}
