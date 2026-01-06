# Phase 11: Writable Filesystem (tmpfs) for Levbox

**Planning Team:** TEAM_193  
**Created:** 2026-01-06

## Overview

This plan addresses the blockers preventing full functionality of levbox utilities (mkdir, rm, rmdir, mv, cp, touch, ln).

## Problem Statement

The kernel already has syscall handlers for:
- `sys_mkdirat` (34)
- `sys_unlinkat` (35)
- `sys_renameat` (38)

However, they all return `EROFS` (read-only filesystem) because **initramfs is read-only**.

## Solution

Implement a **tmpfs** (temporary in-memory filesystem) that supports:
- Creating files and directories
- Removing files and directories
- Renaming/moving files
- Writing to files

## Phases

1. **Phase 1 - Discovery**: Understand current VFS architecture and requirements
2. **Phase 2 - Design**: Design tmpfs and VFS abstraction layer
3. **Phase 3 - Implementation**: Build tmpfs and update syscall handlers
4. **Phase 4 - Integration**: Mount tmpfs at `/tmp`, update shell CWD support
5. **Phase 5 - Testing & Docs**: Verify all levbox utilities work

## Files

- `phase-1.md` - Discovery phase
- `phase-2.md` - Design phase (question-heavy)
- `phase-3.md` - Implementation phase
- `phase-4.md` - Integration phase
- `phase-5.md` - Testing and documentation

## Dependencies

- Existing initramfs and CPIO parser
- Kernel memory allocator (slab/buddy)
- Syscall infrastructure

## Success Criteria

- [ ] `mkdir /tmp/test` creates directory
- [ ] `touch /tmp/file` creates file
- [ ] `rm /tmp/file` removes file
- [ ] `rmdir /tmp/test` removes directory
- [ ] `mv /tmp/a /tmp/b` renames file
- [ ] `cp /init /tmp/init_copy` copies file
