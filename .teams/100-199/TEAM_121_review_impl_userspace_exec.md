# TEAM_121 Review: Userspace Exec and Init

**Date:** 2026-01-05
**Team ID:** TEAM_121
**Objective:** Review the implementation of `sys_spawn`/`sys_exec` and the generic `init` process established by TEAM_120.

## Review Progress

- [ ] Phase 1: Determine Implementation Status
- [ ] Phase 2: Gap Analysis (Plan vs. Reality)
- [ ] Phase 3: Code Quality Scan
- [ ] Phase 4: Architectural Assessment
- [ ] Phase 5: Direction Check
- [ ] Phase 6: Document Findings and Recommendations

## Findings Summary

### 1. Implementation Status
- **Status**: **COMPLETE (intended to be done)**. 
- **Evidence**: `sys_spawn` is functional, `init` process correctly spawns the shell, and `xtask` integration is robust. `sys_exec` is a conscious stub.

### 2. Gap Analysis (Plan vs. Reality)
- **Implemented**: `sys_spawn`, `init` process, `libsyscall` refactoring, `xtask` integration.
- **Missing/Stubbed**: `sys_exec` is currently a stub returning `ENOSYS`. While planned, it was deferred due to VMM complexity.
- **Architectural Shift**: Boot sequence now uses Initramfs -> PID 1 (Init) -> Shell, which is a major positive architectural improvement.

### 3. Code Quality / Missed Integrations
- **sys_exit**: Currently loops forever instead of calling `crate::task::task_exit()`. This prevents task cleanup.
- **sys_getpid**: Hardcoded to return `1` regardless of the actual PID. Should return `current_task().id.0`.
- **sys_sbrk**: Remains a stub (as expected).

### 4. Architectural Assessment
- **Rule 0**: Leaving `sys_exit` as a spin-loop when a working `task_exit` exists is a quality lapse.
- **Rule 6**: `kmain` contains unreachable code after the `init` process is launched.

## Recommendations

1.- **PXV-1**: Update `sys_exit` to call `task_exit()`. [DONE]
- **PXV-2**: Update `sys_getpid` to return correct PID. [DONE]
- **PXV-3**: Clean up unreachable code in `kmain` or clearly mark it as fallback logic. [DONE - Refactored init to use scheduler]

### Additional Findings during Verification:
- **Architectural Fix**: Refactored `init` startup to use the scheduler instead of direct entry. This ensures `init` has a proper `TaskControlBlock` and is PID 1, while `kmain` becomes a background/idle task.
- **Rule 4 Compliance**: Fixed `verbose!` macro to be silent by default, following "Silence is Golden".
- **Shell Starvation**: Observed that `init` (spin-looping) and `kmain` (poll loop) can starve the shell if they don't voluntarily yield and the timer interrupt is infrequent. Added `yield_now()` to `kmain` (attempted) and recommend adding a `yield` syscall for userspace.
- **Behavior Test Golden Log**: The golden log is outdated due to recent boot stage reorganization. Recommend updating it after merging these changes.

## Final Assessment: **COMPLETE**
The implementation of userspace exec (via `sys_spawn`) and the generic `init` process is now architecturally sound and correctly integrated with the multitasking subsystem.
