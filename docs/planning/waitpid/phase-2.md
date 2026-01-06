# Phase 2: Root Cause Analysis — Waitpid

**TEAM_188** | 2026-01-06

## Root Cause Summary

**The kernel lacks parent-child process tracking and a blocking wait mechanism.**

When `sys_spawn_args` creates a child process:
1. The child is added to the scheduler's ready queue
2. The parent receives the child's PID immediately
3. The parent continues executing (no blocking)
4. The child runs concurrently
5. When the child exits, no notification occurs

## Hypotheses

### H1: Missing waitpid syscall — CONFIRMED

The kernel has no syscall for waiting on a child process.

**Evidence:**
- Searched for `waitpid` in kernel — not found
- `kernel/src/syscall/mod.rs` doesn't define it
- Roadmap confirms: "🔴 Not implemented | Phase 12"

### H2: No parent-child relationship — CONFIRMED

`TaskControlBlock` has no `parent_pid` field.

**Evidence:**
- Searched for `parent`, `ppid`, `child` in task module — no results
- When spawn creates a child, there's no link back to parent

### H3: No exit code storage — CONFIRMED

Exit code is passed to `sys_exit` but not stored.

**Evidence:**
- `task_exit()` sets state to `Exited` but doesn't save code
- TCB has no `exit_code` field

### H4: No wait queue mechanism — CONFIRMED

No data structure exists for tasks waiting on PIDs.

**Evidence:**
- `Scheduler` only has `ready_list`
- No `wait_queue` or similar

## Key Code Areas

```
kernel/src/task/mod.rs
├── TaskControlBlock     <-- Needs: parent_pid, exit_code
├── task_exit()          <-- Needs: store exit code, wake waiters
└── TaskState            <-- May need: Zombie state

kernel/src/task/scheduler.rs
├── Scheduler            <-- Needs: wait queue or process table
└── (new) wait_for_pid() <-- New function

kernel/src/syscall/process.rs
├── sys_exit()           <-- Needs to pass exit code
└── (new) sys_waitpid()  <-- New syscall

kernel/src/syscall/mod.rs
└── SyscallNumber        <-- Add Waitpid = 16
```

## Investigation Strategy

No further investigation needed — root cause is clear. Proceed to design phase.
