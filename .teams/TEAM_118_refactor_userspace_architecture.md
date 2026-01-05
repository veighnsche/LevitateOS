# TEAM_118: Refactor Userspace Architecture

> **Status:** COMPLETED
> **Drift:** Minimal (deferred `sys_exec`, deferred `init` process to Phase 8c)

## Objectives
- [x] Create shared `libsyscall` crate
- [x] Refactor `shell` to use `libsyscall`
- [x] Eliminate `sys_exec` dependency (interim)
- [x] Clean up `userspace/hello`
- [x] Verify migration (Kernel runs `shell`)

## Log
- **2026-01-05:** Created implementation plan.
- **2026-01-05:** Created `userspace` workspace, `libsyscall`, and updated `shell`.
- **2026-01-05:** Fixed linker issues using per-crate `build.rs`.
- **2026-01-05:** Updated kernel to run `shell` directly (Workaround for missing `sys_exec`).
- **2026-01-05:** Updated `initramfs` workflow.
- **2026-01-05:** Verified logs and updated golden master.
- **2026-01-05:** Cleaned up legacy `userspace/hello`.
- **2026-01-05:** **ALL TESTS PASSED**.

## Next Steps (Handoff)
- **Phase 8c:** Implement `sys_spawn` / `sys_exec` in kernel.
- **Phase 8c:** Create `userspace/init` (PID 1) to manage shell.
- **Validation:** Check VNC output (Visual inspection).
