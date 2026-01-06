# TEAM_210: Linux Syscall ABI Migration

**Created:** 2026-01-06
**Objective:** Migrate syscall numbers to Linux AArch64 ABI for std/uutils compatibility

## Status

- [x] Phase 1 - Audit & Plan
- [x] Phase 2 - Update kernel (`SyscallNumber` enum)
- [x] Phase 3 - Update libsyscall (`SYS_*` constants)
- [x] Phase 4 - Build verification (both pass)

## Changes Made

- Updated 18 syscalls to Linux AArch64 numbers
- Custom syscalls (spawn, spawn_args) moved to 1000+ range
- Kernel and userspace both build successfully

## Key Syscall Mappings

| Syscall | Old | New (Linux) |
|---------|-----|-------------|
| read | 0 | 63 |
| write | 1 | 64 |
| exit | 2 | 93 |
| openat | 9 | 56 |
| futex | 41 | 98 |
