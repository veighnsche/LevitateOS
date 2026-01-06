# TEAM_182: Cat Utility Implementation

**Created:** 2026-01-06  
**Phase:** Phase 11 (Core Utilities)  
**Status:** Planning

## Objective

Implement the `cat` utility as the first utility in Phase 11's "Busybox" Phase levbox suite for LevitateOS.

## References

- **Spec:** [`docs/specs/levbox/cat.md`](file:///home/vince/Projects/LevitateOS/docs/specs/levbox/cat.md)
- **Roadmap:** [`docs/ROADMAP.md`](file:///home/vince/Projects/LevitateOS/docs/ROADMAP.md) - Phase 11

## Discovery Summary

### Available Syscalls (All Implemented)
- `sys_open` / `openat` - Open files for reading ✓
- `sys_read` - Read file contents ✓
- `sys_write` - Write to stdout ✓
- `sys_close` - Close file descriptor ✓

### Available Userspace Libraries
- `ulib::fs::File` - RAII file handle with `Read` trait
- `ulib::io::Read`, `Write` traits
- `ulib::io::BufReader` - Buffered reading
- `libsyscall::read`, `write`, `openat`, `close` - Raw syscalls

### Dependencies
- `libsyscall` - Syscall wrappers
- `ulib` - File abstractions (optional, can use raw syscalls for simplicity)

## Progress Log

- [x] Phase 1: Discovery
- [x] Phase 2: Design
- [x] Phase 3: Implementation
- [x] Phase 4: Integration & Testing
- [ ] Phase 5: Polish & Cleanup

## Handoff Notes

The `cat` utility has been implemented with the following caveats:

1. **No argument passing yet**: The kernel's `sys_spawn` doesn't pass argv to spawned processes. Running `cat file.txt` will spawn `cat` but it won't receive "file.txt" as an argument. A new kernel syscall `sys_spawn_with_args` is needed.

2. **Shell updated**: The shell now tries to spawn external commands from initramfs when builtins don't match.

3. **Testing stdin mode**: Running just `cat` will work (reads from stdin), but `cat <file>` requires argument passing.
