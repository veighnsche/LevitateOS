# Shell Prerequisites Checklist

This document lists ALL OS-level features needed for a POSIX shell (like dash, bash, or brush).
Learned the hard way during TEAM_444/TEAM_445.

**Legend:** ✅ = Implemented | ⚠️ = Partial/Stub | ❌ = Not Implemented

---

## Status Summary

| Category | Status | Notes |
|----------|--------|-------|
| Process Management | ✅ Done | fork, execve, waitpid, exit all working |
| File Descriptors | ✅ Done | open, dup2, pipe, fcntl all working |
| File System | ✅ Done | stat, getcwd, chdir, getdents working |
| Memory | ✅ Done | brk, mmap, munmap, mprotect working |
| Signals | ⚠️ **CRITICAL GAP** | Syscalls work but **DELIVERY NOT IMPLEMENTED** |
| Terminal | ✅ Done | termios, TIOCGPGRP/TIOCSPGRP working |
| Environment | ✅ Done | execve stack setup with argv, envp, auxv |

---

## CRITICAL TODO: Signal Delivery

**⚠️ `check_and_deliver_signals()` in `levitate/src/main.rs:82-86` is a NO-OP!**

Signals can be:
- ✅ Registered via `sigaction()`
- ✅ Sent via `kill()` (sets pending bit)
- ✅ Masked via `sigprocmask()`
- ❌ **NEVER DELIVERED** to userspace handlers

This means Ctrl+C works only because TTY sends signal directly, but custom signal handlers won't run.

---

## 1. Process Management

### 1.1 Process Creation & Execution (CRITICAL)

| Syscall | Status | Notes |
|---------|--------|-------|
| `fork()` / `clone()` | ✅ Done | Full address space copy, CLONE_VM/CLONE_FILES/CLONE_THREAD |
| `execve()` | ✅ Done | argv, envp, auxv stack setup, ELF loading |
| `exit()` / `exit_group()` | ✅ Done | Wakes waiters, closes FDs |
| `waitpid()` / `wait4()` | ✅ Done | Zombie tracking, WNOHANG support |

### 1.2 Process Identity

| Syscall | Status | Notes |
|---------|--------|-------|
| `getpid()` | ✅ Done | |
| `getppid()` | ✅ Done | |
| `getpgid()` / `getpgrp()` | ✅ Done | |
| `setpgid()` | ⚠️ Partial | Works for current process only |
| `setsid()` | ✅ Done | Creates new session |

---

## 2. File Descriptors

### 2.1 Basic I/O (CRITICAL)

| Syscall | Status | Notes |
|---------|--------|-------|
| `open()` / `openat()` | ✅ Done | Both legacy open() and openat() |
| `close()` | ✅ Done | |
| `read()` / `write()` | ✅ Done | |
| `readv()` / `writev()` | ✅ Done | Scatter-gather I/O |
| `lseek()` | ✅ Done | SEEK_SET/CUR/END |
| `pread64()` / `pwrite64()` | ✅ Done | |

### 2.2 File Descriptor Manipulation

| Syscall | Status | Notes |
|---------|--------|-------|
| `dup()` | ✅ Done | |
| `dup2()` / `dup3()` | ✅ Done | |
| `pipe()` / `pipe2()` | ✅ Done | |
| `fcntl()` | ✅ Done | F_DUPFD, F_GETFD/SETFD, F_GETFL/SETFL, F_*PIPE_SZ |

---

## 3. File System

### 3.1 File Information

| Syscall | Status | Notes |
|---------|--------|-------|
| `stat()` / `fstat()` / `lstat()` | ✅ Done | Via fstatat |
| `statx()` | ✅ Done | Extended stat |
| `access()` / `faccessat()` | ⚠️ Stub | Returns success, doesn't check permissions |
| `readlink()` / `readlinkat()` | ✅ Done | |

### 3.2 Directory Operations

| Syscall | Status | Notes |
|---------|--------|-------|
| `getcwd()` | ✅ Done | |
| `chdir()` | ✅ Done | |
| `fchdir()` | ❌ ENOSYS | Not commonly needed |
| `getdents()` / `getdents64()` | ✅ Done | |
| `mkdir()` / `mkdirat()` | ✅ Done | |
| `unlinkat()` | ✅ Done | |
| `renameat()` | ✅ Done | |

---

## 4. Memory Management

| Syscall | Status | Notes |
|---------|--------|-------|
| `brk()` / `sbrk()` | ✅ Done | |
| `mmap()` | ✅ Done | |
| `munmap()` | ✅ Done | |
| `mprotect()` | ✅ Done | |
| `mremap()` | ❌ Not implemented | Not critical for shells |
| `madvise()` | ⚠️ Stub | Returns success |

---

## 5. Signals - **NEEDS WORK**

### 5.1 Signal Syscalls

| Syscall | Status | Notes |
|---------|--------|-------|
| `rt_sigaction()` | ✅ Done | Arch-specific struct parsing, stores handlers |
| `rt_sigprocmask()` | ✅ Done | 32-bit mask (TODO: 64-bit) |
| `kill()` | ✅ Done | Sets pending bit, wakes blocked tasks |
| `tkill()` | ✅ Done | Thread-specific |
| `pause()` | ✅ Done | Blocks until signal |
| `rt_sigreturn()` | ✅ Done | Restores signal frame |
| `rt_sigaltstack()` | ⚠️ Stub | Returns success, not functional |

### 5.2 Signal Delivery - **❌ NOT IMPLEMENTED**

```rust
// levitate/src/main.rs:82-86
pub extern "C" fn check_and_deliver_signals(_frame: &mut SyscallFrame) {
    // TODO(TEAM_422): Implement proper signal delivery
    // For now, this is a no-op placeholder
}
```

**What's missing:**
- ❌ Push signal frame to user stack
- ❌ Redirect PC to signal handler
- ❌ Actually invoke registered handlers
- ❌ Handle SA_SIGINFO, SA_RESTART flags

**Impact:**
- Custom SIGINT handlers won't run
- SIGCHLD for job control won't work properly
- Signal-based communication between processes broken

### 5.3 TTY Signal Generation

| Feature | Status | Notes |
|---------|--------|-------|
| Ctrl+C → SIGINT | ✅ Done | TTY driver calls `signal_foreground_process()` |
| Ctrl+Z → SIGTSTP | ✅ Done | Sets pending bit |
| Ctrl+\\ → SIGQUIT | ✅ Done | Sets pending bit |

**Note:** These work for termination because the kernel checks pending signals and terminates,
but custom handlers registered via `sigaction()` will never be invoked.

---

## 6. Terminal / TTY

### 6.1 Terminal Control

| Operation | Status | Notes |
|-----------|--------|-------|
| `ioctl(TCGETS)` | ✅ Done | Get termios |
| `ioctl(TCSETS/TCSETSW/TCSETSF)` | ✅ Done | Set termios |
| `ioctl(TIOCGPGRP)` | ✅ Done | Get foreground pgrp |
| `ioctl(TIOCSPGRP)` | ✅ Done | Set foreground pgrp |
| `ioctl(TIOCGWINSZ)` | ✅ Done | Returns 80x24 (hardcoded) |
| `ioctl(TIOCSWINSZ)` | ❌ Not implemented | Set window size |
| `ioctl(TIOCSCTTY)` | ⚠️ Stub | Set controlling terminal |
| `isatty()` | ✅ Done | Via TCGETS success |

### 6.2 Termios / Line Discipline

| Feature | Status | Notes |
|---------|--------|-------|
| INITIAL_TERMIOS | ✅ Done | Properly initialized (TEAM_445 fix) |
| ICANON (canonical mode) | ✅ Done | Line buffering works |
| ECHO | ✅ Done | |
| ICRNL (CR→LF) | ✅ Done | |
| ISIG (signal chars) | ✅ Done | Ctrl+C/Z/\\ generate signals |
| Control chars (VINTR, VEOF, etc.) | ✅ Done | All initialized |

### 6.3 PTY Support

| Feature | Status | Notes |
|---------|--------|-------|
| PTY allocation | ✅ Done | Master/slave pairs |
| `ioctl(TIOCGPTN)` | ✅ Done | Get PTY number |
| `ioctl(TIOCSPTLCK)` | ✅ Done | Lock/unlock PTY |

---

## 7. Environment & Arguments

### 7.1 Process Stack Layout

| Feature | Status | Notes |
|---------|--------|-------|
| argc on stack | ✅ Done | |
| argv pointers + strings | ✅ Done | |
| envp pointers + strings | ✅ Done | |
| Auxiliary vector | ✅ Done | AT_PAGESZ, AT_RANDOM, AT_PHDR, etc. |
| AT_RANDOM (16 bytes) | ✅ Done | Required for Rust std |

---

## 8. User & Permissions

| Syscall | Status | Notes |
|---------|--------|-------|
| `getuid()` / `geteuid()` | ✅ Done | Always returns 0 (root) |
| `getgid()` / `getegid()` | ✅ Done | Always returns 0 |
| `setuid()` / `setgid()` | ❌ Not implemented | Single-user OS |
| `getgroups()` | ❌ Not implemented | |

---

## 9. Time

| Syscall | Status | Notes |
|---------|--------|-------|
| `clock_gettime()` | ✅ Done | |
| `clock_getres()` | ✅ Done | |
| `gettimeofday()` | ✅ Done | |
| `nanosleep()` | ✅ Done | |
| `clock_nanosleep()` | ✅ Done | |

---

## 10. Miscellaneous

| Syscall | Status | Notes |
|---------|--------|-------|
| `uname()` | ✅ Done | |
| `umask()` | ✅ Done | |
| `getrandom()` | ✅ Done | |
| `poll()` / `ppoll()` | ✅ Done | |
| `epoll_create1/ctl/wait` | ✅ Done | |
| `eventfd2()` | ✅ Done | |
| `futex()` | ✅ Done | WAIT/WAKE |
| `prlimit64()` | ⚠️ Stub | Returns sensible defaults |
| `getrlimit()` / `setrlimit()` | ⚠️ Stub | |

---

## Remaining TODOs

### Critical (Blocks Job Control)

1. **Signal Delivery** - Implement `check_and_deliver_signals()`:
   - Push signal frame to user stack
   - Save current registers for `sigreturn()`
   - Redirect PC to handler address
   - Handle SA_SIGINFO (3-arg handler)
   - Handle SA_RESTART (restart syscalls)

### Important (Nice to Have)

2. **setpgid for other processes** - Currently only works for self
3. **faccessat** - Actually check permissions
4. **64-bit signal mask** - Currently 32-bit
5. **TIOCSWINSZ** - Set terminal window size

### Low Priority

6. **fchdir** - Change directory by fd
7. **setuid/setgid** - For multi-user support
8. **getgroups** - Supplementary groups
9. **mremap** - Resize mappings
10. **sigaltstack** - Alternate signal stack

---

## Testing Checklist

### Working Now ✅
- [x] `echo $$` prints PID
- [x] Simple command runs: `ls`
- [x] Command with args: `ls -la /`
- [x] Exit status works: `false; echo $?` prints 1
- [x] Backspace works
- [x] Ctrl+U clears line
- [x] Ctrl+D on empty line exits

### Should Work (Not Fully Tested)
- [ ] Output redirect: `echo hi > /tmp/test`
- [ ] Input redirect: `cat < /etc/passwd`
- [ ] Simple pipe: `echo hi | cat`
- [ ] Shebang scripts: `#!/bin/sh`

### Blocked by Signal Delivery ❌
- [ ] Ctrl+C with custom SIGINT handler
- [ ] Ctrl+Z suspends foreground (needs SIGTSTP delivery)
- [ ] `fg` resumes stopped job (needs SIGCONT delivery)
- [ ] `bg` runs stopped job in background
- [ ] `jobs` lists jobs (needs SIGCHLD)
- [ ] Proper job control

---

## References

- POSIX.1-2017 Shell & Utilities: https://pubs.opengroup.org/onlinepubs/9699919799/
- Linux man-pages: https://man7.org/linux/man-pages/
- dash source: https://git.kernel.org/pub/scm/utils/dash/dash.git
- musl libc source: https://git.musl-libc.org/cgit/musl/
