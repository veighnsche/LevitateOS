# TEAM_175: Directory Iteration Feature Planning

## Status: In Progress

## Objective
Create a feature plan for Directory Iteration (`ReadDir`) in ulib Phase 10.

## Context
- Phase 10 roadmap item: "Directory Iteration: `ReadDir` iterator (requires `sys_getdents`)"
- Kernel needs `SYS_GETDENTS` syscall (currently missing from libsyscall)
- userspace-abi.md defines `getdents64` (NR 61 Linux) and `Dirent64` struct

## Artifacts
- Planning docs: `docs/planning/directory-iteration/`
- Phase 1: Discovery
- Phase 2: Design (questions for user)

## Dependencies
- libsyscall needs new syscall wrapper
- Kernel needs syscall implementation
- ulib/fs.rs needs ReadDir type

## Progress
- [x] Team registered
- [x] Context gathered
- [x] Phase 1 written (`docs/planning/directory-iteration/phase-1.md`)
- [x] Phase 2 written (`docs/planning/directory-iteration/phase-2.md`)
- [x] Questions file created (`.questions/TEAM_175_directory_iteration.md`)
- [ ] User reviews questions (7 questions pending)
- [ ] Implementation plan created (Phase 3)
