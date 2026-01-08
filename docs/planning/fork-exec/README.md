# Fork/Exec Implementation Plan

**TEAM_312**: Kernel Fork/Exec
**Created**: 2026-01-08
**Status**: Planning Complete, Ready for Implementation

---

## Overview

Implement Linux-compatible `fork()` and `execve()` syscalls to replace custom 
`Spawn`/`SpawnArgs` syscalls. This unblocks Phase 3 of the stability-maturation plan.

## Documents

| Document | Description | Status |
|----------|-------------|--------|
| [phase-1.md](phase-1.md) | Discovery - Current process model | ✅ Complete |
| [phase-2.md](phase-2.md) | Design - Fork/exec behavior | ✅ Complete |
| [phase-3.md](phase-3.md) | Implementation - Step-by-step | ✅ Complete |

---

## Quick Summary

### What Needs to Be Done

1. **Fork**: Modify `sys_clone()` to copy address space when `!(CLONE_VM | CLONE_THREAD)`
2. **Exec**: Implement `sys_exec()` to replace current process with new ELF
3. **Migrate**: Update spawn callsites to use fork+exec
4. **Cleanup**: Remove Spawn/SpawnArgs from kernel and los_abi

### Key New Functions

| Function | File | Purpose |
|----------|------|---------|
| `copy_user_address_space()` | `memory/user.rs` | Copy parent's pages for fork |
| `clear_user_address_space()` | `memory/user.rs` | Free user pages for exec |
| `sys_clone_fork()` | `syscall/process.rs` | Fork case of clone |
| `fork()` | `libsyscall/process.rs` | Userspace fork wrapper |

### Implementation Order

```
Step 1: Memory primitives (copy/clear address space)
Step 2: Fork implementation (modify sys_clone)
Step 3: Exec implementation (full sys_exec)
Step 4: Userspace API (fork() wrapper)
Step 5: Integration (migrate spawn callsites, remove deprecated)
```

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| COW vs Eager Copy | Eager | Simpler, COW is future optimization |
| FD inheritance | Clone on fork | Standard Unix behavior |
| vfork | Not implemented | Not needed initially |

---

## Files to Modify

### Kernel
- `kernel/src/memory/user.rs` - Add copy/clear functions
- `kernel/src/syscall/process.rs` - Fork case, full exec
- `kernel/src/task/process.rs` - May need helper functions

### Userspace  
- `userspace/libsyscall/src/process.rs` - Add fork()
- `userspace/init/src/main.rs` - Migrate spawn
- `userspace/shell/src/main.rs` - Migrate spawn_args
- `userspace/levbox/src/bin/test/*.rs` - Migrate spawn

### ABI
- `crates/abi/src/syscall/*.rs` - Remove Spawn/SpawnArgs

---

## Success Criteria

- [ ] `fork()` creates child with copied address space
- [ ] `execve()` replaces current process
- [ ] All spawn callsites migrated
- [ ] Spawn/SpawnArgs removed from los_abi
- [ ] All tests pass
- [ ] No deprecation warnings
