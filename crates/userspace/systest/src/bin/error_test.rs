//! TEAM_253: Negative testing
#![no_std]
#![no_main]

extern crate ulib;
use ulib::libsyscall::{openat, println, read, EBADF, ENOENT};

#[no_mangle]
pub fn main() -> i32 {
    println!("[error_test] Starting...");

    // Test 1: Open non-existent
    let ret = openat("/this/does/not/exist", 0);
    if ret != ENOENT as isize {
        println!(
            "[error_test] FAIL: Expected ENOENT ({}), got {}",
            ENOENT, ret
        );
        return 1;
    }

    // Test 2: Read invalid fd
    let mut buf = [0u8; 1];
    let ret = read(9999, &mut buf);
    if ret != EBADF as isize {
        println!("[error_test] FAIL: Expected EBADF ({}), got {}", EBADF, ret);
        return 1;
    }

    println!("[error_test] PASS");
    0
}
