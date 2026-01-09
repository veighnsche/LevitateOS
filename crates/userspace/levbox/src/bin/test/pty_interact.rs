//! pty_interact.rs
//! Manual interactive test for PTYs.
//! Proxy between console (stdin/stdout) and a PTY master.

#![no_std]
#![no_main]

extern crate ulib;
use libsyscall::{clone, ioctl, isatty, openat, println, read, write};
use ulib::exit;

// termios constants (matches kernel/src/fs/tty/mod.rs)
const TCGETS: u64 = 0x5401;
const TCSETS: u64 = 0x5402;
const ECHO: u32 = 0x08;
const ICANON: u32 = 0x02;

// PTY constants
const TIOCGPTN: u64 = 0x80045430;
const TIOCSPTLCK: u64 = 0x40045431;

// Clone constants
const CLONE_VM: u64 = 0x00000100;
const CLONE_THREAD: u64 = 0x00010000;

const STACK_SIZE: usize = 4096 * 4;
#[repr(C, align(16))]
struct ThreadStack {
    data: [u8; STACK_SIZE],
}

static mut THREAD_STACK: ThreadStack = ThreadStack {
    data: [0; STACK_SIZE],
};

#[no_mangle]
pub fn main() -> i32 {
    println!("PTY Interactive Demo");
    println!("--------------------");

    // 1. Open PTY Master
    let master_fd = openat("/dev/ptmx", 2); // O_RDWR
    if master_fd < 0 {
        println!("Error: Failed to open /dev/ptmx");
        return 1;
    }

    // 2. Get PTY number
    let mut pty_num: u32 = 0;
    if ioctl(
        master_fd as usize,
        TIOCGPTN,
        &mut pty_num as *mut _ as usize,
    ) != 0
    {
        println!("Error: TIOCGPTN failed");
        return 1;
    }
    println!("Allocated /dev/pts/{}", pty_num);

    // 3. Unlock PTY
    let lock: u32 = 0;
    if ioctl(master_fd as usize, TIOCSPTLCK, &lock as *const _ as usize) != 0 {
        println!("Error: TIOCSPTLCK failed");
        return 1;
    }

    // 4. Open Slave for ourselves (to see echoing)
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
    let slave_fd = openat(slave_path_str, 2);
    if slave_fd < 0 {
        println!("Error: Failed to open slave {}", slave_path_str);
        return 1;
    }
    println!("Opened slave PTY. ECHO and ICANON are ON by default.");
    println!("Type anything below. It will be sent to MASTER, echoed by SLAVE,");
    println!("and then read back from MASTER. Use Ctrl+C to exit.");
    println!("--------------------");

    // 5. Proxy Threads
    // Thread 1: Read from Master and print to Console
    let child_stack_top = unsafe { THREAD_STACK.data.as_ptr() as usize + STACK_SIZE };
    let res = unsafe {
        clone(
            CLONE_VM | CLONE_THREAD,
            child_stack_top,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        )
    };

    if res < 0 {
        println!("Error: clone failed {}", res);
        return 1;
    }

    if res == 0 {
        // Master -> Console thread
        let mut buf = [0u8; 128];
        loop {
            let n = read(master_fd as usize, &mut buf);
            if n > 0 {
                // We use a prefix to show 'hardware' output (from master)
                write(1, b"[MASTER OUT] ");
                write(1, &buf[..n as usize]);
            }
        }
    }

    // Main thread: Console -> Master thread
    // Note: Console has its own line discipline, so we'll get lines here.
    let mut buf = [0u8; 128];
    loop {
        let n = read(0, &mut buf);
        if n > 0 {
            write(master_fd as usize, &buf[..n as usize]);
        }
    }
}
