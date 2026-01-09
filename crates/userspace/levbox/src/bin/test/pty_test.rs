//! TEAM_247: PTY Test Suite
//!
//! Verifies PTY master/slave communication, ioctls, and line discipline.

#![no_std]
#![no_main]

extern crate ulib;
use libsyscall::{close, ioctl, isatty, openat, println, read, write};

static mut PASSED: u32 = 0;
static mut FAILED: u32 = 0;

fn test_pass(name: &str) {
    println!("[pty_test] {}: PASS", name);
    unsafe {
        PASSED += 1;
    }
}

fn test_fail(name: &str, reason: &str) {
    println!("[pty_test] {}: FAIL - {}", name, reason);
    unsafe {
        FAILED += 1;
    }
}

const TIOCGPTN: u64 = 0x80045430;
const TIOCSPTLCK: u64 = 0x40045431;

#[no_mangle]
pub fn main() -> i32 {
    println!("[pty_test] Starting PTY tests...");

    // 1. Open /dev/ptmx
    let master_fd = openat("/dev/ptmx", 2); // O_RDWR
    if master_fd < 0 {
        test_fail("open_ptmx", "failed to open /dev/ptmx");
        return 1;
    }
    test_pass("open_ptmx");

    // 2. Get PTY number (TIOCGPTN)
    let mut pty_num: u32 = 999;
    if ioctl(
        master_fd as usize,
        TIOCGPTN,
        &mut pty_num as *mut _ as usize,
    ) != 0
    {
        test_fail("tiocgptn", "ioctl TIOCGPTN failed");
    } else {
        println!("[pty_test] PTY number: {}", pty_num);
        test_pass("tiocgptn");
    }

    // 3. Unlock PTY (TIOCSPTLCK)
    let lock: u32 = 0;
    if ioctl(master_fd as usize, TIOCSPTLCK, &lock as *const _ as usize) != 0 {
        test_fail("tiocsptlck", "ioctl TIOCSPTLCK failed");
    } else {
        test_pass("tiocsptlck");
    }

    // 4. Open Slave PTY
    let mut slave_path = [0u8; 32];
    let path_prefix = b"/dev/pts/";
    slave_path[..path_prefix.len()].copy_from_slice(path_prefix);
    let mut idx = path_prefix.len();
    if pty_num == 0 {
        slave_path[idx] = b'0';
        idx += 1;
    } else {
        let mut n = pty_num;
        let mut digits = [0u8; 10];
        let mut d_idx = 0;
        while n > 0 {
            digits[d_idx] = (n % 10) as u8 + b'0';
            n /= 10;
            d_idx += 1;
        }
        for d in (0..d_idx).rev() {
            slave_path[idx] = digits[d];
            idx += 1;
        }
    }

    let slave_path_str = core::str::from_utf8(&slave_path[..idx]).unwrap();

    let slave_fd = openat(slave_path_str, 2); // O_RDWR
    if slave_fd < 0 {
        test_fail("open_slave", "failed to open slave pty");
    } else {
        test_pass("open_slave");

        // 5. Verify slave is a TTY
        if isatty(slave_fd as i32) == 1 {
            test_pass("slave_isatty");
        } else {
            test_fail("slave_isatty", "slave fd is not reported as a TTY");
        }

        // 6. Test basic communication: Master -> Slave
        let test_str = b"hello pty\n";
        write(master_fd as usize, test_str);

        let mut read_buf = [0u8; 32];
        let n = read(slave_fd as usize, &mut read_buf);
        if n == test_str.len() as isize && &read_buf[..n as usize] == test_str {
            test_pass("master_to_slave");
        } else {
            test_fail("master_to_slave", "data mismatch or read failed");
        }

        // 7. Test basic communication: Slave -> Master
        let test_str_2 = b"echo back\n";
        write(slave_fd as usize, test_str_2);

        let mut read_buf_2 = [0u8; 32];
        let n2 = read(master_fd as usize, &mut read_buf_2);

        if n2 > 0 {
            test_pass("slave_to_master");
        } else {
            test_fail("slave_to_master", "read from master failed");
        }

        close(slave_fd as usize);
    }

    close(master_fd as usize);

    println!(
        "[pty_test] Summary: {} passed, {} failed",
        unsafe { PASSED },
        unsafe { FAILED }
    );
    if unsafe { FAILED } == 0 {
        0
    } else {
        1
    }
}
