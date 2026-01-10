# TEAM_394: Brush Shell Syscall Gap Analysis

## Overview

This document analyzes syscalls required by brush shell (via nix, tokio, libc crates) and compares against LevitateOS kernel implementation.

## Brush Dependencies Analysis

**Key crates and their syscall requirements:**
- **tokio** (1.48.0) - rt, rt-multi-thread, process, signal, sync, io-util
- **nix** (0.30.1) - fs, poll, process, resource, signal, term, user
- **libc** (0.2.178) - direct syscall wrappers

---

## Syscall Status Matrix

### ✅ Already Implemented

| Syscall | x86_64 | aarch64 | Used By |
|---------|--------|---------|---------|
| read | 0 | 63 | tokio I/O |
| write | 1 | 64 | tokio I/O |
| close | 3 | 57 | nix::unistd |
| fstat | 5 | 80 | nix::sys::stat |
| poll/ppoll | 271 | 73 | nix::poll |
| pipe2 | 293 | 59 | tokio process |
| dup/dup3 | 32/292 | 23/24 | nix::unistd |
| clone | 56 | 220 | tokio threads |
| execve | 59 | 221 | nix::process |
| wait4/waitpid | 61 | 260 | nix::sys::wait |
| kill | 62 | 129 | nix::sys::signal |
| getpid | 39 | 172 | nix::unistd |
| getppid | 110 | 173 | nix::unistd |
| sigaction | 13 | 134 | nix::sys::signal |
| sigprocmask | 14 | 135 | nix::sys::signal |
| ioctl | 16 | 29 | nix::sys::termios, TIOCSCTTY |
| nanosleep | 35 | 101 | tokio time |
| clock_gettime | 228 | 113 | tokio time |
| faccessat | 269 | 48 | nix::unistd::access |
| **epoll_create1** | 291 | 20 | tokio runtime (TEAM_394) |
| **epoll_ctl** | 233 | 21 | tokio runtime (TEAM_394) |
| **epoll_wait** | 232 | 22 | tokio runtime (TEAM_394) |
| **eventfd2** | 290 | 19 | tokio sync (TEAM_394) |

### ❌ Missing - Required for brush

| Syscall | x86_64 | aarch64 | Used By | Priority |
|---------|--------|---------|---------|----------|
| setpgid | 109 | 154 | nix::unistd::setpgid | HIGH |
| getpgid | 121 | 155 | nix::unistd::getpgid | HIGH |
| getpgrp | 111 | 155 | nix::unistd::getpgrp | HIGH |
| setsid | 112 | 157 | nix::unistd::setsid | HIGH |
| tcgetpgrp | (via ioctl) | (via ioctl) | nix::unistd::tcgetpgrp | HIGH |
| tcsetpgrp | (via ioctl) | (via ioctl) | nix::unistd::tcsetpgrp | HIGH |
| getrusage | 98 | 165 | nix::sys::resource | MEDIUM |
| fcntl | 72 | 25 | nix::fcntl (F_SETPIPE_SZ) | MEDIUM |
| ttyname (via readlink /proc/self/fd/N) | - | - | nix::unistd::ttyname | LOW |

### ⚠️ Partially Implemented

| Syscall | Status | Notes |
|---------|--------|-------|
| ioctl | Basic | Need TIOCSCTTY for controlling terminal |
| waitid | via wait4 | brush uses waitid for extended status |

---

## Implementation Priority

### Phase 0 (DONE - TEAM_394)
- [x] epoll_create1, epoll_ctl, epoll_wait
- [x] eventfd2

### Phase 0.5 (Required for brush)
- [ ] setpgid/getpgid/getpgrp - Process group management
- [ ] setsid - Session leader
- [ ] tcsetpgrp/tcgetpgrp - Terminal foreground process group (via ioctl TIOCSPGRP/TIOCGPGRP)
- [ ] fcntl (F_SETPIPE_SZ) - Pipe buffer size

### Phase 0.6 (Nice to have)
- [ ] getrusage - Resource usage statistics
- [ ] waitid - Extended wait with more status info

---

## Implementation Notes

### Process Groups (setpgid/getpgid/setsid)
These are essential for job control in brush:
- `setpgid(pid, pgid)` - Set process group ID
- `getpgid(pid)` - Get process group ID
- `getpgrp()` - Get calling process's group (= getpgid(0))
- `setsid()` - Create new session, become session leader

### Terminal Control (tcsetpgrp/tcgetpgrp)
Implemented via ioctl:
- `TIOCGPGRP` (0x540f) - Get foreground process group
- `TIOCSPGRP` (0x5410) - Set foreground process group

These are needed for proper job control (fg/bg commands).

### fcntl
brush uses `F_SETPIPE_SZ` to set pipe buffer size. Can stub initially.

---

## Recommended Next Steps

1. **Implement process group syscalls** (setpgid, getpgid, getpgrp, setsid)
2. **Add TIOCSPGRP/TIOCGPGRP ioctls** for terminal control
3. **Add fcntl stub** (return success for F_SETPIPE_SZ)
4. **Test with brush** - Try to compile and run

---

## Implementation Notes (TEAM_394)

### Gotchas Discovered

1. **TaskControlBlock fields must be added in 3 places:**
   - `task/mod.rs` - TaskControlBlock struct definition
   - `task/mod.rs` - Default impl
   - `task/thread.rs` - create_thread() TCB initialization
   
   Missing any of these causes compile errors about missing fields.

2. **FdType exhaustiveness:** When adding new FdType variants (e.g., Epoll, EventFd), you must update ALL match expressions:
   - `syscall/fs/stat.rs` - sys_fstat
   - `syscall/fs/statx.rs` - statx_from_fd
   - `syscall/sync.rs` - poll_fd_type
   - Any other files matching on FdType

3. **Eyra build target:** Eyra workspace defaults to wrong target. Always specify:
   ```bash
   cargo build -p <package> --release --target x86_64-unknown-linux-gnu
   ```

4. **Timer functions:** The kernel doesn't have `crate::time::ticks()`. Use:
   ```rust
   crate::arch::time::read_timer_counter()
   crate::arch::time::read_timer_frequency()
   ```

5. **Pattern matching in Rust 2024:** Can't use `ref` in implicitly-borrowing patterns:
   ```rust
   // Wrong:
   FdType::EventFd(ref efd) => { ... }
   // Correct:
   FdType::EventFd(efd) => { ... }
   ```

### Verification Commands

```bash
# Build kernel for both architectures
cargo xtask build kernel --arch x86_64
cargo xtask build kernel --arch aarch64

# Build Eyra userspace binary
cd crates/userspace/eyra
cargo build -p <package> --release --target x86_64-unknown-linux-gnu

# Check binary format
file target/x86_64-unknown-linux-gnu/release/<binary>
# Should show: ELF 64-bit LSB pie executable, static-pie linked
```

## References

- Linux syscall tables: https://github.com/torvalds/linux/blob/master/arch/x86/entry/syscalls/syscall_64.tbl
- nix crate: https://docs.rs/nix/latest/nix/
- brush source: https://github.com/reubeno/brush
