//! TEAM_253: Test stat and utimensat
#![no_std]
#![no_main]

extern crate ulib;
use ulib::libsyscall::{
    self, close, fstat, openat, println, unlinkat, utimensat, Stat, Timespec, O_CREAT, O_RDWR,
};

#[no_mangle]
pub fn main() -> i32 {
    println!("[stat_test] Starting...");

    let path = "/tmp/stat_test_file";
    let _ = unlinkat(0, path, 0); // Cleanup

    // Create file
    let fd = openat(path, O_CREAT | O_RDWR);
    if fd < 0 {
        println!("[stat_test] FAIL: openat failed");
        return 1;
    }

    // Check fstat
    let mut st: Stat = unsafe { core::mem::zeroed() };
    if fstat(fd as usize, &mut st) < 0 {
        println!("[stat_test] FAIL: fstat failed");
        return 1;
    }

    if st.st_size != 0 {
        println!("[stat_test] FAIL: new file size != 0");
        return 1;
    }

    // Write something
    libsyscall::write(fd as usize, b"hello");

    if fstat(fd as usize, &mut st) < 0 {
        println!("[stat_test] FAIL: fstat failed");
        return 1;
    }

    if st.st_size != 5 {
        println!("[stat_test] FAIL: file size != 5");
        return 1;
    }

    // Test utimensat
    let ts = [
        Timespec {
            tv_sec: 1000,
            tv_nsec: 0,
        },
        Timespec {
            tv_sec: 2000,
            tv_nsec: 0,
        },
    ];
    if utimensat(0, path, Some(&ts), 0) < 0 {
        println!("[stat_test] FAIL: utimensat failed");
        return 1;
    }

    if fstat(fd as usize, &mut st) < 0 {
        println!("[stat_test] FAIL: fstat 3 failed");
        return 1;
    }

    if st.st_atime != 1000 || st.st_mtime != 2000 {
        println!(
            "[stat_test] FAIL: timestamps mismatch. atime={} mtime={}",
            st.st_atime, st.st_mtime
        );
        return 1;
    }

    close(fd as usize);
    let _ = unlinkat(0, path, 0);

    println!("[stat_test] PASS");
    0
}
