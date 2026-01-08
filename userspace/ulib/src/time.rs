//! TEAM_170: Time abstractions for LevitateOS userspace.
//!
//! Provides `Duration`, `Instant`, and `sleep()` functions.

use core::ops::{Add, Sub};

/// TEAM_170: A duration of time.
///
/// Represents a span of time with nanosecond precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Duration {
    secs: u64,
    nanos: u32, // Always < 1_000_000_000
}

impl Duration {
    /// One second.
    pub const SECOND: Duration = Duration { secs: 1, nanos: 0 };

    /// One millisecond.
    pub const MILLISECOND: Duration = Duration {
        secs: 0,
        nanos: 1_000_000,
    };

    /// One microsecond.
    pub const MICROSECOND: Duration = Duration {
        secs: 0,
        nanos: 1_000,
    };

    /// Zero duration.
    pub const ZERO: Duration = Duration { secs: 0, nanos: 0 };

    /// TEAM_170: Create a new duration from seconds and nanoseconds.
    pub const fn new(secs: u64, nanos: u32) -> Self {
        Self {
            secs: secs + (nanos / 1_000_000_000) as u64,
            nanos: nanos % 1_000_000_000,
        }
    }

    /// TEAM_170: Create a duration from seconds.
    pub const fn from_secs(secs: u64) -> Self {
        Self { secs, nanos: 0 }
    }

    /// TEAM_170: Create a duration from milliseconds.
    pub const fn from_millis(millis: u64) -> Self {
        Self {
            secs: millis / 1_000,
            nanos: ((millis % 1_000) * 1_000_000) as u32,
        }
    }

    /// TEAM_170: Create a duration from microseconds.
    pub const fn from_micros(micros: u64) -> Self {
        Self {
            secs: micros / 1_000_000,
            nanos: ((micros % 1_000_000) * 1_000) as u32,
        }
    }

    /// TEAM_170: Create a duration from nanoseconds.
    pub const fn from_nanos(nanos: u64) -> Self {
        Self {
            secs: nanos / 1_000_000_000,
            nanos: (nanos % 1_000_000_000) as u32,
        }
    }

    /// TEAM_170: Get the seconds component.
    pub const fn as_secs(&self) -> u64 {
        self.secs
    }

    /// TEAM_170: Get the nanoseconds component (0-999,999,999).
    pub const fn subsec_nanos(&self) -> u32 {
        self.nanos
    }

    /// TEAM_170: Get the total duration in milliseconds.
    pub const fn as_millis(&self) -> u128 {
        (self.secs as u128) * 1_000 + (self.nanos as u128) / 1_000_000
    }

    /// TEAM_170: Get the total duration in microseconds.
    pub const fn as_micros(&self) -> u128 {
        (self.secs as u128) * 1_000_000 + (self.nanos as u128) / 1_000
    }

    /// TEAM_170: Get the total duration in nanoseconds.
    pub const fn as_nanos(&self) -> u128 {
        (self.secs as u128) * 1_000_000_000 + (self.nanos as u128)
    }

    /// TEAM_170: Check if the duration is zero.
    pub const fn is_zero(&self) -> bool {
        self.secs == 0 && self.nanos == 0
    }

    /// TEAM_170: Saturating addition.
    pub fn saturating_add(self, rhs: Self) -> Self {
        let total_nanos = self.nanos as u64 + rhs.nanos as u64;
        let extra_secs = total_nanos / 1_000_000_000;
        let nanos = (total_nanos % 1_000_000_000) as u32;
        let secs = self
            .secs
            .saturating_add(rhs.secs)
            .saturating_add(extra_secs);
        Self { secs, nanos }
    }

    /// TEAM_170: Saturating subtraction.
    pub fn saturating_sub(self, rhs: Self) -> Self {
        if self <= rhs {
            return Self::ZERO;
        }
        let (secs, borrow) = if self.nanos >= rhs.nanos {
            (self.secs - rhs.secs, 0)
        } else {
            (self.secs - rhs.secs - 1, 1_000_000_000)
        };
        let nanos = (self.nanos as u64 + borrow - rhs.nanos as u64) as u32;
        Self { secs, nanos }
    }
}

impl Add for Duration {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }
}

impl Sub for Duration {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
    }
}

/// TEAM_170: A measurement of a monotonically increasing clock.
///
/// Useful for measuring elapsed time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    secs: u64,
    nanos: u32,
}

impl Instant {
    /// TEAM_170: Get the current instant.
    pub fn now() -> Self {
        let mut ts: libsyscall::Timespec = unsafe { core::mem::zeroed() };
        // TEAM_186: Check syscall return, use zeros on error
        if libsyscall::clock_gettime(&mut ts) < 0 {
            return Self { secs: 0, nanos: 0 };
        }
        Self {
            secs: ts.tv_sec as u64,
            // TEAM_186: Clamp tv_nsec to valid range in case of corruption
            nanos: (ts.tv_nsec.min(999_999_999)) as u32,
        }
    }

    /// TEAM_170: Get the duration since this instant was created.
    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        now.duration_since(*self)
    }

    /// TEAM_170: Get the duration between two instants.
    pub fn duration_since(&self, earlier: Self) -> Duration {
        if *self <= earlier {
            return Duration::ZERO;
        }
        let (secs, borrow) = if self.nanos >= earlier.nanos {
            (self.secs - earlier.secs, 0)
        } else {
            (self.secs - earlier.secs - 1, 1_000_000_000)
        };
        let nanos = (self.nanos as u64 + borrow - earlier.nanos as u64) as u32;
        Duration { secs, nanos }
    }

    /// TEAM_170: Add a duration to this instant.
    pub fn checked_add(&self, duration: Duration) -> Option<Self> {
        let total_nanos = self.nanos as u64 + duration.nanos as u64;
        let extra_secs = total_nanos / 1_000_000_000;
        let nanos = (total_nanos % 1_000_000_000) as u32;
        let secs = self
            .secs
            .checked_add(duration.secs)?
            .checked_add(extra_secs)?;
        Some(Self { secs, nanos })
    }
}

impl Add<Duration> for Instant {
    type Output = Self;
    fn add(self, rhs: Duration) -> Self {
        self.checked_add(rhs).unwrap_or(Self {
            secs: u64::MAX,
            nanos: 999_999_999,
        })
    }
}

impl Sub<Duration> for Instant {
    type Output = Self;
    fn sub(self, rhs: Duration) -> Self {
        let dur = Duration::new(self.secs, self.nanos);
        let result = dur.saturating_sub(rhs);
        Self {
            secs: result.secs,
            nanos: result.nanos,
        }
    }
}

impl Sub for Instant {
    type Output = Duration;
    fn sub(self, rhs: Self) -> Duration {
        self.duration_since(rhs)
    }
}

/// TEAM_170: Sleep for the specified duration.
///
/// # Example
/// ```rust
/// use ulib::time::{Duration, sleep};
///
/// // Sleep for 100 milliseconds
/// sleep(Duration::from_millis(100));
/// ```
pub fn sleep(duration: Duration) {
    let _ = libsyscall::nanosleep(duration.secs, duration.nanos as u64);
}

/// TEAM_170: Sleep for the specified number of milliseconds.
pub fn sleep_ms(millis: u64) {
    sleep(Duration::from_millis(millis));
}
