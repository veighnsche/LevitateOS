# TEAM_192: Implement levbox Utilities (Phase 11)

**Date**: 2026-01-06
**Status**: 🟡 In Progress

## Objective

Implement the "Busybox" core utilities for LevitateOS as specified in the checklist.

## Tasks

- [/] Enhancing `cat` (Add `--help`, `--version`)
- [ ] Implement `ls` (Basic listing, `-a`, `-1`)
- [ ] Implement `sys_getcwd` syscall in kernel
- [ ] Implement `libsyscall::getcwd` wrapper
- [ ] Implement `pwd` utility
- [ ] Implement `sys_mkdirat` syscall in kernel
- [ ] Implement `libsyscall::mkdirat` wrapper
- [ ] Implement `mkdir` utility

## Progress

### 2026-01-06
- Initialized team log.
- Planning to start with `cat` enhancements.

## Notes

- Some utilities require new syscalls. I will implement them following the Linux ABI.
