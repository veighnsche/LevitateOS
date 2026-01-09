#![no_std]
#![no_main]

extern crate ulib; // TEAM_231: Required for _start entry point
use core::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use libsyscall::println;
use ulib::exit;

// TEAM_228: Clone flags
const CLONE_VM: u64 = 0x00000100;
const CLONE_THREAD: u64 = 0x00010000;
const CLONE_SIGHAND: u64 = 0x00000800; // Not used yet but good to have
const CLONE_PARENT_SETTID: u64 = 0x00100000;
const CLONE_CHILD_CLEARTID: u64 = 0x00200000;
const CLONE_CHILD_SETTID: u64 = 0x01000000;

// TEAM_228: Stack size for thread
const STACK_SIZE: usize = 4096 * 4;

// TEAM_230: Align stack to 16 bytes (AArch64 requirement)
#[repr(C, align(16))]
struct ThreadStack {
    data: [u8; STACK_SIZE],
}

static mut THREAD_STACK: ThreadStack = ThreadStack {
    data: [0; STACK_SIZE],
};

static SHARED_VAR: AtomicI32 = AtomicI32::new(0);
static CHILD_TID: AtomicU32 = AtomicU32::new(0);

#[no_mangle]
pub fn main() -> i32 {
    println!("[clone_test] Starting...");

    // 1. Verify shared memory writes
    SHARED_VAR.store(1, Ordering::SeqCst);

    println!("[clone_test] Creating thread...");

    let child_stack_top = unsafe { THREAD_STACK.data.as_ptr() as usize + STACK_SIZE };

    // TEAM_230: We use CLONE_VM | CLONE_THREAD | CLONE_CHILD_CLEARTID | CLONE_CHILD_SETTID
    // to match pthread behavior (kind of).
    let flags = CLONE_VM | CLONE_THREAD | CLONE_CHILD_CLEARTID | CLONE_CHILD_SETTID;

    let parent_tid = core::ptr::null_mut(); // Not using PARENT_SETTID
    let tls = 0; // Not using TLS yet

    // We pass &CHILD_TID as child_tid argument.
    // The kernel will write the TID there (SETTID) and clear it on exit (CLEARTID).
    let ctid_ptr = &CHILD_TID as *const _ as *mut i32;

    let res = unsafe { libsyscall::clone(flags, child_stack_top, parent_tid, tls, ctid_ptr) };

    if res < 0 {
        println!("[clone_test] FAILED: clone returned {}", res);
        return 1;
    }

    if res == 0 {
        // Child thread execution starts here
        thread_job();
        exit(0); // Should never return
    }

    // Parent continues here
    let child_tid = res as u32;
    println!("[clone_test] Child TID: {}", child_tid);

    // 2. Wait for child to exit
    // We used CLONE_CHILD_CLEARTID, so child will clear memory at ctid_ptr and wake futex.
    // We wait until the value becomes 0.
    println!("[clone_test] Waiting for child via futex...");

    loop {
        let current_val = CHILD_TID.load(Ordering::SeqCst);
        if current_val == 0 {
            break;
        }
        // Wait for it to change from current_val
        // FUTEX_WAIT = 0
        // Cast ctid_ptr to usize for sys_futex
        let uaddr = ctid_ptr as usize;
        let ret = unsafe { libsyscall::sys_futex(uaddr, 0, current_val, 0, 0, 0) };
        if ret < 0 && ret != -11 { // -11 is EAGAIN, retry
             // println!("[clone_test] Warning: futex wait returned {}", ret);
        }
    }

    // 3. Verify shared memory modification
    let final_val = SHARED_VAR.load(Ordering::SeqCst);
    if final_val == 42 {
        println!("[clone_test] PASS: Shared state verified (42)");
        0
    } else {
        println!(
            "[clone_test] FAILED: Shared state is {} (expected 42)",
            final_val
        );
        1
    }
}

fn thread_job() {
    // Modify shared variable
    SHARED_VAR.store(42, Ordering::SeqCst);

    // Demonstrate we can print (syscall)
    println!("[clone_test] Hello from child thread!");

    // Exit
    // Handled by caller
}
