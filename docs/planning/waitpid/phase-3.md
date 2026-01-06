# Phase 3: Fix Design and Validation Plan — Waitpid

**TEAM_188** | 2026-01-06

## Root Cause Summary

The kernel lacks:
1. Parent-child relationship tracking
2. Exit code storage
3. A mechanism for one task to block waiting for another

## Fix Strategy

### Approach: Minimal Wait Queue with Process Table

Add a global process table that tracks all running processes by PID, their parent PID, exit status, and waiters.

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                     PROCESS_TABLE                            │
│  HashMap<Pid, ProcessEntry>                                  │
│                                                              │
│  ProcessEntry {                                              │
│      task: Option<Arc<TCB>>,   // None when zombie          │
│      parent_pid: Pid,                                        │
│      exit_code: Option<i32>,   // Set when exited           │
│      waiters: Vec<Arc<TCB>>,   // Tasks blocked on waitpid  │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
```

### Flow: Child Exit

```
1. Child calls sys_exit(code)
2. sys_exit stores exit_code in PROCESS_TABLE
3. sys_exit checks if parent is waiting
4. If parent waiting: wake parent, remove zombie
5. If parent not waiting: mark as zombie (stays in table)
6. Child task enters Exited state
```

### Flow: Parent Waits

```
1. Parent calls sys_waitpid(child_pid)
2. Kernel looks up child in PROCESS_TABLE
3. If child already exited: return exit code immediately
4. If child running: 
   a. Add parent to child's waiters list
   b. Block parent (set state to Blocked)
   c. Schedule next task
5. When child exits: parent is woken with exit code
```

## Data Structure Changes

### New: ProcessEntry

```rust
/// TEAM_188: Process table entry
pub struct ProcessEntry {
    /// The task control block (None if zombie)
    pub task: Option<Arc<TaskControlBlock>>,
    /// Parent process ID
    pub parent_pid: Pid,
    /// Exit code (Some after exit, None while running)
    pub exit_code: Option<i32>,
    /// Tasks waiting for this process to exit
    pub waiters: Vec<Arc<TaskControlBlock>>,
}
```

### New: Global Process Table

```rust
/// TEAM_188: Global process table
pub static PROCESS_TABLE: IrqSafeLock<HashMap<Pid, ProcessEntry>> = 
    IrqSafeLock::new(HashMap::new());
```

### Modified: TaskControlBlock

No changes needed — parent info is in PROCESS_TABLE.

## Syscall Design

### SYS_WAITPID (16)

```rust
/// Wait for a child process to exit.
///
/// # Arguments
/// - x0: pid        - PID to wait for (must be > 0 for now)
/// - x1: status_ptr - Pointer to store exit status (or 0 to ignore)
///
/// # Returns
/// PID of exited child on success, negative errno on failure.
pub fn sys_waitpid(pid: i32, status_ptr: usize) -> i64;
```

### Modified: sys_exit

```rust
pub fn sys_exit(code: i32) -> ! {
    let task = current_task();
    let pid = task.id.0 as Pid;
    
    // 1. Store exit code and wake waiters
    let mut table = PROCESS_TABLE.lock();
    if let Some(entry) = table.get_mut(&pid) {
        entry.exit_code = Some(code);
        entry.task = None; // Become zombie
        
        // Wake all waiters
        for waiter in entry.waiters.drain(..) {
            waiter.set_state(TaskState::Ready);
            SCHEDULER.add_task(waiter);
        }
    }
    drop(table);
    
    // 2. Mark as exited and schedule
    task.set_state(TaskState::Exited);
    SCHEDULER.schedule();
    // ...
}
```

## Reversal Strategy

If the fix doesn't work:

1. Revert the commit(s)
2. Remove PROCESS_TABLE and related code
3. Revert sys_exit to original behavior
4. Shell will work as before (no waiting, but functional)

**Signals to revert:**
- Deadlocks when spawning processes
- Kernel panics in waitpid path
- Memory corruption in process table

## Test Strategy

### New Tests

1. **Basic waitpid test:**
   - Spawn child that exits with code 42
   - Wait for child
   - Verify returned exit code is 42

2. **Wait for already-exited child (zombie):**
   - Spawn child that exits immediately
   - Sleep briefly
   - Call waitpid — should return immediately

3. **Invalid PID:**
   - Call waitpid(-1) or waitpid(nonexistent)
   - Should return ECHILD

### Manual Verification

```bash
# In shell:
cat /hello.txt
# Should see file content, THEN prompt
```

## Impact Analysis

| Aspect | Change |
|--------|--------|
| Memory | Small: ~64 bytes per process in table |
| Performance | Minimal: One hash lookup per spawn/exit |
| API | New syscall, no changes to existing |
| Compatibility | Existing code unaffected |

## Files to Modify

| File | Changes |
|------|---------|
| `kernel/src/task/mod.rs` | Add ProcessEntry, PROCESS_TABLE |
| `kernel/src/syscall/mod.rs` | Add Waitpid = 16 |
| `kernel/src/syscall/process.rs` | Add sys_waitpid, modify sys_exit, sys_spawn_args |
| `userspace/libsyscall/src/lib.rs` | Add waitpid wrapper |
| `userspace/shell/src/main.rs` | Call waitpid after spawn_args |
