# TEAM_312: Kernel Fork/Exec Implementation

**Created**: 2026-01-08
**Status**: Planning
**Parent**: Unblocks Phase 3 of stability-maturation plan

---

## Problem Statement

The kernel's current process creation model uses custom syscalls (`Spawn`, `SpawnArgs`) 
instead of the Linux-standard `fork()` + `execve()` pattern. This blocks:

1. Removal of deprecated custom syscalls from `los_abi`
2. Linux binary compatibility
3. Standard process creation patterns (fork+exec, vfork+exec)

### Current State
- `sys_clone`: Only supports **thread-style** clones (CLONE_VM | CLONE_THREAD)
- `sys_exec`: Returns **ENOSYS** (stub)
- `sys_spawn/sys_spawn_args`: Work but use custom syscall numbers (1000, 1001)

### Target State
- `sys_clone`: Support **fork-style** clones (new address space)
- `sys_exec`: Replace current process image with new ELF
- Migrate all spawn callsites to fork+exec pattern
- Remove Spawn/SpawnArgs from los_abi

---

## Planning Location

All planning docs: `docs/planning/fork-exec/`

---

## Progress

- [x] Phase 1: Discovery (understand current process model) ✅
- [x] Phase 2: Design (define fork/exec behavior) ✅
- [x] Phase 3: Implementation plan ✅
- [ ] Phase 4: Integration and testing (implementation needed)
- [ ] Phase 5: Cleanup and handoff

## Planning Complete - Ready for Implementation

All planning documents created in `docs/planning/fork-exec/`:
- `phase-1.md` - Discovery: analyzed current spawn/clone/exec
- `phase-2.md` - Design: defined fork/exec behavior contracts  
- `phase-3.md` - Implementation: step-by-step plan with code

## Implementation Order

1. **Memory primitives**: `copy_user_address_space()`, `clear_user_address_space()`
2. **Fork**: Modify `sys_clone()` to handle fork case
3. **Exec**: Full `sys_exec()` implementation
4. **Userspace**: Add `fork()` wrapper
5. **Migration**: Update spawn callsites, remove deprecated syscalls

---

## Handoff Checklist
- [ ] Project builds
- [ ] All tests pass
- [ ] Fork+exec works
- [ ] Spawn callsites migrated
- [ ] Spawn/SpawnArgs removed from los_abi
