//! TEAM_253: Test time syscalls
#![no_std]
#![no_main]

extern crate ulib;
use ulib::libsyscall::{clock_gettime, nanosleep, println, Timespec};

#[no_mangle]
pub fn main() -> i32 {
    println!("[time_test] Starting...");

    let mut start = Timespec { tv_sec: 0, tv_nsec: 0 };
    let mut end = Timespec { tv_sec: 0, tv_nsec: 0 };

    if clock_gettime(&mut start) < 0 {
        println!("[time_test] FAIL: clock_gettime failed");
        return 1;
    }

    // Sleep 100ms
    if nanosleep(0, 100_000_000) < 0 {
        println!("[time_test] FAIL: nanosleep failed");
        return 1;
    }

    if clock_gettime(&mut end) < 0 {
        println!("[time_test] FAIL: clock_gettime 2 failed");
        return 1;
    }

    let elapsed_sec = end.tv_sec - start.tv_sec;
    let elapsed_ns = end.tv_nsec - start.tv_nsec;
    let total_ms = elapsed_sec * 1000 + elapsed_ns / 1_000_000;

    println!("[time_test] Slept for {} ms", total_ms);

    if total_ms < 100 {
        println!("[time_test] FAIL: slept less than requested");
        return 1;
    }

    println!("[time_test] PASS");
    0
}
