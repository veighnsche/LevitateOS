# TEAM_404: Syscall Audit - Linux x86_64 vs LevitateOS

## Common x86_64 Syscalls (0-100) needed by coreutils

| # | Name | Kernel Has? | Notes |
|---|------|-------------|-------|
| 0 | read | ✅ | |
| 1 | write | ✅ | |
| 2 | open | ❌ | Use openat(257) |
| 3 | close | ✅ | |
| 4 | stat | ❌ | Use fstat/statx |
| 5 | fstat | ✅ | |
| 6 | lstat | ❌ | MISSING |
| 7 | poll | ❌ | MISSING (has ppoll=271) |
| 8 | lseek | ❌ | MISSING - critical! |
| 9 | mmap | ✅ | |
| 10 | mprotect | ✅ | |
| 11 | munmap | ✅ | |
| 12 | brk | ✅ | (as Sbrk) |
| 13 | rt_sigaction | ✅ | |
| 14 | rt_sigprocmask | ✅ | |
| 15 | rt_sigreturn | ✅ | |
| 16 | ioctl | ✅ | |
| 17 | pread64 | ❌ | MISSING |
| 18 | pwrite64 | ❌ | MISSING |
| 19 | readv | ✅ | |
| 20 | writev | ✅ | |
| 21 | access | ❌ | Use faccessat(269) |
| 22 | pipe | ✅ | (mapped to Pipe2) |
| 23 | select | ❌ | MISSING |
| 24 | sched_yield | ✅ | |
| 25-31 | mremap/msync/etc | ❌ | MISSING |
| 32 | dup | ✅ | |
| 33 | dup2 | ❌ | MISSING - use dup3? |
| 34 | pause | ✅ | |
| 35 | nanosleep | ✅ | |
| 36-38 | alarm/etc | ❌ | MISSING |
| 39 | getpid | ✅ | |
| 40-55 | various | ❌ | MISSING |
| 56 | clone | ✅ | |
| 57 | fork | ❌ | MISSING |
| 58 | vfork | ❌ | MISSING |
| 59 | execve | ✅ | |
| 60 | exit | ✅ | |
| 61 | wait4 | ✅ | |
| 62 | kill | ✅ | |
| 63 | uname | ❌ | MISSING - needed! |
| 72 | fcntl | ✅ | |
| 77 | ftruncate | ❌ | MISSING |
| 78 | getdents | ✅ | |
| 79 | getcwd | ✅ | |
| 80 | chdir | ❌ | MISSING - needed! |
| 81 | fchdir | ❌ | MISSING |
| 82 | rename | ❌ | Use renameat |
| 83 | mkdir | ❌ | Use mkdirat |
| 84 | rmdir | ❌ | Use unlinkat |
| 85-100 | various | ❌ | MISSING |

## Critical Missing Syscalls for Coreutils

1. **lseek (8)** - File positioning, critical for many utilities
2. **dup2 (33)** - Standard dup, map to dup3
3. **uname (63)** - System info, needed by many utilities
4. **chdir (80)** - Change directory, critical for cd
5. **fchdir (81)** - Change directory by fd
6. **ftruncate (77)** - Truncate file
7. **pread64/pwrite64 (17/18)** - Positioned read/write
8. **lstat (6)** - Stat without following symlinks
9. **getdents64 (217)** - Modern directory reading

## Action Items

Add these syscall number mappings to make coreutils work.
