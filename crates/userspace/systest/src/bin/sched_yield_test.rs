//! TEAM_253: Test sched_yield
#![no_std]
#![no_main]

extern crate ulib;
use ulib::libsyscall::{println, sched_yield};

#[no_mangle]
pub fn main() -> i32 {
    println!("[sched_yield_test] Starting...");

    // Yield loop
    for i in 0..5 {
        println!("[sched_yield_test] Yielding iteration {}", i);
        sched_yield();
    }

    println!("[sched_yield_test] PASS");
    0
}
