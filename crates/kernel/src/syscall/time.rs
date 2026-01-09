use crate::memory::user as mm_user;

use crate::syscall::{Timespec, errno};

/// TEAM_198: Get uptime in seconds (for tmpfs timestamps).
pub fn uptime_seconds() -> u64 {
    let freq = read_timer_frequency();
    if freq == 0 {
        return 0;
    }
    read_timer_counter() / freq
}

/// TEAM_170: Read the ARM generic timer counter.
#[inline]
fn read_timer_counter() -> u64 {
    crate::arch::time::read_timer_counter()
}

/// TEAM_170: Read the ARM generic timer frequency.
#[inline]
fn read_timer_frequency() -> u64 {
    crate::arch::time::read_timer_frequency()
}

/// TEAM_170: sys_nanosleep - Sleep for specified duration.
pub fn sys_nanosleep(seconds: u64, nanoseconds: u64) -> i64 {
    // TEAM_186: Normalize nanoseconds if > 1e9
    let extra_secs = nanoseconds / 1_000_000_000;
    let norm_nanos = nanoseconds % 1_000_000_000;
    let total_secs = seconds.saturating_add(extra_secs);

    let freq = read_timer_frequency();
    if freq == 0 {
        // TEAM_186: Use saturating arithmetic for fallback path
        let millis = total_secs
            .saturating_mul(1000)
            .saturating_add(norm_nanos / 1_000_000);
        for _ in 0..(millis.min(u64::MAX as u64) as usize).min(1_000_000) {
            crate::task::yield_now();
        }
        return 0;
    }

    let start = read_timer_counter();

    // TEAM_186: Calculate ticks with overflow protection
    // ticks = seconds * freq + (nanoseconds * freq) / 1e9
    // Split calculation to avoid overflow
    let ticks_from_secs = total_secs.saturating_mul(freq);
    let ticks_from_nanos = (norm_nanos as u128 * freq as u128 / 1_000_000_000) as u64;
    let ticks_to_wait = ticks_from_secs.saturating_add(ticks_from_nanos);

    let target = start.saturating_add(ticks_to_wait);

    while read_timer_counter() < target {
        crate::task::yield_now();
    }

    0
}

/// TEAM_170: sys_clock_gettime - Get current monotonic time.
pub fn sys_clock_gettime(timespec_buf: usize) -> i64 {
    let task = crate::task::current_task();
    let ts_size = core::mem::size_of::<Timespec>();
    if mm_user::validate_user_buffer(task.ttbr0, timespec_buf, ts_size, true).is_err() {
        return errno::EFAULT;
    }

    let freq = read_timer_frequency();
    let counter = read_timer_counter();

    let ts = if freq > 0 {
        let seconds = counter / freq;
        let remainder = counter % freq;
        let nanoseconds = (remainder * 1_000_000_000) / freq;
        Timespec {
            tv_sec: seconds as i64,
            tv_nsec: nanoseconds as i64,
        }
    } else {
        Timespec::default()
    };

    let ts_bytes =
        unsafe { core::slice::from_raw_parts(&ts as *const Timespec as *const u8, ts_size) };

    for (i, &byte) in ts_bytes.iter().enumerate() {
        if let Some(ptr) = mm_user::user_va_to_kernel_ptr(task.ttbr0, timespec_buf + i) {
            unsafe { *ptr = byte };
        } else {
            return errno::EFAULT;
        }
    }

    0
}
