# Phase 1: Discovery — Eyra Integration Prerequisites

**TEAM_349** | Eyra Integration Planning  
**Created:** 2026-01-09

---

## 1. Feature Summary

### Problem Statement
LevitateOS currently uses a custom `ulib` library for userspace programs. To run production Rust applications (like `uutils-levbox`), we need full Rust `std` support.

### Solution
Integrate **Eyra** — a pure Rust runtime that implements `std` by making Linux syscalls directly, without a C libc. Since LevitateOS already implements Linux-compatible syscalls, Eyra is the ideal path.

### Who Benefits
- **End users**: Can run standard Rust binaries
- **Developers**: Can use familiar `std` APIs instead of custom `ulib`
- **Ecosystem**: Opens door to `uutils`, `ripgrep`, and other Rust tools

---

## 2. Success Criteria

- [ ] A "Hello, World!" program using Rust `std` boots and runs on LevitateOS
- [ ] `std::thread::spawn` creates threads successfully
- [ ] `std::fs::read` / `std::fs::write` work with tmpfs
- [ ] `std::env::args()` returns correct arguments
- [ ] `std::time::Instant::now()` works
- [ ] `println!` and `eprintln!` output to console

---

## 3. What is Eyra?

Eyra is a package from **sunfishcode** that enables building Rust programs entirely in Rust:

| Component | Purpose |
|-----------|---------|
| **Origin** | Program startup (`_start`), thread creation, TLS setup |
| **rustix** | Pure Rust syscall wrappers (no libc) |
| **c-gull** | ABI-compatible libc function implementations |
| **linux-raw-sys** | Linux constants and struct definitions |

### Key Properties
- Works on **Nightly Rust** only
- Supports **x86-64, x86, aarch64, riscv64**
- Makes **direct syscalls** to Linux kernel
- **Static linking only** (no dynamic linking support)

---

## 4. Current State Analysis

### What LevitateOS Already Has ✅

Based on TEAM_217-239 and TEAM_345 work:

| Category | Syscall | Status |
|----------|---------|--------|
| **Runtime** | argc/argv/envp stack | ✅ Complete |
| **Runtime** | Auxiliary vector (auxv) | ✅ Complete (TEAM_217) |
| **Memory** | mmap/munmap | ✅ Complete (TEAM_228/238) |
| **Memory** | mprotect | ✅ Complete (TEAM_239) |
| **Memory** | brk/sbrk | ✅ Complete |
| **Threading** | clone (CLONE_VM threads) | ✅ Complete (TEAM_230) |
| **Threading** | TLS (TPIDR_EL0) | ✅ Complete |
| **Threading** | set_tid_address | ✅ Complete (TEAM_228) |
| **Threading** | futex (WAIT/WAKE) | ✅ Complete (TEAM_208) |
| **Signals** | kill, sigaction, sigprocmask | ✅ Complete |
| **Signals** | sigreturn | ✅ Complete |
| **I/O** | read/write/writev/readv | ✅ Complete |
| **I/O** | openat, close | ✅ Complete |
| **I/O** | pipe2 | ✅ Complete (TEAM_233) |
| **I/O** | dup/dup3 | ✅ Complete (TEAM_233) |
| **I/O** | ioctl (TTY) | ✅ Complete |
| **FS** | fstat, getdents64 | ✅ Complete |
| **FS** | getcwd | ✅ Complete |
| **Time** | clock_gettime, nanosleep | ✅ Complete |

### What May Be Missing or Need Verification ⚠️

| Category | Syscall/Feature | Status | Notes |
|----------|-----------------|--------|-------|
| **Process** | exit_group | ⚠️ Unknown | Required for clean thread group exit |
| **Process** | arch_prctl (x86_64) | ⚠️ Unknown | TLS on x86_64 |
| **Memory** | madvise | ⚠️ Unknown | Optional but used by allocators |
| **I/O** | poll/ppoll | ❌ Missing | Required for async I/O |
| **I/O** | eventfd2 | ⚠️ Unknown | Used by some async runtimes |
| **FS** | readlink | ⚠️ Verify | /proc/self/exe resolution |
| **FS** | access/faccessat | ⚠️ Unknown | File permission checks |
| **Misc** | getrandom | ⚠️ Unknown | Required for random number generation |
| **Misc** | getuid/geteuid/getgid/getegid | ⚠️ Unknown | User/group IDs |

---

## 5. Eyra's Syscall Requirements

Based on Origin and rustix source code, Eyra requires these syscalls at minimum:

### P0 — Program Startup
| Syscall | Nr (aarch64) | Purpose |
|---------|--------------|---------|
| `write` | 64 | Console output |
| `exit_group` | 94 | Program termination |
| `mmap` | 222 | Memory allocation |
| `munmap` | 215 | Memory deallocation |
| `mprotect` | 226 | Guard pages |
| `brk` | 214 | Heap |

### P1 — Threading
| Syscall | Nr (aarch64) | Purpose |
|---------|--------------|---------|
| `clone` | 220 | Thread creation |
| `set_tid_address` | 96 | TID management |
| `futex` | 98 | Synchronization |
| `exit` | 93 | Thread exit |

### P2 — Filesystem
| Syscall | Nr (aarch64) | Purpose |
|---------|--------------|---------|
| `openat` | 56 | Open files |
| `close` | 57 | Close files |
| `read` | 63 | Read data |
| `write` | 64 | Write data |
| `fstat` | 80 | File metadata |
| `lseek` | 62 | Seek position |

### P3 — Time & Random
| Syscall | Nr (aarch64) | Purpose |
|---------|--------------|---------|
| `clock_gettime` | 113 | Get time |
| `nanosleep` | 101 | Sleep |
| `getrandom` | 278 | Random bytes |

---

## 6. Codebase Reconnaissance

### Files Likely Touched

| Area | Files |
|------|-------|
| Syscall dispatch | `crates/kernel/src/syscall/mod.rs` |
| Memory syscalls | `crates/kernel/src/syscall/mm.rs` |
| Process syscalls | `crates/kernel/src/syscall/process.rs` |
| New syscalls | May need `crates/kernel/src/syscall/misc.rs` |
| ABI definitions | `crates/abi/src/lib.rs` |

### Tests to Verify

| Test | Purpose |
|------|---------|
| `cargo xtask test --arch aarch64` | Golden log verification |
| `cargo xtask test --arch x86_64` | x86_64 behavior |
| Manual: Run Eyra hello-world | End-to-end validation |

---

## 7. Constraints

1. **Nightly Rust Required**: Eyra uses `-Zbuild-std` which requires nightly
2. **Static Linking Only**: Eyra doesn't support dynamic linking
3. **Linux ABI**: Must match Linux syscall numbers and struct layouts exactly
4. **No /proc filesystem**: LevitateOS doesn't have procfs yet (may affect some features)

---

## 8. Next Steps

1. **Phase 2**: Design — Audit each syscall Eyra needs, verify implementation
2. **Phase 3**: Implementation — Add any missing syscalls
3. **Phase 4**: Integration — Build and test Eyra hello-world
4. **Phase 5**: Validation — Run std test suite
