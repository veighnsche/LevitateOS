#![no_std]
#![no_main]

//! TEAM_239: mmap_test - Verify mmap/munmap/mprotect syscalls work correctly.

extern crate ulib;
use libsyscall::println;

// TEAM_228: mmap protection flags
const PROT_READ: u32 = 1;
const PROT_WRITE: u32 = 2;

// TEAM_228: mmap flags
const MAP_PRIVATE: u32 = 0x02;
const MAP_ANONYMOUS: u32 = 0x20;

// Page size
const PAGE_SIZE: usize = 4096;

#[no_mangle]
pub fn main() -> i32 {
    println!("[mmap_test] Starting memory syscall tests...");

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Basic mmap of anonymous memory
    println!("[mmap_test] Test 1: Basic anonymous mmap");
    if test_basic_mmap() {
        println!("[mmap_test] Test 1: PASS");
        passed += 1;
    } else {
        println!("[mmap_test] Test 1: FAIL");
        failed += 1;
    }

    // Test 2: Write and read from mmap'd memory
    println!("[mmap_test] Test 2: mmap read/write");
    if test_mmap_readwrite() {
        println!("[mmap_test] Test 2: PASS");
        passed += 1;
    } else {
        println!("[mmap_test] Test 2: FAIL");
        failed += 1;
    }

    // Test 3: munmap frees memory
    println!("[mmap_test] Test 3: munmap");
    if test_munmap() {
        println!("[mmap_test] Test 3: PASS");
        passed += 1;
    } else {
        println!("[mmap_test] Test 3: FAIL");
        failed += 1;
    }

    // Test 4: mprotect changes protection
    println!("[mmap_test] Test 4: mprotect");
    if test_mprotect() {
        println!("[mmap_test] Test 4: PASS");
        passed += 1;
    } else {
        println!("[mmap_test] Test 4: FAIL");
        failed += 1;
    }

    // Test 5: Multi-page allocation
    println!("[mmap_test] Test 5: Multi-page mmap");
    if test_multipage_mmap() {
        println!("[mmap_test] Test 5: PASS");
        passed += 1;
    } else {
        println!("[mmap_test] Test 5: FAIL");
        failed += 1;
    }

    println!("[mmap_test] Results: {} passed, {} failed", passed, failed);

    if failed == 0 {
        println!("[mmap_test] All tests passed!");
        0
    } else {
        println!("[mmap_test] Some tests failed!");
        1
    }
}

/// Test basic anonymous mmap returns a valid address
fn test_basic_mmap() -> bool {
    let addr = libsyscall::mmap(
        0,                           // hint addr
        PAGE_SIZE,                   // 1 page
        PROT_READ | PROT_WRITE,      // read/write
        MAP_PRIVATE | MAP_ANONYMOUS, // private, anonymous
        -1,                          // no fd
        0,                           // no offset
    );

    if addr < 0 {
        println!("[mmap_test]   mmap failed with error: {}", addr);
        return false;
    }

    // Should be page-aligned
    if (addr as usize) & (PAGE_SIZE - 1) != 0 {
        println!(
            "[mmap_test]   mmap returned non-aligned address: 0x{:x}",
            addr
        );
        return false;
    }

    // Clean up
    let res = libsyscall::munmap(addr as usize, PAGE_SIZE);
    if res < 0 {
        println!("[mmap_test]   munmap failed with error: {}", res);
        return false;
    }

    true
}

/// Test that we can write to and read from mmap'd memory
fn test_mmap_readwrite() -> bool {
    let addr = libsyscall::mmap(
        0,
        PAGE_SIZE,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );

    if addr < 0 {
        println!("[mmap_test]   mmap failed with error: {}", addr);
        return false;
    }

    // Write a pattern
    let ptr = addr as *mut u8;
    for i in 0..256 {
        unsafe {
            *ptr.add(i) = i as u8;
        }
    }

    // Read it back
    for i in 0..256 {
        let val = unsafe { *ptr.add(i) };
        if val != i as u8 {
            println!(
                "[mmap_test]   mismatch at offset {}: got {} expected {}",
                i, val, i
            );
            return false;
        }
    }

    // Clean up
    libsyscall::munmap(addr as usize, PAGE_SIZE);
    true
}

/// Test that munmap actually frees memory
fn test_munmap() -> bool {
    let addr = libsyscall::mmap(
        0,
        PAGE_SIZE,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );

    if addr < 0 {
        println!("[mmap_test]   mmap failed with error: {}", addr);
        return false;
    }

    // Write to it to make sure it's valid
    let ptr = addr as *mut u64;
    unsafe {
        *ptr = 0xDEADBEEF;
    }

    // Unmap it
    let res = libsyscall::munmap(addr as usize, PAGE_SIZE);
    if res < 0 {
        println!("[mmap_test]   munmap failed with error: {}", res);
        return false;
    }

    // Note: We can't safely test that access fails after munmap
    // because that would cause a fault. Just verify munmap succeeded.
    true
}

/// Test that mprotect can change protection
fn test_mprotect() -> bool {
    let addr = libsyscall::mmap(
        0,
        PAGE_SIZE,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );

    if addr < 0 {
        println!("[mmap_test]   mmap failed with error: {}", addr);
        return false;
    }

    // Write some data first
    let ptr = addr as *mut u64;
    unsafe {
        *ptr = 0x12345678;
    }

    // Change to read-only
    let res = libsyscall::mprotect(addr as usize, PAGE_SIZE, PROT_READ);
    if res < 0 {
        println!("[mmap_test]   mprotect failed with error: {}", res);
        libsyscall::munmap(addr as usize, PAGE_SIZE);
        return false;
    }

    // Verify we can still read data (it wasn't corrupted)
    let val = unsafe { *ptr };
    if val != 0x12345678 {
        println!("[mmap_test]   data corrupted after mprotect: 0x{:x}", val);
        libsyscall::munmap(addr as usize, PAGE_SIZE);
        return false;
    }

    // Note: Writing to a read-only page would cause a fault.
    // We can't easily test this without signal handling.

    // Clean up
    libsyscall::munmap(addr as usize, PAGE_SIZE);
    true
}

/// Test multi-page allocation
fn test_multipage_mmap() -> bool {
    const NUM_PAGES: usize = 4;
    let size = PAGE_SIZE * NUM_PAGES;

    let addr = libsyscall::mmap(
        0,
        size,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );

    if addr < 0 {
        println!("[mmap_test]   mmap failed with error: {}", addr);
        return false;
    }

    // Write a unique value at the start of each page
    let ptr = addr as *mut u64;
    for i in 0..NUM_PAGES {
        let page_ptr = unsafe { ptr.add((i * PAGE_SIZE) / 8) };
        unsafe {
            *page_ptr = (i as u64) * 0x1111_1111;
        }
    }

    // Read them back
    for i in 0..NUM_PAGES {
        let page_ptr = unsafe { ptr.add((i * PAGE_SIZE) / 8) };
        let val = unsafe { *page_ptr };
        let expected = (i as u64) * 0x1111_1111;
        if val != expected {
            println!(
                "[mmap_test]   page {} mismatch: 0x{:x} != 0x{:x}",
                i, val, expected
            );
            libsyscall::munmap(addr as usize, size);
            return false;
        }
    }

    // Clean up
    let res = libsyscall::munmap(addr as usize, size);
    if res < 0 {
        println!("[mmap_test]   munmap failed with error: {}", res);
        return false;
    }

    true
}
