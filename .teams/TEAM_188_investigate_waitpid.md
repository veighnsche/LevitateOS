# TEAM_188: Investigation — Shell Child Process Synchronization (waitpid)

**Created:** 2026-01-06  
**Type:** Investigation / Missing Feature  
**Status:** Phase 1 — Understanding the Symptom

## Bug Report / Issue

**Symptom:** When the shell spawns an external command (e.g., `cat /hello.txt`), it returns to the prompt immediately without waiting for the child process to complete. The child runs concurrently, causing interleaved output.

**Expected:** Shell should wait for the child to finish before printing the next prompt.

**Current Behavior:** Shell calls `spawn_args()`, gets PID, and immediately returns to `execute()` → next prompt loop.

## Environment

- LevitateOS Phase 11
- Kernel with `SYS_SPAWN_ARGS` syscall
- Shell spawns processes but cannot wait for them

---

## Phase 1: Understand the Symptom

### Symptom Description

| Expected | Actual |
|----------|--------|
| `cat /hello.txt` prints file, then `#` prompt | `#` prompt may appear before/during cat output |
| Sequential execution | Concurrent execution |
| Shell blocked until child exits | Shell immediately ready for next command |

### Location of Symptom

- **Shell side:** `execute()` in `userspace/shell/src/main.rs` line ~156
- **Kernel side:** No `waitpid`-like syscall exists

### Trace of Execution

```
shell: execute()
  → spawn_args("/cat", ["cat", "/hello.txt"])
    → kernel: sys_spawn_args() → spawns task, returns PID
  → shell receives PID
  → shell returns from execute() immediately (no blocking)
  → shell loops back to prompt
  
Meanwhile:
  scheduler runs cat process
  cat outputs "Hello from initramfs!"
  cat exits
  (shell doesn't know cat finished)
```

---

## Phase 2: Form Hypotheses

This isn't a "bug" per se — it's a **missing feature**. But I'll investigate what's needed.

### Hypothesis 1: Need waitpid syscall

**Description:** The kernel needs a `waitpid(pid)` syscall that blocks until the specified child process exits.

**Evidence needed:**
- Confirm no waitpid syscall exists
- Understand how process exit works
- Identify what state is tracked when a process exits

**Confidence:** HIGH — this is the standard solution.

### Hypothesis 2: Alternative — poll-based waiting

**Description:** Instead of blocking waitpid, add `getpid_status(pid)` and have shell poll.

**Evidence needed:**
- Check if process exit status is stored anywhere
- Evaluate if polling is acceptable

**Confidence:** LOW — polling is worse than blocking.

---

## Phase 3: Test Hypotheses with Evidence

### Hypothesis 1: Need waitpid syscall — CONFIRMED

**Evidence gathered:**

1. **No parent-child relationship tracked:**
   - Searched for `parent`, `ppid`, `child` in task module — no results
   - `TaskControlBlock` has no field for parent PID

2. **Process exit doesn't notify anyone:**
   - `task_exit()` just sets state to `Exited` and calls `schedule()`
   - No notification mechanism for waiting processes
   - No exit code stored anywhere

3. **Scheduler doesn't track exited tasks:**
   - Ready list is just a `VecDeque<Arc<TaskControlBlock>>`
   - Exited tasks simply aren't re-added to ready queue
   - No separate list of zombie/exited processes

4. **Roadmap confirms this:**
   - `docs/ROADMAP.md` line 217: "`wait4` / `waitpid` | 🔴 Not implemented | Phase 12"
   - `docs/planning/interactive-shell-phase8b/EPIC.md` line 66: "[ ] waitpid(pid)"

---

## Phase 4: Root Cause Analysis

### Root Cause

**This is not a bug — it's a missing feature.** The kernel was designed for cooperative multitasking without parent-child process relationships.

### What's Missing for waitpid

| Component | Current State | Required |
|-----------|---------------|----------|
| Parent PID | Not tracked | Store spawner's PID in child TCB |
| Exit code | Not stored | Store exit code in TCB when exited |
| Wait queue | Doesn't exist | List of tasks waiting for specific PIDs |
| Zombie state | Not implemented | Task in `Exited` state waiting to be reaped |
| Notification | None | Wake waiters when child exits |

### Scope Assessment

This is a **medium-sized feature**, not a small fix:

- **Kernel changes**: ~150-200 lines
  - Add `parent_pid` and `exit_code` to TCB
  - Add wait queue data structure
  - Add `sys_waitpid` syscall
  - Modify `task_exit()` to wake waiters

- **Userspace changes**: ~30 lines
  - Add `waitpid()` wrapper to libsyscall
  - Call it in shell after `spawn_args()`

---

## Phase 5: Decision

### Decision: **Create a Feature Plan**

This exceeds the "fix immediately" criteria:
- > 50 lines of code
- Touches critical kernel paths (task management)
- Requires new data structures

**Recommendation:** Use `/make-a-new-feature-plan` workflow.

---

## Handoff Notes

**Summary:** Shell needs `waitpid` to wait for child processes. The kernel lacks:
1. Parent-child relationship tracking
2. Exit code storage
3. Wait queue mechanism
4. Zombie process state

**Breadcrumbs placed:** TODO comment in shell at line 156.

**Next steps:** Create feature plan for `waitpid` implementation.
