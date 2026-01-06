# Userspace Runtime Abstraction Plan

## Complete Process Lifecycle APIs (Linux/POSIX)

A complete userspace runtime must handle **all** of these, not just `_start`:

### 1. Process Startup
| Function | Description | Priority |
|----------|-------------|----------|
| `_start` | Entry point, captures SP, calls `__libc_start_main` | **Critical** |
| `__libc_start_main` | Initializes libc, calls `main(argc, argv, envp)` | **Critical** |
| `_init` | Runs global constructors (C++ static init, `__attribute__((constructor))`) | Medium |
| `atexit()` | Registers functions to run at exit | High |
| `on_exit()` | Like atexit but with status/arg (GNU extension) | Low |

### 2. Process Termination
| Function | Description | Priority |
|----------|-------------|----------|
| `exit()` | Runs atexit handlers, flushes stdio, calls `_exit` | **Critical** |
| `_exit()` / `_Exit()` | Immediate termination (syscall), no cleanup | **Critical** |
| `_fini` | Runs global destructors | Medium |
| `abort()` | Terminate with SIGABRT (core dump) | Medium |

### 3. Signal Handling
| Signal | Default | Description | Priority |
|--------|---------|-------------|----------|
| `SIGTERM` | Term | Graceful termination request | **Critical** |
| `SIGKILL` | Term | Forced kill (cannot be caught) | Kernel-only |
| `SIGINT` | Term | Interrupt from keyboard (Ctrl+C) | High |
| `SIGQUIT` | Core | Quit from keyboard (Ctrl+\) | Medium |
| `SIGSTOP` | Stop | Pause process (cannot be caught) | Kernel-only |
| `SIGCONT` | Cont | Resume stopped process | Kernel-only |
| `SIGTSTP` | Stop | Stop typed at terminal (Ctrl+Z) | Medium |
| `SIGSEGV` | Core | Invalid memory access | High |
| `SIGCHLD` | Ign | Child process terminated | High |
| `SIGPIPE` | Term | Write to pipe with no reader | Medium |
| `SIGALRM` | Term | Timer alarm | Medium |
| `SIGUSR1/2` | Term | User-defined signals | Low |

**Signal APIs:**
| Function | Description | Priority |
|----------|-------------|----------|
| `signal()` | Simple signal handler (deprecated) | Low |
| `sigaction()` | Full signal handler with options | High |
| `sigprocmask()` | Block/unblock signals | Medium |
| `kill()` | Send signal to process | High |
| `raise()` | Send signal to self | Medium |
| `pause()` | Wait for any signal | Low |
| `sigsuspend()` | Atomically unblock and wait | Low |

### 4. Process Control
| Function | Description | Priority |
|----------|-------------|----------|
| `fork()` | Create child process | High |
| `exec*()` | Replace process image | High |
| `wait()/waitpid()` | Wait for child termination | **Critical** |
| `getpid()/getppid()` | Get process/parent ID | **Critical** |
| `sleep()/nanosleep()` | Suspend execution | Medium |
| `sched_yield()` | Voluntarily yield CPU | Low |

### 5. What LevitateOS Currently Has

| Feature | Status |
|---------|--------|
| `_start` entry | ✅ Manual per-app (needs abstraction) |
| `exit()` syscall | ✅ Works |
| `argc/argv` parsing | ✅ Works (with naked _start fix) |
| `spawn()` (like fork+exec) | ✅ Works |
| `waitpid()` | ✅ Works |
| `getpid()` | ✅ Works |
| Signal handling | ❌ Not implemented |
| `atexit()` handlers | ❌ Not implemented |
| `_init`/`_fini` | ❌ Not implemented |
| `fork()` | ❌ Not implemented (spawn only) |

---

## Problem

Currently each LevitateOS app must manually handle low-level entry:
```rust
#[unsafe(naked)]
pub extern "C" fn _start() -> ! {
    core::arch::naked_asm!("mov x0, sp", "b {main}", main = sym main_with_sp)
}

extern "C" fn main_with_sp(sp: *const usize) -> ! {
    unsafe { ulib::env::init_args(sp); }
    // ... actual app logic
}
```

This violates separation of concerns - apps shouldn't deal with ABI details.

## Research: Linux ABI (musl libc)

### musl's Approach (Linux Standard)

**1. Architecture-specific assembly (`crt_arch.h`)**
```asm
// aarch64
_start:
  mov x29, #0        // Clear frame pointer (end of stack trace)
  mov x30, #0        // Clear link register
  mov x0, sp         // Pass SP as first argument
  and sp, x0, #-16   // Align SP to 16 bytes
  b _start_c         // Tail call to C entry
```

**2. Generic C entry (`crt1.c`)**
```c
void _start_c(long *p) {
    int argc = p[0];
    char **argv = (void *)(p+1);
    __libc_start_main(main, argc, argv, _init, _fini, 0);
}
```

**3. User code** - just defines `main(int argc, char **argv)`

### Stack Layout at Entry (Linux ABI)
```
SP+0:   argc (usize)
SP+8:   argv[0] (pointer)
SP+16:  argv[1] (pointer)
...
SP+N:   NULL (argv terminator)
SP+N+8: envp[0] (pointer)
...
SP+M:   NULL (envp terminator)
SP+M+8: auxv entries...
```

## Proposed LevitateOS Design

### Option A: ulib Provides _start (Recommended)

**`ulib/src/entry.rs`:**
```rust
#[unsafe(naked)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "mov x29, #0",      // Clear frame pointer
        "mov x30, #0",      // Clear link register
        "mov x0, sp",       // Pass SP as first argument
        "and sp, x0, #-16", // Align SP
        "b {entry}",
        entry = sym _start_rust,
    )
}

extern "C" fn _start_rust(sp: *const usize) -> ! {
    // Initialize args/env from stack
    unsafe { crate::env::init_args(sp); }
    
    // Call user's main()
    extern "Rust" {
        fn main() -> i32;
    }
    let exit_code = unsafe { main() };
    
    libsyscall::exit(exit_code);
}
```

**User app (`cat.rs`):**
```rust
#![no_std]
#![no_main]
extern crate ulib; // This provides _start

fn main() -> i32 {
    let args = ulib::env::args();
    if args.iter().any(|a| a == "--help") {
        println!("Usage: cat [FILE]...");
        return 0;
    }
    // ... actual logic
    0
}
```

### Option B: Separate crt0 Crate

Create `userspace/crt0/` that's automatically linked first.

### Comparison

| Aspect | Option A (ulib) | Option B (crt0) |
|--------|-----------------|-----------------|
| Simplicity | ✅ Single dependency | ❌ Extra crate |
| Linux-like | ⚠️ Different structure | ✅ More similar |
| Flexibility | ⚠️ Tied to ulib | ✅ Swappable |

## Implementation Steps

1. [ ] Add `entry.rs` to ulib with naked `_start`
2. [ ] Export `_start` symbol from ulib
3. [ ] Update linker script to ensure ulib's _start is entry
4. [ ] Migrate cat.rs to use simple `fn main() -> i32`
5. [ ] Migrate other levbox binaries
6. [ ] Add `#[panic_handler]` to ulib (currently in each app)
7. [ ] Document the pattern

## References

- musl libc: https://git.musl-libc.org/cgit/musl/tree/crt/crt1.c
- musl aarch64: https://git.musl-libc.org/cgit/musl/tree/arch/aarch64/crt_arch.h
- Linux ABI: https://refspecs.linuxfoundation.org/elf/gabi4+/ch5.intro.html
