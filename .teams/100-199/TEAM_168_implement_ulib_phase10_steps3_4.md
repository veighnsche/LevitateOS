# TEAM_168: Implement ulib Phase 10 Steps 3-4

## Objective
Implement file syscalls (openat, close, fstat) and File abstraction.

## Context
- **Plan:** `docs/planning/ulib-phase10/phase-3.md`
- **Previous:** TEAM_166 (Steps 1-2), reviewed by TEAM_167
- **Scope:** Steps 3-4 (file syscalls + File abstraction)

## Implementation Progress

### Step 3: Kernel File Syscalls ✅
- [x] UoW 1: Create FdTable infrastructure
  - Created `kernel/src/task/fd_table.rs`
  - FdTable with stdin/stdout/stderr pre-populated
  - Support for initramfs file entries
  - Added to TaskControlBlock
- [x] UoW 2: Implement openat/close syscalls
  - Added syscall numbers 9 (openat), 10 (close), 11 (fstat)
  - Implemented sys_openat - opens initramfs files
  - Implemented sys_close - closes fds (protects 0/1/2)
- [x] UoW 3: Implement fstat syscall
  - Implemented sys_fstat - returns file size and type
  - Defined Stat structure

### Step 4: ulib File Abstractions ✅
- [x] UoW 1: Create io module
  - Created `userspace/ulib/src/io.rs`
  - ErrorKind enum with errno conversion
  - Error type with Display impl
  - Read and Write traits
- [x] UoW 2: Create File abstraction
  - Created `userspace/ulib/src/fs.rs`
  - File struct with open(), metadata(), as_raw_fd()
  - Metadata struct with len(), is_file()
  - Auto-close on Drop

### Syscall Wrappers (libsyscall) ✅
- [x] Added openat(), close(), fstat() wrappers
- [x] Added Stat struct for fstat

## Status
- COMPLETE

## Files Modified

### Kernel
- `kernel/src/task/fd_table.rs` — NEW: FdTable infrastructure
- `kernel/src/task/mod.rs` — Added fd_table module, FdTable to TCB
- `kernel/src/syscall.rs` — Added openat, close, fstat syscalls

### Userspace
- `userspace/libsyscall/src/lib.rs` — Added syscall wrappers
- `userspace/ulib/src/lib.rs` — Added fs, io modules
- `userspace/ulib/src/io.rs` — NEW: I/O abstractions
- `userspace/ulib/src/fs.rs` — NEW: File abstraction

## Known Limitations
1. File::read() returns NotImplemented - requires kernel-side read position tracking
2. No write support (per Q4: initramfs is read-only)

## Handoff Checklist
- [x] Project builds cleanly
- [x] All tests pass
- [x] Team file updated
- [x] Code comments include TEAM_168

## Next Steps (for future teams)
1. Step 5-6: Argument/environment passing
2. Step 7-8: Time syscalls
3. Step 9: Integration demo
4. Implement read() for initramfs files (requires kernel offset tracking)
