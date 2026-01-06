# Phase 1: Discovery — Spawn Argument Passing

**TEAM_186** | 2026-01-06

## Feature Summary

Enable user processes to pass command-line arguments (`argv`) and environment variables (`envp`) when spawning new processes via the `sys_spawn` syscall.

### Problem Statement

Currently, when the shell runs `cat file.txt`, it spawns `/cat` but the `cat` process receives `argc=0` and no access to `"file.txt"`. This blocks all levbox that need arguments.

### Who Benefits

- Users running any command with arguments
- All Phase 11 levbox (cat, ls, cp, mv, etc.)
- Future shell scripting capabilities

## Success Criteria

1. `cat /hello.txt` prints the file contents (argv[1] = "/hello.txt")
2. `cat` without args reads from stdin (argc = 1, only program name)
3. Shell can parse command lines and pass arguments to kernel
4. Environment variables can be passed (optional/future)

## Current State Analysis

### What Works Today

1. **Kernel infrastructure for args** — ALREADY EXISTS:
   - [`spawn_from_elf_with_args(elf_data, args, envs)`](file:///home/vince/Projects/LevitateOS/kernel/src/task/process.rs#L59) accepts `&[&str]` arguments
   - [`setup_stack_args(ttbr0, stack_top, args, envs)`](file:///home/vince/Projects/LevitateOS/kernel/src/task/user_mm.rs#L196) writes argc/argv/envp to user stack following Linux ABI
   - This was implemented by **TEAM_169** for future use

2. **User-side arg parsing** — ALREADY EXISTS:
   - [`ulib::env::init_args(sp)`](file:///home/vince/Projects/LevitateOS/userspace/ulib/src/env.rs#L43) parses argc/argv from stack
   - [`ulib::env::args()`](file:///home/vince/Projects/LevitateOS/userspace/ulib/src/env.rs#L95) returns iterator over arguments
   - `cat.rs` already calls these functions

### What's Missing (The Gap)

1. **Syscall layer gap**: 
   - [`sys_spawn(path_ptr, path_len)`](file:///home/vince/Projects/LevitateOS/kernel/src/syscall/process.rs#L24) only takes path, no argv
   - It internally calls `spawn_from_elf(elf_data)` without args

2. **libsyscall gap**:
   - [`spawn(path)`](file:///home/vince/Projects/LevitateOS/userspace/libsyscall/src/lib.rs#L156) only takes path, no argv

3. **Shell gap**:
   - Shell parses command but only passes first word to spawn
   - Needs to split command line and pass all args

## Codebase Reconnaissance

### Code Areas to Modify

| File | Change |
|------|--------|
| **Kernel** | |
| [`kernel/src/syscall/mod.rs`](file:///home/vince/Projects/LevitateOS/kernel/src/syscall/mod.rs) | Add `SpawnArgs` syscall number (15) |
| [`kernel/src/syscall/process.rs`](file:///home/vince/Projects/LevitateOS/kernel/src/syscall/process.rs) | Add `sys_spawn_args()` function |
| **Userspace** | |
| [`userspace/libsyscall/src/lib.rs`](file:///home/vince/Projects/LevitateOS/userspace/libsyscall/src/lib.rs) | Add `spawn_args()` wrapper |
| [`userspace/shell/src/main.rs`](file:///home/vince/Projects/LevitateOS/userspace/shell/src/main.rs) | Parse args and call `spawn_args()` |

### Syscall ABI Design Challenge

The tricky part is **how to pass a variable-length array of strings** through a syscall. Options:

1. **Linux execve style**: `execve(path, argv, envp)` where argv/envp are null-terminated arrays of char pointers
2. **Packed buffer**: Single buffer with length-prefixed strings
3. **Limit to a few args**: Fixed number of arg slots (simpler but limited)

## Constraints

### ABI Compatibility
- Must follow Linux ABI for stack layout (argc, argv[], NULL, envp[], NULL)
- Already implemented in `setup_stack_args`

### Memory Safety
- Argv pointers are in user space — kernel must validate each pointer
- Similar pattern to existing `validate_user_buffer` usage

### Performance
- Argument copying happens once at spawn time
- Not a hot path, simplicity over performance

## Existing Tests Impacted

- Behavior test may need update to test `cat /hello.txt`
- No existing tests for argument passing

## Summary

> [!TIP]
> **Good news**: 90% of the infrastructure already exists!
> 
> The kernel can already set up argc/argv on the stack. We just need to:
> 1. Add a syscall that accepts arguments
> 2. Update libsyscall to call it
> 3. Update shell to parse and pass arguments

## Questions

No blocking discovery questions — the architecture is clear.
