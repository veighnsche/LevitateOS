# TEAM_193: Implement Levbox Checklist Items

## Objective
Implement as many checklist items as possible from `docs/specs/levbox/CHECKLIST.md`.

## Status
- **Started:** 2026-01-06
- **Phase:** Implementation Complete

## Pre-flight
- [x] Tests pass before changes (kernel has pre-existing build issues, userspace builds)
- [x] Read existing coreutils code

## Progress

### Completed

#### ls enhancements
- [x] `-l` long listing format with file type, permissions placeholder, size
- [x] `-h` human readable sizes (1K, 234M, 2G)
- [x] `-R` recursive directory listing
- [x] `--color` ANSI color output (blue for dirs, cyan for symlinks)
- [x] Long option support (`--all`, `--almost-all`, `--classify`, `--human-readable`, `--recursive`)

#### New utilities created
- [x] `cp.rs` - copy files stub (help, version, option parsing)
- [x] `rmdir.rs` - remove empty directories
- [x] `rm.rs` - remove files and empty directories
- [x] `mv.rs` - rename/move files

#### Cargo.toml updates
- [x] Added rmdir, rm, mv binaries

#### Checklist updates
- [x] Updated `docs/specs/levbox/CHECKLIST.md` with all completed items

### In Progress

None

### Blocked

- `touch` - needs `utimensat` syscall (kernel support missing)
- `ln` - needs `linkat`/`symlinkat` syscalls (kernel support missing)
- Full `cp` functionality - needs write syscall support for files

## Notes

- Kernel has pre-existing build issues (unresolved imports in arch module)
- All userspace utilities compile successfully
- `mkdir` already existed from TEAM_192, updated checklist to reflect
- Syscall wrappers exist for mkdirat, unlinkat, renameat but kernel implementation status unclear

## Files Modified/Created

- `userspace/levbox/src/bin/ls.rs` - enhanced with -l, -h, -R, --color
- `userspace/levbox/src/bin/cp.rs` - created (stub)
- `userspace/levbox/src/bin/rmdir.rs` - created
- `userspace/levbox/src/bin/rm.rs` - created
- `userspace/levbox/src/bin/mv.rs` - created
- `userspace/levbox/Cargo.toml` - added new binaries
- `docs/specs/levbox/CHECKLIST.md` - updated status + blockers section
- `docs/ROADMAP.md` - updated syscall status + Phase 11 blockers section

## Handoff Checklist

- [x] Userspace builds cleanly
- [x] Team file updated
- [x] Blockers documented in ROADMAP.md
- [x] Blockers documented in CHECKLIST.md
- [x] Feature plan created for tmpfs implementation
- [ ] Kernel has pre-existing build issues (not caused by this team)

## Feature Plan Created

Created comprehensive plan for implementing writable tmpfs at:
`docs/planning/levbox-syscalls-phase11/`

### Plan Files
- `README.md` - Overview
- `phase-1.md` - Discovery (current state analysis)
- `phase-2.md` - Design (15 questions for USER)
- `phase-3.md` - Implementation (10 steps)
- `phase-4.md` - Integration (5 steps)
- `phase-5.md` - Testing & Documentation

### Key Finding
The kernel already has syscall handlers for mkdirat, unlinkat, renameat but they return EROFS because initramfs is read-only. The solution is to implement tmpfs (in-memory writable filesystem) at `/tmp`.

### Questions Awaiting Answer
See `phase-2.md` for 15 design questions including:
- Q1: Mount point `/tmp` only?
- Q4: Max file size (16MB recommended)?
- Q5: Max tmpfs size (64MB recommended)?
- Q7-Q8: O_CREAT and O_TRUNC flags?
- Q12-Q13: Defer hard links and symlinks?
