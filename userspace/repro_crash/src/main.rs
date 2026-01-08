#![no_std]
#![no_main]

use libsyscall::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // We can't use println! easily without a buffer, but sys_write is what we are testing.
    // Let's try to print a banner first to verify we are running.
    println!("REPRO: Starting crash test...");

    // 1. Try to write from an unmapped address
    // 0x4000_0000 is likely unmapped in userspace (heap is small)
    // or just use a random high pointer that is valid "user space" range check
    // (< 0x8000_0000_0000) but definitely not mapped.
    let bad_ptr = 0x1234_5678 as *const u8;

    println!(
        "REPRO: Attempting sys_write from unmapped ptr {:p}",
        bad_ptr
    );

    // This calls write(1, bad_ptr, 1) to stdout
    // Kernel will try to read from bad_ptr to print it.
    // If validation fails (as expected), kernel will crash or kill us (if implemented).
    // Current behavior: Kernel Panic (Data Abort in EL1).
    // Note: write() wrapper takes &[u8], so we need to fake a slice.
    let bad_slice = unsafe { core::slice::from_raw_parts(bad_ptr, 1) };
    let ret = write(1, bad_slice);

    println!("REPRO: Survived! Ret = {}", ret);
    exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit(1);
}
