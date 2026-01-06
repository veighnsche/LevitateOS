# TEAM_176: Implement Directory Iteration

## Status: Complete ✅

## Objective
Implement the Directory Iteration feature per the plan in `docs/planning/directory-iteration/`.

## Decisions (All Recommendations Accepted)
- **Q1**: Buffer size = 4096 bytes (C)
- **Q2**: Filter out `.` and `..` (B)
- **Q3**: New `NotADirectory` error kind (A)
- **Q4**: Empty dir returns `Ok(ReadDir)` + `None` (B)
- **Q5**: Enhance `CpioArchive` with helpers (A)
- **Q6**: Syscall number = NR 14 (A)
- **Q7**: Entry stores filename only (A)

## Implementation Steps
1. [x] Add `CpioEntryType`, `is_directory()`, `list_directory()` to `CpioArchive` (los_utils)
2. [x] Add `SYS_GETDENTS` (NR 14) syscall to kernel with `sys_getdents` handler
3. [x] Add `getdents()` wrapper, `Dirent64`, `d_type` constants to libsyscall
4. [x] Add `ReadDir`, `DirEntry`, `FileType`, `read_dir()` to ulib/fs.rs

## Files Modified
- `crates/utils/src/cpio.rs` - Added CpioEntryType, mode/ino parsing, list_directory
- `kernel/src/syscall/mod.rs` - Added Getdents syscall number and dispatch
- `kernel/src/syscall/fs.rs` - Added sys_getdents, updated sys_openat for directories
- `kernel/src/task/fd_table.rs` - Added InitramfsDir FdType variant
- `userspace/libsyscall/src/lib.rs` - Added getdents wrapper, Dirent64, d_type
- `userspace/ulib/src/fs.rs` - Added ReadDir, DirEntry, FileType, read_dir
- `userspace/ulib/src/lib.rs` - Re-exported new types

## Handoff Checklist
- [x] Kernel builds cleanly
- [x] Userspace builds cleanly
- [x] Team file updated
- [x] ROADMAP updated
