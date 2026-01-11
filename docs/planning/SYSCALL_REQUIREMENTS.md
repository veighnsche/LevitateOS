# Syscall Requirements for General-Purpose OS

**Created**: 2026-01-10
**Updated**: 2026-01-10 (TEAM_404 audit)
**Status**: Reference Document

This document lists all syscalls required for a general-purpose Unix-compatible OS, organized by epic.

---

## Legend

- ✅ Implemented (fully working)
- 🔨 Stub (mapped but returns ENOSYS or minimal impl)
- ⏳ Planned
- ❌ Not Started

---

## Epic 1: Process Model (TEAM_400)

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| fork | 57 | 1071 (clone) | ⏳ | Clone process |
| vfork | 58 | 1071 (clone) | ⏳ | Lightweight fork |
| clone | 56 | 220 | ✅ | General process creation |
| clone3 | 435 | 435 | ⏳ | Modern clone |
| execve | 59 | 221 | ✅ | Execute program |
| execveat | 322 | 281 | ⏳ | Execute relative to fd |
| wait4 | 61 | 260 | ✅ | Wait for child |
| waitid | 247 | 95 | ⏳ | Wait with options |
| exit | 60 | 93 | ✅ | Exit thread |
| exit_group | 231 | 94 | ✅ | Exit process |
| getpid | 39 | 172 | ✅ | Get process ID |
| getppid | 110 | 173 | ✅ | Get parent PID |
| gettid | 186 | 178 | ✅ | Get thread ID |
| set_tid_address | 218 | 96 | ✅ | Set clear_child_tid |
| prctl | 157 | 167 | ⏳ | Process control |
| sched_yield | 24 | 124 | ✅ | Yield CPU |
| kill | 62 | 129 | ✅ | Send signal |
| tkill | 200 | 130 | ✅ | Send to thread |
| pause | 34 | - | ✅ | Wait for signal |
| getrusage | 98 | 165 | ✅ | Resource usage (zeroed) |
| prlimit64 | 302 | 261 | ✅ | Get/set resource limits |

---

## Epic 2: Filesystem Operations (TEAM_401)

### Core File Operations

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| read | 0 | 63 | ✅ | Read from fd |
| write | 1 | 64 | ✅ | Write to fd |
| open | 2 | - | ✅ | Open file (legacy) |
| openat | 257 | 56 | ✅ | Open file at path |
| close | 3 | 57 | ✅ | Close fd |
| lseek | 8 | 62 | ✅ | Seek in file |
| pread64 | 17 | 67 | ✅ | Positioned read |
| pwrite64 | 18 | 68 | ✅ | Positioned write |
| readv | 19 | 65 | ✅ | Vectored read |
| writev | 20 | 66 | ✅ | Vectored write |
| truncate | 76 | 45 | ✅ | Truncate file by path |
| ftruncate | 77 | 46 | ✅ | Truncate file by fd |

### File Descriptor Operations

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| dup | 32 | 23 | ✅ | Duplicate fd |
| dup2 | 33 | - | ✅ | Duplicate to specific fd |
| dup3 | 292 | 24 | ✅ | Duplicate with flags |
| fcntl | 72 | 25 | ✅ | File control |
| ioctl | 16 | 29 | ✅ | Device control |
| pipe | 22 | - | ✅ | Create pipe (mapped to pipe2) |
| pipe2 | 293 | 59 | ✅ | Create pipe with flags |

### File Metadata

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| stat | 4 | - | ✅ | Get file status (legacy) |
| fstat | 5 | 80 | ✅ | Get status by fd |
| lstat | 6 | - | ✅ | Get symlink status |
| newfstatat | 262 | 79 | ✅ | Get status at path |
| statx | 332 | 291 | ✅ | Extended file status |
| faccessat | 269 | 48 | ✅ | Check file access |
| utimensat | 280 | 88 | ✅ | Update timestamps |

### Directory Operations

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| getcwd | 79 | 17 | ✅ | Get current directory |
| chdir | 80 | 49 | ✅ | Change directory |
| fchdir | 81 | 50 | 🔨 | Change dir by fd (stub) |
| mkdir | 83 | - | ✅ | Create directory (legacy) |
| mkdirat | 258 | 34 | ✅ | Create directory at path |
| rmdir | 84 | - | ✅ | Remove directory |
| getdents64 | 217 | 61 | ✅ | Read directory entries |

### Path Operations

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| unlink | 87 | - | ✅ | Remove file (legacy) |
| unlinkat | 263 | 35 | ✅ | Remove at path |
| rename | 82 | - | ✅ | Rename file (legacy) |
| renameat | 264 | 38 | ✅ | Rename at path |
| link | 86 | - | ✅ | Create hard link (legacy) |
| linkat | 265 | 37 | ✅ | Create hard link at path |
| symlink | 88 | - | ✅ | Create symlink (legacy) |
| symlinkat | 266 | 36 | ✅ | Create symlink at path |
| readlink | 89 | 78 | ✅ | Read symlink (legacy) |
| readlinkat | 267 | 78 | ✅ | Read symlink at path |

### Device Operations

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| mknod | 133 | - | ⏳ | Create device node |
| mknodat | 259 | 33 | ⏳ | Create device at path |

### Mount Operations

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| mount | 165 | 40 | ✅ | Mount filesystem |
| umount2 | 166 | 39 | ✅ | Unmount filesystem |
| pivot_root | 155 | 41 | ⏳ | Change root (TEAM_402) |

---

## Epic 3: Memory Management (TEAM_402)

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| brk | 12 | 214 | ✅ | Adjust heap |
| mmap | 9 | 222 | ✅ | Map memory |
| munmap | 11 | 215 | ✅ | Unmap memory |
| mprotect | 10 | 226 | ✅ | Change protection |
| madvise | 28 | 233 | ✅ | Memory advice |
| pkey_alloc | 330 | 289 | ✅ | Allocate protection key |
| pkey_mprotect | 329 | 288 | ✅ | Protect with key |

---

## Epic 4: Disk Root & Sync (TEAM_403)

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| pivot_root | 155 | 41 | ⏳ | Switch root filesystem |
| chroot | 161 | 51 | ⏳ | Change root directory |
| sync | 162 | 81 | ⏳ | Sync filesystems |
| syncfs | 306 | 267 | ⏳ | Sync one filesystem |
| fsync | 74 | 82 | ⏳ | Sync file |
| fdatasync | 75 | 83 | ⏳ | Sync file data |

---

## Epic 5: Users & Permissions (TEAM_405)

### Identity Query

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| getuid | 102 | 174 | ✅ | Get real UID (returns 0) |
| geteuid | 107 | 175 | ✅ | Get effective UID (returns 0) |
| getgid | 104 | 176 | ✅ | Get real GID (returns 0) |
| getegid | 108 | 177 | ✅ | Get effective GID (returns 0) |
| getresuid | 118 | 148 | ⏳ | Get real/eff/saved UID |
| getresgid | 120 | 150 | ⏳ | Get real/eff/saved GID |
| getgroups | 115 | 80 | ⏳ | Get supplementary groups |

### Identity Change

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| setuid | 105 | 146 | ⏳ | Set UID |
| setgid | 106 | 144 | ⏳ | Set GID |
| setreuid | 113 | 145 | ⏳ | Set real/effective UID |
| setregid | 114 | 143 | ⏳ | Set real/effective GID |
| setresuid | 117 | 147 | ⏳ | Set real/eff/saved UID |
| setresgid | 119 | 149 | ⏳ | Set real/eff/saved GID |
| setgroups | 116 | 81 | ⏳ | Set supplementary groups |
| setfsuid | 122 | 151 | ⏳ | Set filesystem UID |
| setfsgid | 123 | 152 | ⏳ | Set filesystem GID |

### File Permissions

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| chmod | 90 | - | 🔨 | No-op (single-user OS) |
| fchmod | 91 | 52 | 🔨 | No-op (single-user OS) |
| fchmodat | 268 | 53 | 🔨 | No-op (single-user OS) |
| chown | 92 | - | 🔨 | No-op (single-user OS) |
| fchown | 93 | 55 | 🔨 | No-op (single-user OS) |
| fchownat | 260 | 54 | 🔨 | No-op (single-user OS) |
| lchown | 94 | - | ⏳ | Change symlink owner |
| access | 21 | - | ⏳ | Check access |
| faccessat | 269 | 48 | ✅ | Check access at path |
| faccessat2 | 439 | 439 | ⏳ | Check access with flags |
| umask | 95 | 166 | ✅ | Set file creation mask |

---

## Epic 6: Signals (TEAM_406)

### Signal Handling

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| rt_sigaction | 13 | 134 | ✅ | Set signal handler |
| rt_sigprocmask | 14 | 135 | ✅ | Block/unblock signals |
| rt_sigreturn | 15 | 139 | ✅ | Return from handler |
| rt_sigsuspend | 130 | 133 | ⏳ | Wait for signal |
| rt_sigpending | 127 | 136 | ⏳ | Get pending signals |
| rt_sigtimedwait | 128 | 137 | ⏳ | Wait with timeout |
| rt_sigqueueinfo | 129 | 138 | ⏳ | Queue signal |
| sigaltstack | 131 | 132 | ✅ | Set alternate stack |
| kill | 62 | 129 | ✅ | Send signal |
| tgkill | 234 | 131 | ⏳ | Send to thread |
| tkill | 200 | 130 | ✅ | Send to thread (old) |

### Process Groups & Sessions

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| getpgid | 121 | 155 | ✅ | Get process group |
| setpgid | 109 | 154 | ✅ | Set process group |
| getpgrp | 111 | - | ✅ | Get own process group |
| getsid | 124 | 156 | ⏳ | Get session ID |
| setsid | 112 | 157 | ✅ | Create session |

---

## Epic 7: Networking (Future)

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| socket | 41 | 198 | ❌ | Create socket |
| bind | 49 | 200 | ❌ | Bind address |
| listen | 50 | 201 | ❌ | Listen for connections |
| accept | 43 | 202 | ❌ | Accept connection |
| accept4 | 288 | 242 | ❌ | Accept with flags |
| connect | 42 | 203 | ❌ | Connect to server |
| sendto | 44 | 206 | ❌ | Send data |
| recvfrom | 45 | 207 | ❌ | Receive data |
| sendmsg | 46 | 211 | ❌ | Send message |
| recvmsg | 47 | 212 | ❌ | Receive message |
| shutdown | 48 | 210 | ❌ | Shutdown socket |
| setsockopt | 54 | 208 | ❌ | Set socket option |
| getsockopt | 55 | 209 | ❌ | Get socket option |
| getsockname | 51 | 204 | ❌ | Get socket address |
| getpeername | 52 | 205 | ❌ | Get peer address |
| socketpair | 53 | 199 | ❌ | Create socket pair |

---

## Epic 8: Event & Poll (TEAM_394)

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| poll | 7 | - | ✅ | Wait for events |
| ppoll | 271 | 73 | ✅ | Poll with timeout |
| epoll_create1 | 291 | 20 | ✅ | Create epoll instance |
| epoll_ctl | 233 | 21 | ✅ | Control epoll |
| epoll_wait | 232 | 22 | ✅ | Wait for events |
| eventfd2 | 290 | 19 | ✅ | Create event fd |
| futex | 202 | 98 | ✅ | Fast userspace mutex |

---

## Epic 9: Time (TEAM_407)

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| nanosleep | 35 | 101 | ✅ | Sleep |
| clock_gettime | 228 | 113 | ✅ | Get time |
| clock_getres | 229 | 114 | ✅ | Get clock resolution |
| gettimeofday | 96 | 1094 | ✅ | Get time (legacy) |
| clock_nanosleep | 230 | 115 | ✅ | Sleep with clock |

---

## Epic 10: Architecture-Specific

### x86_64

| Syscall | Number | Status | Notes |
|---------|--------|--------|-------|
| arch_prctl | 158 | ✅ | Set/get arch state (FS/GS base) |

### aarch64

| Syscall | Number | Status | Notes |
|---------|--------|--------|-------|
| (set_tls via msr) | - | ✅ | Thread-local storage |

---

## Epic 11: Miscellaneous

| Syscall | x86_64 | aarch64 | Status | Notes |
|---------|--------|---------|--------|-------|
| getrandom | 318 | 278 | ✅ | Get random bytes |
| reboot | 169 | 142 | ✅ | Reboot/shutdown |

---

## LevitateOS Custom Syscalls

These are non-Linux syscalls specific to LevitateOS:

| Syscall | Number | Status | Notes |
|---------|--------|--------|-------|
| spawn | 1000 | ✅ | Spawn process |
| spawn_args | 1001 | ✅ | Spawn with arguments |
| set_foreground | 1002 | ✅ | Set foreground process |
| get_foreground | 1003 | ✅ | Get foreground process |
| isatty | 1010 | ✅ | Check if fd is TTY |

---

## Syscall Count Summary

| Category | Implemented | Stub | Planned | Not Started |
|----------|-------------|------|---------|-------------|
| Epic 1 (Process) | 14 | 0 | 5 | 0 |
| Epic 2 (Filesystem) | 46 | 3 | 2 | 0 |
| Epic 3 (Memory) | 7 | 0 | 0 | 0 |
| Epic 4 (Disk/Sync) | 0 | 0 | 6 | 0 |
| Epic 5 (Users) | 6 | 6 | 13 | 0 |
| Epic 6 (Signals) | 9 | 0 | 6 | 0 |
| Epic 7 (Networking) | 0 | 0 | 0 | 16 |
| Epic 8 (Event/Poll) | 7 | 0 | 0 | 0 |
| Epic 9 (Time) | 5 | 0 | 0 | 0 |
| Custom | 5 | 0 | 0 | 0 |
| **Total** | **~99** | **~9** | **~32** | **~16** |

---

## Critical Path Syscalls

These syscalls are blocking for general-purpose OS:

1. ~~**fork/clone**~~ ✅ Can spawn processes
2. ~~**execve**~~ ✅ Can run programs
3. ~~**wait4**~~ ✅ Can manage children
4. **setuid/setgid** ⏳ Needed for proper users
5. ~~**chmod/chown**~~ 🔨 No-op stubs (sufficient for single-user)
6. **pivot_root** ⏳ Needed for disk root
7. **fsync** ⏳ Needed for data integrity

---

## Next Priority Syscalls

Based on coreutils and shell requirements:

1. **pread64/pwrite64** - Many tools use positioned I/O
2. **ftruncate** - File editing tools need this
3. **fchdir** - Some directory operations
4. **fsync/fdatasync** - Data integrity
5. **chmod/fchmod** - Permission management

---

## Implementation Reference

This section maps syscalls to their kernel implementation files.

### Module: `syscall/process.rs` (24 syscalls)

| Syscall | Status | Notes |
|---------|--------|-------|
| sys_exit | ✅ | Exit thread |
| sys_getpid | ✅ | Get PID |
| sys_getppid | ✅ | Get parent PID |
| sys_gettid | ✅ | Get thread ID |
| sys_spawn | ✅ | LevitateOS custom |
| sys_spawn_args | ✅ | LevitateOS custom |
| sys_exec | ✅ | execve |
| sys_yield | ✅ | sched_yield |
| sys_waitpid | ✅ | wait4 |
| sys_clone | ✅ | Thread/process creation |
| sys_set_tid_address | ✅ | Thread ID address |
| sys_exit_group | ✅ | Exit all threads |
| sys_getuid | ✅ | Returns 0 (root) |
| sys_geteuid | ✅ | Returns 0 (root) |
| sys_getgid | ✅ | Returns 0 (root) |
| sys_getegid | ✅ | Returns 0 (root) |
| sys_arch_prctl | ✅ | x86_64 only |
| sys_setpgid | ✅ | Set process group |
| sys_getpgid | ✅ | Get process group |
| sys_getpgrp | ✅ | Get own process group |
| sys_setsid | ✅ | Create session |
| sys_set_foreground | ✅ | LevitateOS custom |
| sys_get_foreground | ✅ | LevitateOS custom |

### Module: `syscall/fs/` (33 syscalls)

| File | Syscall | Status |
|------|---------|--------|
| fd.rs | sys_dup | ✅ |
| fd.rs | sys_dup2 | ✅ |
| fd.rs | sys_dup3 | ✅ |
| fd.rs | sys_pipe2 | ✅ |
| fd.rs | sys_fcntl | ✅ |
| fd.rs | sys_ioctl | ✅ |
| fd.rs | sys_isatty | ✅ |
| fd.rs | sys_lseek | ✅ |
| fd.rs | sys_chdir | ✅ |
| fd.rs | sys_fchdir | 🔨 Stub |
| fd.rs | sys_ftruncate | 🔨 Stub |
| fd.rs | sys_pread64 | 🔨 Stub |
| fd.rs | sys_pwrite64 | 🔨 Stub |
| dir.rs | sys_getcwd | ✅ |
| dir.rs | sys_getdents | ✅ |
| dir.rs | sys_mkdirat | ✅ |
| dir.rs | sys_renameat | ✅ |
| dir.rs | sys_unlinkat | ✅ |
| link.rs | sys_linkat | ✅ |
| link.rs | sys_readlinkat | ✅ |
| link.rs | sys_symlinkat | ✅ |
| link.rs | sys_utimensat | ✅ |
| open.rs | sys_openat | ✅ |
| open.rs | sys_close | ✅ |
| open.rs | sys_faccessat | ✅ |
| read.rs | sys_read | ✅ |
| read.rs | sys_readv | ✅ |
| write.rs | sys_write | ✅ |
| write.rs | sys_writev | ✅ |
| mount.rs | sys_mount | ✅ |
| mount.rs | sys_umount | ✅ |
| stat.rs | sys_fstat | ✅ |
| statx.rs | sys_statx | ✅ |

### Module: `syscall/mm.rs` (7 syscalls)

| Syscall | Status | Notes |
|---------|--------|-------|
| sys_sbrk | ✅ | brk equivalent |
| sys_mmap | ✅ | Memory mapping |
| sys_munmap | ✅ | Unmap memory |
| sys_mprotect | ✅ | Change protection |
| sys_madvise | ✅ | Memory advice |
| sys_pkey_alloc | ✅ | Protection keys |
| sys_pkey_mprotect | ✅ | Protect with key |

### Module: `syscall/signal.rs` (7 syscalls)

| Syscall | Status | Notes |
|---------|--------|-------|
| sys_kill | ✅ | Send signal |
| sys_pause | ✅ | Wait for signal |
| sys_sigaction | ✅ | Set handler |
| sys_sigreturn | ✅ | Return from handler |
| sys_sigprocmask | ✅ | Block signals |
| sys_tkill | ✅ | Signal to thread |
| sys_sigaltstack | ✅ | Alternate stack |

### Module: `syscall/epoll.rs` (4 syscalls)

| Syscall | Status | Notes |
|---------|--------|-------|
| sys_epoll_create1 | ✅ | Create epoll |
| sys_epoll_ctl | ✅ | Control epoll |
| sys_epoll_wait | ✅ | Wait for events |
| sys_eventfd2 | ✅ | Create eventfd |

### Module: `syscall/sync.rs` (2 syscalls)

| Syscall | Status | Notes |
|---------|--------|-------|
| sys_futex | ✅ | Fast mutex |
| sys_ppoll | ✅ | Poll with timeout |

### Module: `syscall/time.rs` (4 syscalls)

| Syscall | Status | Notes |
|---------|--------|-------|
| sys_nanosleep | ✅ | Sleep |
| sys_clock_nanosleep | ✅ | Sleep with clock |
| sys_clock_getres | ✅ | Clock resolution |
| sys_clock_gettime | ✅ | Get time |

### Module: `syscall/sys.rs` (2 syscalls)

| Syscall | Status | Notes |
|---------|--------|-------|
| sys_shutdown | ✅ | Reboot/shutdown |
| sys_getrandom | ✅ | Random bytes |

---

## Total Implemented: 82 syscall functions

---

## Stub Analysis (TEAM_409)

**Audit Date**: 2026-01-10

This section provides a detailed analysis of syscalls that are mapped but not fully implemented, categorized by type.

### Category 1: Returns ENOSYS (3 syscalls)

These syscalls are mapped to handlers that explicitly return `-ENOSYS`:

| Syscall | Location | Work Required |
|---------|----------|---------------|
| fchdir | syscall/fs/fd.rs | Implement fd-to-path lookup, validate is directory |
| pkey_alloc | syscall/mm.rs | Memory protection keys (hardware-specific) |
| pkey_mprotect | syscall/mm.rs | Memory protection keys (hardware-specific) |

**Note**: `pkey_*` syscalls require Intel MPK or ARM MTE support.
**Note**: `pread64` and `pwrite64` are now fully implemented (TEAM_409).

### Category 2: No-Op by Design (6 syscalls)

These return success but deliberately do nothing (single-user OS, root runs everything):

| Syscall | Location | Current Behavior | Real Implementation |
|---------|----------|------------------|---------------------|
| chmod | syscall/fs/permission.rs | Returns 0 | Store mode in inode |
| fchmod | syscall/fs/permission.rs | Returns 0 | Store mode in inode |
| fchmodat | syscall/fs/permission.rs | Returns 0 | Store mode in inode |
| chown | syscall/fs/permission.rs | Returns 0 | Store uid/gid in inode |
| fchown | syscall/fs/permission.rs | Returns 0 | Store uid/gid in inode |
| fchownat | syscall/fs/permission.rs | Returns 0 | Store uid/gid in inode |

**Rationale**: LevitateOS currently runs as single-user root. Permissions will be needed for Epic 5 (Users & Permissions).

### Category 3: Stubs Returning Success (8 syscalls)

These accept calls and return success, but ignore some or all parameters:

| Syscall | Location | What's Ignored | Work Required |
|---------|----------|----------------|---------------|
| fcntl F_SETFL | syscall/fs/fd.rs | O_NONBLOCK, O_APPEND flags | Hook into VFS file ops |
| fcntl F_SETFD | syscall/fs/fd.rs | FD_CLOEXEC flag | Track in FD table, apply on exec |
| fcntl F_GETFL | syscall/fs/fd.rs | Returns 0 always | Return actual file flags |
| fcntl F_GETFD | syscall/fs/fd.rs | Returns 0 always | Return actual FD flags |
| ioctl TIOCSCTTY | syscall/fs/fd.rs | Controlling terminal | Implement session/ctty tracking |
| ioctl TIOCGWINSZ | syscall/fs/fd.rs | Returns fixed 80x24 | Query real terminal size |
| sigaltstack | syscall/signal.rs | All parameters | Allocate/track alt stack per thread |
| madvise | syscall/mm.rs | All hints | Implement MADV_DONTNEED, WILLNEED |

**Note**: `ftruncate` is now fully implemented (TEAM_410).

### Category 4: Partial Implementations (11 syscalls)

These work for common cases but fail on edge cases:

#### 4a. `*at` syscalls ignore dirfd (9 syscalls)

All `*at` syscalls treat paths as absolute or CWD-relative, ignoring the `dirfd` parameter:

| Syscall | Impact | Fix Required |
|---------|--------|--------------|
| openat | Low (usually AT_FDCWD) | Resolve path relative to dirfd |
| mkdirat | Low | Resolve path relative to dirfd |
| unlinkat | Low | Resolve path relative to dirfd |
| renameat | Medium | Resolve both paths relative to dirfds |
| linkat | Medium | Resolve both paths relative to dirfds |
| symlinkat | Low | Resolve target relative to dirfd |
| readlinkat | Low | Resolve path relative to dirfd |
| faccessat | Low | Resolve path relative to dirfd |
| utimensat | Low | Resolve path relative to dirfd |
| fstatat | Low | Resolve path relative to dirfd |

**Implementation Note**: Need `fd_to_path()` helper or store dentry in FD table.

#### 4b. mmap file-backed mappings

| Limitation | Current Behavior | Fix Required |
|------------|------------------|--------------|
| MAP_PRIVATE file | Returns EINVAL | Implement copy-on-write for file pages |
| MAP_SHARED file | Returns EINVAL | Implement writeback to file |

**Note**: Anonymous mappings (MAP_ANONYMOUS) work correctly.

#### 4c. chdir validation

| Limitation | Current Behavior | Fix Required |
|------------|------------------|--------------|
| Symlink resolution | May not fully resolve | Use VFS resolve_path |
| Existence check | Weak validation | Verify dentry is directory |

### Category 5: Missing syscall handlers

These syscalls have no handler mapped at all:

| Syscall | Priority | Notes |
|---------|----------|-------|
| fork | High | Use clone with CLONE_CHILD |
| vfork | Medium | Use clone with CLONE_VFORK |
| clone3 | Medium | Extended clone interface |
| execveat | Low | execve relative to fd |
| prctl | Medium | Process control ops |

**Note**: `gettimeofday` is now implemented (TEAM_409).

### Summary by Priority

**High Priority** (blocking coreutils/shells):
1. ~~`pread64` / `pwrite64`~~ ✅ Implemented (TEAM_409)
2. ~~`ftruncate`~~ ✅ Implemented (TEAM_410)
3. `fcntl` flags - Proper O_NONBLOCK support
4. `dirfd` support in `*at` syscalls
5. `fchdir` - Change directory by fd

**Medium Priority** (full Unix compat):
1. Permission syscalls (chmod, chown family) - no-op ok for now
2. sigaltstack proper implementation
3. Controlling terminal (TIOCSCTTY)

**Low Priority** (edge cases):
1. Memory protection keys (pkey_*)
2. madvise hints
3. File-backed mmap

### Recently Implemented (TEAM_409)

- `pread64` - Positioned read without changing file offset
- `pwrite64` - Positioned write without changing file offset
- `gettimeofday` - Legacy time syscall (x86_64: 96, aarch64: custom 1094)
- `getrusage` - Resource usage statistics (returns zeroed struct)

---
