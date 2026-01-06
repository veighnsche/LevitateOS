# TEAM_196: Plan Remaining Levbox Syscalls

## Objective
Create feature plan for remaining syscalls needed to complete levbox utilities.

## Status
- **Started:** 2026-01-06
- **Phase:** Planning

## Blockers Identified

| Syscall | Number | Unblocks | Complexity |
|---------|--------|----------|------------|
| `utimensat` | 88 | touch | Medium - needs time handling |
| `linkat` | 37 | ln | Low - simple tmpfs extension |
| `symlinkat` | 36 | ln -s | Medium - needs symlink node type |

## Additional Blockers

| Task | Unblocks | Complexity |
|------|----------|------------|
| Add levbox utils to initramfs | Testing | Low |
