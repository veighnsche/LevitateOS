# TEAM_186: Spawn Argument Passing Feature

**Created:** 2026-01-06  
**Phase:** Phase 11 Enhancement  
**Status:** Discovery

## Objective

Enable the kernel's `sys_spawn` syscall to pass command-line arguments (`argv`) and environment variables (`envp`) to spawned processes, allowing external commands like `cat file.txt` to work correctly.

## Problem Statement

Currently, the shell can spawn external commands via `sys_spawn`, but the spawned process doesn't receive any arguments. This blocks all coreutils that need command-line arguments (cat, ls, cp, etc.).

## References

- **Roadmap:** Phase 11 requires coreutils with argument support
- **Existing code:** `kernel/src/task/process.rs` has `spawn_from_elf_with_args`
- **Spec:** [`docs/specs/userspace-abi.md`](file:///home/vince/Projects/LevitateOS/docs/specs/userspace-abi.md)

## Progress Log

- [x] Phase 1: Discovery  
- [x] Phase 2: Design  
- [x] Phase 3: Implementation  
- [ ] Phase 4: Integration & Testing  
- [ ] Phase 5: Polish & Cleanup

## Handoff Notes

**Implementation Complete:**

1. **Kernel**: Added `SYS_SPAWN_ARGS` (15) syscall with full argv parsing
2. **libsyscall**: Added `ArgvEntry` struct and `spawn_args()` wrapper
3. **Shell**: Added `split_args()` and integrated argument passing

**Testing Needed:**
- Boot and run `cat /hello.txt`
- Verify stdin mode with just `cat`
