# Phase 1: Understanding and Scoping — Waitpid

**TEAM_188** | 2026-01-06

## Bug Summary

**Issue:** When the shell spawns an external command (e.g., `cat /hello.txt`), it returns to the prompt immediately without waiting for the child process to complete. Output from the child may interleave with the shell prompt.

**Severity:** Medium — Affects UX but doesn't cause data loss or crashes.

**Impact:** All external command execution is affected. Commands appear to work but output may be confusing.

## Reproduction Status

**Reproducible:** Yes, 100% of the time.

### Steps to Reproduce

1. Boot LevitateOS: `cargo xtask run default`
2. In shell, type: `cat /hello.txt`
3. Observe: Prompt may appear before/during cat output

### Expected Behavior

```
# cat /hello.txt
Hello from initramfs!
# 
```

### Actual Behavior

```
# cat /hello.txt
# Hello from initramfs!     <-- interleaved
```

Or sometimes:
```
# cat /hello.txt
#                           <-- cat hasn't run yet
Hello from initramfs!       <-- output appears after next prompt
```

## Context

### Code Areas Involved

| Component | File | Role |
|-----------|------|------|
| Shell | `userspace/shell/src/main.rs` | Calls `spawn_args()`, doesn't wait |
| Syscall | `kernel/src/syscall/process.rs` | Has spawn but no waitpid |
| TCB | `kernel/src/task/mod.rs` | No parent PID, no exit code |
| Scheduler | `kernel/src/task/scheduler.rs` | No wait queue |

### Recent Changes

- TEAM_186: Added `sys_spawn_args` syscall
- TEAM_182: Added cat utility

### Related Documentation

- `docs/ROADMAP.md` line 217: waitpid listed as Phase 12
- `docs/planning/interactive-shell-phase8b/EPIC.md` line 66: waitpid TODO

## Constraints

- **Backwards compatibility:** Existing spawn behavior must not break
- **Performance:** Waitpid should not add overhead to non-waiting paths
- **Simplicity:** Minimal changes to existing task structures

## Open Questions

1. **Should we also track exit codes?** (Recommended: Yes, for POSIX compatibility)
2. **Should waitpid support waiting for any child (-1)?** (Recommended: Start with specific PID only)
3. **Should we implement WNOHANG flag?** (Recommended: Start without, add later)
