# TEAM_233 — Feature: Process Orchestration (pipe2, dup)

## Purpose
Implement Phase 6 of std-support: `sys_pipe2`, `sys_dup`, and `sys_dup3` for process I/O redirection and piping.

## Status: Implementation Complete ✅

## Context
- Phase 4 Threading: ✅ Complete (TEAM_230)
- Phase 6 Pipe/Dup: ✅ Implementation Complete

## Implementation Summary
- **Kernel**:
  - `kernel/src/fs/pipe.rs` — RingBuffer and Pipe struct with read/write
  - `kernel/src/task/fd_table.rs` — Added PipeRead/PipeWrite variants, dup/dup_to methods
  - `kernel/src/syscall/fs/fd.rs` — sys_dup, sys_dup3, sys_pipe2 handlers
  - `kernel/src/syscall/fs/read.rs` — Pipe read handling
  - `kernel/src/syscall/fs/write.rs` — Pipe write handling
  - `kernel/src/syscall/fs/stat.rs` — FIFO mode for pipes
  - `kernel/src/syscall/mod.rs` — Syscall numbers Dup=23, Dup3=24, Pipe2=59
- **Userspace**:
  - `userspace/libsyscall/src/lib.rs` — pipe2, dup, dup2, dup3 wrappers
