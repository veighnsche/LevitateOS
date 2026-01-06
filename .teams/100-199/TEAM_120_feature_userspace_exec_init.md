# TEAM_120: Userspace Exec and Init Process

**Team ID:** TEAM_120
**Objective:** Implement kernel support for `sys_spawn`/`sys_exec` and establish the `init` process (PID 1).

## Notes

- **Recursive Deadlock**: Found that holding the `INITRAMFS` lock while calling `run_from_initramfs` (which never returns) caused the system to hang on the first `spawn` syscall. Releasing the lock by copying the `CpioArchive` struct solved it.
- **Copy/Clone for Archives**: Derived `Copy` and `Clone` for `CpioArchive` to facilitate releasing locks before long operations.
- **Task Trampoline**: Implemented a kernel trampoline (`task_entry_trampoline`) to ensure system hooks (like `post_switch_hook`) are called for new tasks before they enter userspace.
- **`From<UserTask>` Pattern**: Used a clean conversion pattern to map high-level `UserTask` objects into the scheduler's `TaskControlBlock`.
- **Log Capture Technique**: Used `cargo xtask run > boot_output.txt 2>&1 & sleep 10 && kill $! || true` for reliable, string-verifiable boot tests in an agentic environment.

## Scope
1. **Infrastructure:** Synchronize userspace lints with kernel lints.
2. **Phase 8c:** `sys_spawn` / `sys_exec` implementation.
3. **Phase 8c:** `init` process implementation.
4. **Cleanup:** Integrate userspace builds into the core test suite.

## Log
- **2026-01-05:** Initialized team and started Discovery phase.

## Links
- [task.md](file:///home/vince/.gemini/antigravity/brain/831af642-7367-4e9a-a333-4df557baafaa/task.md)
- [phase-1.md](file:///home/vince/.gemini/antigravity/brain/831af642-7367-4e9a-a333-4df557baafaa/phase-1.md)
