# Levbox Remaining Syscalls Plan

**Team:** TEAM_196  
**Created:** 2026-01-06

## Overview

This plan addresses the remaining syscalls needed to complete levbox utilities:

1. **`utimensat`** (88) - Set file timestamps → unblocks `touch`
2. **`linkat`** (37) - Create hard links → unblocks `ln`
3. **`symlinkat`** (36) - Create symbolic links → unblocks `ln -s`

Plus: Adding levbox utilities to the initramfs for testing.

## Phases

1. **Phase 1 - Discovery**: Understand requirements and current state ✅
2. **Phase 2 - Design**: Define syscall interfaces and tmpfs extensions ✅
3. **Phase 3 - Implementation**: Implement syscalls

> **Note:** Integration and polish steps are included in Phase 3.

## Success Criteria

- [ ] `touch /tmp/file` creates file with current timestamp
- [ ] `touch -c /tmp/nonexistent` doesn't create file
- [ ] `ln -s /tmp/a /tmp/link` creates symbolic link
- [ ] All levbox utilities available in initramfs
- [ ] `ls`, `mkdir`, `rm`, `mv` work in QEMU

> **Note:** Hard links (`linkat`) deferred to future — complexity not justified for Phase 11.

## Priority Order

1. **P0**: Add levbox utilities to initramfs (enables testing)
2. **P1**: Implement `utimensat` (enables `touch`)
3. **P2**: Implement `symlinkat` (enables `ln -s`)
4. **P3**: Defer `linkat` (hard links are complex, low priority)
