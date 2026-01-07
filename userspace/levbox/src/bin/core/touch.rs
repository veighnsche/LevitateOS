//! TEAM_198: `touch` utility for LevitateOS
//! TEAM_199: Enhanced with full -t, -r, -d timestamp support
//!
//! Change file timestamps or create empty files.
//! See `docs/specs/levbox/touch.md` for specification.

#![no_std]
#![no_main]

extern crate alloc;
extern crate ulib;

use alloc::string::String;
use alloc::vec::Vec;
use libsyscall::{openat, println, utimensat, Timespec, UTIME_NOW, UTIME_OMIT};

// ============================================================================
// Constants
// ============================================================================

const AT_FDCWD: i32 = -100;
const O_CREAT: u32 = 0o100;
const O_WRONLY: u32 = 0o1;

// ============================================================================
// Help and Version
// ============================================================================

fn print_help() {
    println!("Usage: touch [OPTION]... FILE...");
    println!("Update the access and modification times of each FILE to the current time.");
    println!();
    println!("A FILE argument that does not exist is created empty, unless -c or -h");
    println!("is supplied.");
    println!();
    println!("  -a                     change only the access time");
    println!("  -c, --no-create        do not create any files");
    println!("  -d, --date=STRING      parse STRING and use it instead of current time");
    println!("  -m                     change only the modification time");
    println!("  -r, --reference=FILE   use this file's times instead of current time");
    println!("  -t STAMP               use [[CC]YY]MMDDhhmm[.ss] instead of current time");
    println!("      --help             display this help and exit");
    println!("      --version          output version information and exit");
}

fn print_version() {
    println!("touch (LevitateOS levbox) 0.1.0");
}

// ============================================================================
// Time Parsing: -t [[CC]YY]MMDDhhmm[.SS]
// ============================================================================

/// TEAM_199: Days in each month (non-leap year)
const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// TEAM_199: Check if year is a leap year
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// TEAM_199: Get days in a month (accounting for leap years)
fn days_in_month(year: u32, month: u32) -> u32 {
    if month == 2 && is_leap_year(year) {
        29
    } else {
        DAYS_IN_MONTH[(month - 1) as usize]
    }
}

/// TEAM_199: Convert date components to seconds since Unix epoch (1970-01-01 00:00:00 UTC)
fn datetime_to_epoch(year: u32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> u64 {
    // Count days from 1970 to the given date
    let mut days: u64 = 0;

    // Add days for complete years
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Add days for complete months in the target year
    for m in 1..month {
        days += days_in_month(year, m) as u64;
    }

    // Add remaining days (day is 1-indexed)
    days += (day - 1) as u64;

    // Convert to seconds and add time components
    days * 86400 + (hour as u64) * 3600 + (min as u64) * 60 + (sec as u64)
}

/// TEAM_199: Parse a 2-digit number from a string slice
fn parse_2digits(s: &str) -> Option<u32> {
    if s.len() < 2 {
        return None;
    }
    let bytes = s.as_bytes();
    let d1 = bytes[0].wrapping_sub(b'0');
    let d2 = bytes[1].wrapping_sub(b'0');
    if d1 > 9 || d2 > 9 {
        return None;
    }
    Some((d1 as u32) * 10 + (d2 as u32))
}

/// TEAM_199: Parse -t timestamp format: [[CC]YY]MMDDhhmm[.SS]
/// Returns seconds since epoch, or None on parse error.
fn parse_touch_timestamp(stamp: &str) -> Option<u64> {
    // Split on '.' to separate seconds
    let (main_part, seconds) = if let Some(dot_pos) = stamp.find('.') {
        let (main, sec) = stamp.split_at(dot_pos);
        let sec_str = &sec[1..]; // Skip the '.'
        if sec_str.len() != 2 {
            return None;
        }
        (main, parse_2digits(sec_str)?)
    } else {
        (stamp, 0u32)
    };

    // Validate seconds
    if seconds > 60 {
        return None;
    }

    // Parse based on length:
    // 8 chars: MMDDhhmm (use current year)
    // 10 chars: YYMMDDhhmm (use current century)
    // 12 chars: CCYYMMDDhhmm
    let (year, month, day, hour, min) = match main_part.len() {
        8 => {
            // MMDDhhmm - use current year (2025 for LevitateOS)
            let month = parse_2digits(&main_part[0..2])?;
            let day = parse_2digits(&main_part[2..4])?;
            let hour = parse_2digits(&main_part[4..6])?;
            let min = parse_2digits(&main_part[6..8])?;
            (2025u32, month, day, hour, min)
        }
        10 => {
            // YYMMDDhhmm - use century 20
            let yy = parse_2digits(&main_part[0..2])?;
            let month = parse_2digits(&main_part[2..4])?;
            let day = parse_2digits(&main_part[4..6])?;
            let hour = parse_2digits(&main_part[6..8])?;
            let min = parse_2digits(&main_part[8..10])?;
            let year = 2000 + yy;
            (year, month, day, hour, min)
        }
        12 => {
            // CCYYMMDDhhmm
            let cc = parse_2digits(&main_part[0..2])?;
            let yy = parse_2digits(&main_part[2..4])?;
            let month = parse_2digits(&main_part[4..6])?;
            let day = parse_2digits(&main_part[6..8])?;
            let hour = parse_2digits(&main_part[8..10])?;
            let min = parse_2digits(&main_part[10..12])?;
            let year = cc * 100 + yy;
            (year, month, day, hour, min)
        }
        _ => return None,
    };

    // Validate components
    if month < 1 || month > 12 {
        return None;
    }
    if day < 1 || day > days_in_month(year, month) {
        return None;
    }
    if hour > 23 {
        return None;
    }
    if min > 59 {
        return None;
    }

    Some(datetime_to_epoch(year, month, day, hour, min, seconds))
}

// ============================================================================
// Time Parsing: -d date string (simplified)
// ============================================================================

/// TEAM_199: Parse -d date string format (simplified subset)
/// Supports: "YYYY-MM-DD HH:MM:SS", "YYYY-MM-DD", "now"
fn parse_date_string(s: &str) -> Option<u64> {
    let s = s.trim();

    // Handle "now"
    if s == "now" {
        return None; // Signal to use UTIME_NOW
    }

    // Try "YYYY-MM-DD HH:MM:SS" format
    if s.len() >= 10 && s.as_bytes()[4] == b'-' && s.as_bytes()[7] == b'-' {
        let year = parse_4digits(&s[0..4])?;
        let month = parse_2digits(&s[5..7])?;
        let day = parse_2digits(&s[8..10])?;

        let (hour, min, sec) = if s.len() >= 19 && s.as_bytes()[10] == b' ' {
            let hour = parse_2digits(&s[11..13])?;
            let min = parse_2digits(&s[14..16])?;
            let sec = parse_2digits(&s[17..19])?;
            (hour, min, sec)
        } else {
            (0, 0, 0)
        };

        // Validate
        if month < 1 || month > 12 {
            return None;
        }
        if day < 1 || day > days_in_month(year, month) {
            return None;
        }
        if hour > 23 || min > 59 || sec > 60 {
            return None;
        }

        return Some(datetime_to_epoch(year, month, day, hour, min, sec));
    }

    None
}

/// TEAM_199: Parse 4-digit number
fn parse_4digits(s: &str) -> Option<u32> {
    if s.len() < 4 {
        return None;
    }
    let bytes = s.as_bytes();
    let mut val = 0u32;
    for i in 0..4 {
        let d = bytes[i].wrapping_sub(b'0');
        if d > 9 {
            return None;
        }
        val = val * 10 + (d as u32);
    }
    Some(val)
}

// ============================================================================
// Reference File Support (-r)
// ============================================================================

/// TEAM_199: Get timestamps from a reference file
fn get_reference_times(ref_path: &str) -> Option<(u64, u64)> {
    // Open the reference file
    let fd = openat(ref_path, 0); // O_RDONLY
    if fd < 0 {
        return None;
    }

    // Get file stats
    let mut stat = libsyscall::Stat::default();
    let ret = libsyscall::fstat(fd as usize, &mut stat);
    libsyscall::close(fd as usize);

    if ret < 0 {
        return None;
    }

    // Return access time and modification time from stat
    Some((stat.st_atime as u64, stat.st_mtime as u64))
}

// ============================================================================
// Time Source Enum
// ============================================================================

/// TEAM_199: Source of timestamp to use
#[derive(Clone)]
enum TimeSource {
    /// Use current time (UTIME_NOW)
    Now,
    /// Use specific epoch timestamp
    Epoch(u64),
    /// Use times from reference file
    Reference(String),
}

// ============================================================================
// Core Logic
// ============================================================================

/// TEAM_199: Touch a file with specified time source
fn touch_file(
    path: &str,
    no_create: bool,
    atime_only: bool,
    mtime_only: bool,
    time_source: &TimeSource,
) -> bool {
    // Determine the timestamp to use
    let (atime_spec, mtime_spec) = match time_source {
        TimeSource::Now => {
            let atime = Timespec {
                tv_sec: 0,
                tv_nsec: if mtime_only {
                    UTIME_OMIT as i64
                } else {
                    UTIME_NOW as i64
                },
            };
            let mtime = Timespec {
                tv_sec: 0,
                tv_nsec: if atime_only {
                    UTIME_OMIT as i64
                } else {
                    UTIME_NOW as i64
                },
            };
            (atime, mtime)
        }
        TimeSource::Epoch(epoch_secs) => {
            let atime = Timespec {
                tv_sec: if mtime_only { 0 } else { *epoch_secs as i64 },
                tv_nsec: if mtime_only { UTIME_OMIT as i64 } else { 0 },
            };
            let mtime = Timespec {
                tv_sec: if atime_only { 0 } else { *epoch_secs as i64 },
                tv_nsec: if atime_only { UTIME_OMIT as i64 } else { 0 },
            };
            (atime, mtime)
        }
        TimeSource::Reference(ref_path) => {
            // Try to get times from reference file
            if let Some((ref_atime, ref_mtime)) = get_reference_times(ref_path) {
                let atime = Timespec {
                    tv_sec: if mtime_only { 0 } else { ref_atime as i64 },
                    tv_nsec: if mtime_only { UTIME_OMIT as i64 } else { 0 },
                };
                let mtime = Timespec {
                    tv_sec: if atime_only { 0 } else { ref_mtime as i64 },
                    tv_nsec: if atime_only { UTIME_OMIT as i64 } else { 0 },
                };
                (atime, mtime)
            } else {
                libsyscall::write(2, b"touch: failed to get attributes of '");
                libsyscall::write(2, ref_path.as_bytes());
                libsyscall::write(2, b"'\n");
                return false;
            }
        }
    };

    let times = [atime_spec, mtime_spec];

    // First, try to update timestamps on existing file
    let ret = utimensat(AT_FDCWD, path, Some(&times), 0);

    if ret == 0 {
        return true; // Success - file existed and timestamps updated
    }

    // File doesn't exist - check if we should create it
    if no_create {
        return true; // -c flag: don't create, but don't error
    }

    // Try to create the file
    let ret = openat(path, O_CREAT | O_WRONLY);
    if ret < 0 {
        libsyscall::write(2, b"touch: cannot touch '");
        libsyscall::write(2, path.as_bytes());
        libsyscall::write(2, b"': ");
        if ret == -30 {
            libsyscall::write(2, b"Read-only file system\n");
        } else if ret == -2 {
            libsyscall::write(2, b"No such file or directory\n");
        } else {
            libsyscall::write(2, b"Error\n");
        }
        return false;
    }

    // Close the file
    libsyscall::close(ret as usize);

    // If we created the file and have a specific time, set it now
    if let TimeSource::Epoch(_) = time_source {
        let _ = utimensat(AT_FDCWD, path, Some(&times), 0);
    }

    true
}

// ============================================================================
// Entry Point
// ============================================================================

#[no_mangle]
pub fn main() -> i32 {
    let mut no_create = false;
    let mut atime_only = false;
    let mut mtime_only = false;
    let mut time_source = TimeSource::Now;
    let mut files: Vec<String> = Vec::new();
    let mut expect_t_arg = false;
    let mut expect_d_arg = false;
    let mut expect_r_arg = false;

    for arg in ulib::env::args().skip(1) {
        // Handle arguments that expect a following value
        if expect_t_arg {
            expect_t_arg = false;
            match parse_touch_timestamp(&arg) {
                Some(epoch) => time_source = TimeSource::Epoch(epoch),
                None => {
                    libsyscall::write(2, b"touch: invalid date format '");
                    libsyscall::write(2, arg.as_bytes());
                    libsyscall::write(2, b"'\n");
                    return 1;
                }
            }
            continue;
        }
        if expect_d_arg {
            expect_d_arg = false;
            match parse_date_string(&arg) {
                Some(epoch) => time_source = TimeSource::Epoch(epoch),
                None => {
                    // "now" returns None, use current time
                    if arg.trim() == "now" {
                        time_source = TimeSource::Now;
                    } else {
                        libsyscall::write(2, b"touch: invalid date '");
                        libsyscall::write(2, arg.as_bytes());
                        libsyscall::write(2, b"'\n");
                        return 1;
                    }
                }
            }
            continue;
        }
        if expect_r_arg {
            expect_r_arg = false;
            time_source = TimeSource::Reference(String::from(arg));
            continue;
        }

        // Parse options
        if arg == "--help" {
            print_help();
            return 0;
        } else if arg == "--version" {
            print_version();
            return 0;
        } else if arg == "-c" || arg == "--no-create" {
            no_create = true;
        } else if arg == "-a" {
            atime_only = true;
        } else if arg == "-m" {
            mtime_only = true;
        } else if arg == "-t" {
            expect_t_arg = true;
        } else if arg == "-d" {
            expect_d_arg = true;
        } else if arg == "-r" {
            expect_r_arg = true;
        } else if arg.starts_with("--date=") {
            let date_str = &arg[7..];
            match parse_date_string(date_str) {
                Some(epoch) => time_source = TimeSource::Epoch(epoch),
                None => {
                    if date_str.trim() == "now" {
                        time_source = TimeSource::Now;
                    } else {
                        libsyscall::write(2, b"touch: invalid date '");
                        libsyscall::write(2, date_str.as_bytes());
                        libsyscall::write(2, b"'\n");
                        return 1;
                    }
                }
            }
        } else if arg.starts_with("--reference=") {
            let ref_path = &arg[12..];
            time_source = TimeSource::Reference(String::from(ref_path));
        } else if arg.starts_with('-') && arg.len() > 1 {
            // Parse combined flags like -ac or -t202501061530
            let chars: Vec<char> = arg.chars().skip(1).collect();
            let mut i = 0;
            while i < chars.len() {
                match chars[i] {
                    'a' => atime_only = true,
                    'c' => no_create = true,
                    'm' => mtime_only = true,
                    't' => {
                        // Rest of arg is timestamp
                        let rest: String = chars[i + 1..].iter().collect();
                        if rest.is_empty() {
                            expect_t_arg = true;
                        } else {
                            match parse_touch_timestamp(&rest) {
                                Some(epoch) => time_source = TimeSource::Epoch(epoch),
                                None => {
                                    libsyscall::write(2, b"touch: invalid date format '");
                                    libsyscall::write(2, rest.as_bytes());
                                    libsyscall::write(2, b"'\n");
                                    return 1;
                                }
                            }
                        }
                        break;
                    }
                    'd' => {
                        let rest: String = chars[i + 1..].iter().collect();
                        if rest.is_empty() {
                            expect_d_arg = true;
                        } else {
                            match parse_date_string(&rest) {
                                Some(epoch) => time_source = TimeSource::Epoch(epoch),
                                None => time_source = TimeSource::Now,
                            }
                        }
                        break;
                    }
                    'r' => {
                        let rest: String = chars[i + 1..].iter().collect();
                        if rest.is_empty() {
                            expect_r_arg = true;
                        } else {
                            time_source = TimeSource::Reference(rest);
                        }
                        break;
                    }
                    _ => {
                        libsyscall::write(2, b"touch: invalid option -- '");
                        libsyscall::write(2, &[chars[i] as u8]);
                        libsyscall::write(2, b"'\n");
                        return 1;
                    }
                }
                i += 1;
            }
        } else {
            files.push(String::from(arg));
        }
    }

    // Check for missing arguments
    if expect_t_arg {
        libsyscall::write(2, b"touch: option requires an argument -- 't'\n");
        return 1;
    }
    if expect_d_arg {
        libsyscall::write(2, b"touch: option requires an argument -- 'd'\n");
        return 1;
    }
    if expect_r_arg {
        libsyscall::write(2, b"touch: option requires an argument -- 'r'\n");
        return 1;
    }

    if files.is_empty() {
        libsyscall::write(2, b"touch: missing file operand\n");
        libsyscall::write(2, b"Try 'touch --help' for more information.\n");
        return 1;
    }

    let mut exit_code = 0;
    for file in &files {
        if !touch_file(file, no_create, atime_only, mtime_only, &time_source) {
            exit_code = 1;
        }
    }

    exit_code
}
