# TEAM_349 — Eyra Integration Planning

**Created:** 2026-01-09
**Status:** Planning Phase

## Objective

Create a comprehensive plan for integrating Eyra (pure Rust `std` runtime) with LevitateOS.

## Context

Eyra is a pure Rust implementation of `std` that makes syscalls directly to Linux without a C libc. Since LevitateOS implements Linux-compatible syscalls, Eyra is an ideal path to `std` support.

## Planning Documents

- `docs/planning/eyra-integration/phase-1.md` — Discovery
- `docs/planning/eyra-integration/phase-2.md` — Design & Prerequisites

## Key Dependencies

- Origin (program/thread startup)
- rustix (syscall wrappers)
- c-gull (libc ABI compatibility)
- linux-raw-sys (constants/structs)

## Progress

- [x] Research Eyra requirements
- [x] Review existing std-support documentation
- [x] Create phase-1.md (Discovery)
- [x] Create phase-2.md (Design)
- [x] Identify prerequisites and gaps
- [x] Create questions file

## Summary of Prerequisites

### Must Implement (10 items, ~12h total)
1. `exit_group` — Critical for clean shutdown
2. `getrandom` — Critical for std (HashMap, rand)
3. `gettid` — Thread ID
4. `getuid/geteuid/getgid/getegid` — Can return 0
5. `arch_prctl` — x86_64 TLS (critical for x86_64)
6. `tgkill` — Thread-directed signals
7. `faccessat` — File access checks
8. `clock_getres` — Clock resolution
9. `madvise` — Stub (allocator hints)
10. Verify `fcntl` coverage

### Already Complete ✅
- mmap/munmap/mprotect
- clone (threads) + TLS
- futex, set_tid_address
- All basic I/O (read/write/openat/close)
- pipe2, dup/dup3
- Signals (kill, sigaction, sigprocmask)
- Time (clock_gettime, nanosleep)

### Open Questions: 7 (see docs/questions/TEAM_349_eyra_integration.md)
