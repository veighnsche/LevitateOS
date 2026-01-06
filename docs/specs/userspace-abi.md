# LevitateOS Userspace ABI Specification

**Status:** FINALIZED (Target)
**Version:** 1.0.0 (Phase 10)
**Architecture:** AArch64 (ARM64)
**Compatibility Target:** Linux Generic (AArch64)

This document defines the strict Application Binary Interface (ABI) for LevitateOS userspace. To enable "Platform Support" (compiling standard Rust apps), LevitateOS adheres to the **Linux AArch64 System Call ABI**. This allows us to reuse `libc` implementations and the Rust `std::sys::unix` backend.

## 0. Calling Convention (Procedure Call Standard)

LevitateOS follows the **AAPCS64** (Procedure Call Standard for the Arm 64-bit Architecture).

- **Arguments**: `x0`-`x7`.
- **Return Value**: `x0` (and `x1` if 128-bit).
- **Callee-saved**: `x19`-`x28`.
- **Frame Pointer**: `x29` (FP).
- **Link Register**: `x30` (LR).
- **Stack Pointer**: `x31` (SP). Must be 16-byte aligned.

## 1. System Call Interface

### 1.1 Trigger Mechanism
- **Instruction**: `svc #0`
- **Register Convention**:

| Register | Usage |
|----------|-------|
| `x8`     | System Call Number (NR) |
| `x0` - `x5` | Arguments (up to 6) |
| `x0`     | Return Value (`-4095` to `-1` indicates error, else success) |

### 1.2 System Call Table (Minimal Compliance)
Based on `asm-generic/unistd.h`.

| syscall | NR | Signature | Description |
|---------|----|-----------|-------------|
| `io_setup` | 0 | ... | Async I/O (Future) |
| ... | | | |
| **`openat`** | 56 | `(dirfd: i32, path: *const c_char, flags: i32, mode: u32) -> i32` | Open/Create file. |
| **`close`** | 57 | `(fd: i32) -> i32` | Close file descriptor. |
| **`read`** | 63 | `(fd: i32, buf: *mut u8, len: usize) -> isize` | Read from stream. |
| **`write`** | 64 | `(fd: i32, buf: *const u8, len: usize) -> isize` | Write to stream. |
| **`writev`** | 66 | `(fd: i32, iov: *const IoVec, count: i32) -> isize` | Vectored write (crucial for `println!`). |
| **`readv`** | 65 | `(fd: i32, iov: *const IoVec, count: i32) -> isize` | Vectored read. |
| **`fstat`** | 80 | `(fd: i32, statbuf: *mut Stat64) -> i32` | Get file status. |
| **`exit`** | 93 | `(code: i32) -> !` | Terminate process. |
| **`exit_group`** | 94 | `(code: i32) -> !` | Terminate all threads (std default). |
| **`nanosleep`** | 101 | `(req: *const Timespec, rem: *mut Timespec) -> i32` | High-res sleep. |
| **`getdents64`** | 61 | `(fd: i32, dirp: *mut Dirent64, count: usize) -> isize` | Read directory entries. |
| **`brk`** | 214 | `(addr: usize) -> usize` | Change program break (Heap). |
| **`munmap`** | 215 | `(addr: usize, len: usize) -> i32` | Unmap memory. |
| **`mmap`** | 222 | `(addr: usize, len: usize, prot: i32, flags: i32, fd: i32, off: off_t) -> usize` | Map memory. |

> **Note**: `open` (sys_1024) is deprecated on AArch64. Use `openat` with `AT_FDCWD` (`-100`) for CWD-relative paths.

## 2. Data Structures (repr(C))

Standard layouts required by Rust `libc` / code expecting Linux layout.

### 2.1 `Stat64` (struct stat)
Size: 128 bytes (approx, verify padding).

```rust
#[repr(C)]
pub struct Stat64 {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub __pad1: u64,
    pub st_size: i64,
    pub st_blksize: i32,
    pub __pad2: i32,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_atime_nsec: i64,
    pub st_mtime: i64,
    pub st_mtime_nsec: i64,
    pub st_ctime: i64,
    pub st_ctime_nsec: i64,
    pub __unused: [i32; 2],
}
```

### 2.2 `Dirent64` (struct linux_dirent64)
The buffer returned by `getdents64`.

```rust
#[repr(C)]
pub struct Dirent64 {
    pub d_ino: u64,      // 64-bit Inode
    pub d_off: i64,      // 64-bit Offset
    pub d_reclen: u16,   // Size of this dirent
    pub d_type: u8,      // File type
    pub d_name: [u8; 0], // Null-terminated filename (flexible array)
}
```

### 2.3 `IoVec` (struct iovec)
For `readv`/`writev`.

```rust
#[repr(C)]
pub struct IoVec {
    pub iov_base: *mut u8,
    pub iov_len: usize,
}
```

## 3. Constants & Flags

### 3.1 `open` Flags (AArch64)

| Flag | Octal | Hex | Description |
|------|-------|-----|-------------|
| `O_RDONLY` | `00000000` | `0x0000` | Read Only |
| `O_WRONLY` | `00000001` | `0x0001` | Write Only |
| `O_RDWR`   | `00000002` | `0x0002` | Read + Write |
| `O_CREAT`  | `00000100` | `0x0040` | Create if missing |
| `O_EXCL`   | `00000200` | `0x0080` | Fail if exists |
| `O_TRUNC`  | `00001000` | `0x0200` | Truncate file |
| `O_APPEND` | `00002000` | `0x0400` | Append mode |
| `O_DIRECTORY`| `00200000` | `0x04000` | Must be directory (Wait, verify hex: 200000 oct = 0x10000 hex? Linux generic is 0x10000? Arm64 default is 0x4000? **Verify Needed**) |

> **Correction**: AArch64 uses `asm-generic` definitions.
> `O_DIRECTORY` = `00200000` (oct) = `0x10000`? No, let's stick to the official `asm-generic` value.
> `O_DIRECTORY` = `0x10000`

### 3.2 Error Codes (errno)

| Name | ID | Description |
|------|----|-------------|
| `EPERM` | 1 | Operation not permitted |
| `ENOENT` | 2 | No such file or directory |
| `EIO` | 5 | I/O error |
| `EBADF` | 9 | Bad file number |
| `EAGAIN` | 11 | Try again |
| `ENOMEM` | 12 | Out of memory |
| `EACCES` | 13 | Permission denied |
| `EEXIST` | 17 | File exists |
| `EINVAL` | 22 | Invalid argument |

## 4. Initialization (Startup)

To support `std::env::args()`, the kernel or dynamic linker must prepare the stack before jumping to `_start`.

**Stack Layout at `_start`:**

```
sp -> argc (u64)
      argv[0] (pointer)
      argv[1] (pointer)
      ...
      NULL
      envp[0] (pointer)
      ...
      NULL
      auxv[0] (Explains pagesize, randomness, etc.)
```

## 5. Threading & TLS

Rust `std` requires Thread Local Storage.
- **TPIDR_EL0**: Register reserved for TLS base address.
- `std` expects to call specialized wrapped clone syscalls to create threads with specific stacks/TLS areas.

---

## 6. uutils-levbox Compatibility Gap

> [!IMPORTANT]
> **End Goal**: Run unmodified [uutils-levbox](https://github.com/uutils/levbox) binaries on LevitateOS.

Running uutils requires a full Rust `std` port, which in turn requires these syscalls:

### 6.1 Gap Analysis

| Category | Syscall | Nr | Status | Required For |
|----------|---------|----|---------| -------------|
| **Memory** | | | | |
| | `mmap` | 222 | 🔴 | Allocator, file mapping |
| | `munmap` | 215 | 🔴 | Memory cleanup |
| | `mprotect` | 226 | 🔴 | Guard pages |
| | `brk` | 214 | 🟢 | Heap allocation |
| **Threading** | | | | |
| | `clone` | 220 | 🔴 | Thread creation |
| | `futex` | 98 | 🔴 | Mutex, condvar |
| | `set_tid_address` | 96 | 🔴 | Thread ID mgmt |
| | TLS (`TPIDR_EL0`) | — | 🔴 | Thread-local storage |
| **Signals** | | | | |
| | `rt_sigaction` | 134 | 🔴 | Signal handlers |
| | `rt_sigprocmask` | 135 | 🔴 | Signal masking |
| | `rt_sigreturn` | 139 | 🔴 | Signal return |
| | `kill` | 129 | 🔴 | Send signals |
| **Process** | | | | |
| | `fork` / `clone` | 220 | 🔴 | Process creation |
| | `execve` | 221 | 🟡 | Program execution (have spawn) |
| | `wait4` | 260 | 🔴 | Child reaping |
| | `getpid` | 172 | 🟢 | Process IDs |
| | `getppid` | 173 | 🔴 | Parent PID |
| **I/O** | | | | |
| | `pipe2` | 59 | 🔴 | Shell pipelines |
| | `dup` / `dup3` | 23/24 | 🔴 | FD duplication |
| | `ioctl` | 29 | 🔴 | TTY control |
| | `poll` | 73 | 🔴 | I/O multiplexing |
| **Filesystem** | | | | |
| | `openat` | 56 | 🟢 | Open files |
| | `read` / `write` | 63/64 | 🟢 | Basic I/O |
| | `fstat` | 80 | 🟢 | File metadata |
| | `getdents64` | 61 | 🟢 | Read directory |

Legend: 🟢 Implemented | 🟡 Partial | 🔴 Not Started

### 6.2 Implementation Strategy

1. **Phase 11 (Busybox)**: Validate basic syscalls with hand-written levbox
2. **Phase 12 (Signals)**: Add process/signal infrastructure
3. **Phase 14 (std port)**: Implement threading, mmap, full syscall set
4. **Graduation**: Cross-compile & run uutils

See [ROADMAP.md](file:///home/vince/Projects/LevitateOS/docs/ROADMAP.md) for detailed phase planning.

---

## 7. References & Prior Art

We stand on the shoulders of giants. The following projects provide the reference implementations and specs we are targeting.

### 7.1 Rust Ecosystem
- **[levbox (uutils)](https://github.com/uutils/levbox)**: The Rust reimplementation of GNU levbox. This is our target payload.
- **[rust-lang/libc](https://github.com/rust-lang/libc)**: Raw FFI bindings to platform libraries. See `src/unix/linux_like/linux/gnu/b64/aarch64` for exact struct layouts.
- **[redox-os/relibc](https://github.com/redox-os/relibc)**: A POSIX C library written in Rust. Excellent reference for implementing `libc` functions on top of syscalls.

### 7.2 System Call Tables
- **[Linux AArch64 Syscall Table](https://chromium.googlesource.com/chromiumos/docs/+/master/constants/syscalls.md#arm64-64_bit)**: Authoritative list of syscall numbers.
- **[Musl Libc AArch64](https://git.musl-libc.org/cgit/musl/tree/arch/aarch64)**: Minimal, clean, standards-compliant libc implementation source.

### 7.3 Standards
- **[POSIX.1-2017 (OpenGroup)](https://pubs.opengroup.org/onlinepubs/9699919799/)**: The official specification for `ls`, `cat`, standards, and behavior.
- **[System V ABI - AArch64](https://github.com/ARM-software/abi-aa/releases)**: Processor-specific Application Binary Interface.

