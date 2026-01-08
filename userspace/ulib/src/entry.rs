//! TEAM_212: Process entry point and lifecycle management.
//!
//! This module provides the `_start` entry point that all userspace programs use.
//! It follows the Linux/musl libc ABI for compatibility.
//!
//! ## How it works
//! 1. Kernel loads ELF, sets up stack with argc/argv/envp
//! 2. `_start` (naked asm) captures SP before prologue modifies it
//! 3. `_start_rust` initializes args, runs global constructors, calls `main()`
//! 4. After main returns, runs atexit handlers and exits
//!
//! ## Usage
//! Apps just define `main()` - ulib handles the rest:
//! ```rust
//! #![no_std]
//! #![no_main]
//! extern crate ulib;
//!
//! #[no_mangle]
//! fn main() -> i32 {
//!     // Your code here
//!     0
//! }
//! ```

use core::sync::atomic::{AtomicBool, Ordering};

/// Flag to track if we've already initialized
static INITIALIZED: AtomicBool = AtomicBool::new(false);

// ============================================================================
// Entry Point (Linux ABI Compatible)
// ============================================================================

/// The true entry point - naked function to capture SP before Rust prologue.
///
/// This follows musl libc's aarch64 crt_arch.h:
/// ```asm
/// mov x29, #0       // Clear frame pointer (end of stack trace)
/// mov x30, #0       // Clear link register
/// mov x0, sp        // Pass original SP as first argument
/// and sp, x0, #-16  // Align SP to 16 bytes
/// b _start_rust     // Call Rust entry
/// ```
///
/// Only compiled when the `entry` feature is enabled.
/// Binaries that want ulib to provide _start should enable this feature.
#[cfg(all(target_arch = "aarch64", feature = "entry"))]
#[link_section = ".text._start"]
#[unsafe(naked)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "mov x29, #0",       // Clear frame pointer (end of stack trace)
        "mov x30, #0",       // Clear link register
        "mov x0, sp",        // Pass original SP as first argument
        "and sp, x0, #-16",  // Align SP to 16 bytes
        "b {entry}",         // Tail call to Rust entry
        entry = sym _start_rust,
    )
}

/// TEAM_303: x86_64 entry point.
#[cfg(all(target_arch = "x86_64", feature = "entry"))]
#[link_section = ".text._start"]
#[unsafe(naked)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "xor rbp, rbp",      // Clear frame pointer
        "mov rdi, rsp",      // Pass original SP as first argument
        "and rsp, -16",      // Align stack to 16 bytes
        "call {entry}",      // Call Rust entry point
        "ud2",               // Should not return
        entry = sym _start_rust,
    )
}

/// Rust entry point that receives the original stack pointer.
///
/// Stack layout at entry (Linux ABI):
/// ```text
/// SP+0:   argc (usize)
/// SP+8:   argv[0] (pointer to first arg string)
/// SP+16:  argv[1] ...
/// ...
/// SP+N:   NULL (argv terminator)
/// SP+N+8: envp[0] (pointer to first env string)
/// ...
/// SP+M:   NULL (envp terminator)
/// ```
#[cfg(feature = "entry")]
#[no_mangle]
extern "C" fn _start_rust(sp: *const usize) -> ! {
    // Prevent double initialization
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        libsyscall::exit(1);
    }

    // 1. Initialize arguments from stack
    unsafe {
        crate::env::init_args(sp);
    }

    // 2. Run global constructors (_init)
    // TODO: Call _init if defined
    run_init_array();

    // 3. Call user's main function
    extern "Rust" {
        fn main() -> i32;
    }
    let exit_code = unsafe { main() };

    // 4. Run exit handlers and terminate
    exit(exit_code);
}

// ============================================================================
// Process Termination
// ============================================================================

/// Exit the process, running atexit handlers first.
///
/// This is the proper way to exit - it runs cleanup handlers.
/// For immediate exit without cleanup, use `libsyscall::exit()`.
pub fn exit(code: i32) -> ! {
    // Run atexit handlers in reverse order
    run_atexit_handlers();

    // Run global destructors (_fini)
    run_fini_array();

    // Actually exit
    libsyscall::exit(code)
}

/// Abort the process immediately (like SIGABRT).
///
/// Does NOT run atexit handlers.
pub fn abort() -> ! {
    // TEAM_216: Send SIGABRT to self
    raise(Signal::SIGABRT);

    // In case signal is ignored (shouldn't be for SIGABRT), exit anyway
    libsyscall::exit(134) // 128 + 6 (SIGABRT)
}

// ============================================================================
// atexit Handlers
// ============================================================================

/// Maximum number of atexit handlers
const MAX_ATEXIT_HANDLERS: usize = 32;

/// Registered atexit handlers
static mut ATEXIT_HANDLERS: [Option<fn()>; MAX_ATEXIT_HANDLERS] = [None; MAX_ATEXIT_HANDLERS];
static mut ATEXIT_COUNT: usize = 0;

/// Register a function to be called at exit.
///
/// Returns 0 on success, -1 if no more slots available.
pub fn atexit(handler: fn()) -> i32 {
    unsafe {
        if ATEXIT_COUNT >= MAX_ATEXIT_HANDLERS {
            return -1;
        }
        ATEXIT_HANDLERS[ATEXIT_COUNT] = Some(handler);
        ATEXIT_COUNT += 1;
        0
    }
}

/// Run all registered atexit handlers in reverse order.
fn run_atexit_handlers() {
    unsafe {
        while ATEXIT_COUNT > 0 {
            ATEXIT_COUNT -= 1;
            if let Some(handler) = ATEXIT_HANDLERS[ATEXIT_COUNT].take() {
                handler();
            }
        }
    }
}

// ============================================================================
// Global Constructors/Destructors (Stubs)
// ============================================================================

/// Run global constructors from .init_array section.
fn run_init_array() {
    extern "C" {
        static __init_array_start: usize;
        static __init_array_end: usize;
    }

    unsafe {
        let mut f = &__init_array_start as *const usize;
        while f < &__init_array_end as *const usize {
            let func: fn() = core::mem::transmute(*f);
            func();
            f = f.add(1);
        }
    }
}

/// Run global destructors from .fini_array section.
fn run_fini_array() {
    extern "C" {
        static __fini_array_start: usize;
        static __fini_array_end: usize;
    }

    unsafe {
        let mut f = &__fini_array_start as *const usize;
        while f < &__fini_array_end as *const usize {
            let func: fn() = core::mem::transmute(*f);
            func();
            f = f.add(1);
        }
    }
}

// ============================================================================
// Signal Handling (Stubs)
// ============================================================================

/// Signal numbers (Linux aarch64)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    SIGHUP = 1,
    SIGINT = 2,
    SIGQUIT = 3,
    SIGILL = 4,
    SIGTRAP = 5,
    SIGABRT = 6,
    SIGBUS = 7,
    SIGFPE = 8,
    SIGKILL = 9,
    SIGUSR1 = 10,
    SIGSEGV = 11,
    SIGUSR2 = 12,
    SIGPIPE = 13,
    SIGALRM = 14,
    SIGTERM = 15,
    SIGCHLD = 17,
    SIGCONT = 18,
    SIGSTOP = 19,
    SIGTSTP = 20,
    SIGTTIN = 21,
    SIGTTOU = 22,
}

/// Signal handler function type
pub type SignalHandler = extern "C" fn(i32);

/// Default signal handler (does nothing)
pub const SIG_DFL: Option<SignalHandler> = None;

/// Ignore signal handler
pub const SIG_IGN: Option<SignalHandler> = None;

/// Register a signal handler.
pub fn signal(sig: Signal, handler: Option<SignalHandler>) -> Option<SignalHandler> {
    let handler_addr = match handler {
        Some(f) => f as usize,
        None => 0,
    };

    // TEAM_216: Call sigaction with the handler and our default trampoline
    libsyscall::sigaction(
        sig as i32,
        handler_addr,
        sigreturn_trampoline as *const () as usize,
    );
    None
}

/// Send a signal to a process.
pub fn kill(pid: i32, sig: Signal) -> i32 {
    libsyscall::kill(pid, sig as i32) as i32
}

/// Send a signal to self.
pub fn raise(sig: Signal) -> i32 {
    let pid = libsyscall::getpid() as i32;
    libsyscall::kill(pid, sig as i32) as i32
}

/// Wait for a signal.
pub fn pause() -> i32 {
    libsyscall::pause() as i32
}

/// TEAM_216: Signal trampoline called by the kernel after handler returns.
/// Registered in sigaction.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigreturn_trampoline() {
    libsyscall::sigreturn();
}

// ============================================================================
// Process Control (Stubs)
// ============================================================================

/// Sleep for the specified number of seconds.
///
/// STUB: Not yet fully implemented.
pub fn sleep(seconds: u32) -> u32 {
    // Use nanosleep - pass seconds and 0 nanoseconds
    if libsyscall::nanosleep(seconds as u64, 0) == 0 {
        0
    } else {
        seconds // Return remaining time on error
    }
}

/// Yield the CPU to other processes.
pub fn sched_yield() {
    libsyscall::yield_cpu();
}
