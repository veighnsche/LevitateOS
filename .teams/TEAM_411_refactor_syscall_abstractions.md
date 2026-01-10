# TEAM_411: Syscall Abstractions Refactor

**Created**: 2026-01-10
**Status**: Planning

## Objective

Introduce common abstractions to reduce boilerplate and improve consistency across syscall implementations while maintaining full Linux ABI compatibility.

## Identified Abstractions

1. **VfsError → errno conversion** - Centralize error mapping
2. **UserBuffer wrapper** - Safe user-space memory handling
3. **with_fd helper** - Reduce fd lookup boilerplate
4. **resolve_at_path** - Proper dirfd support for `*at()` syscalls
5. **SyscallContext** - Ergonomic task/ttbr0 access

## Key Constraint

**All changes must preserve Linux ABI compatibility.** The syscall interface visible to userspace must remain identical.

## Planning Location

`docs/planning/syscall-abstractions/`

## Progress Log

- 2026-01-10: Team registered, starting refactor planning
