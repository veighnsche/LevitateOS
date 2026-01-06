# TEAM_177: File Read Implementation

## Status: Planning

## Objective
Implement `File::read()` for initramfs files in ulib, completing the File Abstractions unit of work.

## Key Finding
The `InitramfsFile` FdType already has an `offset` field for position tracking!
Current `sys_read` only handles fd 0 (stdin). Need to extend it to handle `InitramfsFile` fds.

## Dependencies
- Kernel `sys_read` needs to dispatch to initramfs read logic
- FdTable already tracks offset per fd
- ulib `File::read()` just needs to call libsyscall::read()

## Progress
- [x] Team registered
- [ ] Phase 1 written (Discovery)
- [ ] Phase 2 written (Design with questions)
- [ ] User reviews questions
- [ ] Implementation
