# Phase 4: Task System Safety

**Status**: Ready for execution
**TEAM**: TEAM_415
**Dependencies**: Phase 3

---

## Purpose

Make `current_task()` return `Option<Arc<Task>>` for safer API, with an `_unchecked` variant for performance-critical paths.

---

## Target Design

### Current (Unsafe)

```rust
// src/task/mod.rs:106
pub fn current_task() -> Arc<Task> {
    CURRENT_TASK.with(|t| t.borrow().clone())
        .expect("current_task() called before scheduler init")
}
```

### New (Safe)

```rust
/// Returns the current task, or None if scheduler not initialized.
pub fn current_task() -> Option<Arc<Task>> {
    CURRENT_TASK.with(|t| t.borrow().clone())
}

/// Returns the current task, panicking if scheduler not initialized.
/// 
/// # Panics
/// Panics if called before scheduler initialization.
/// 
/// # Use Cases
/// - Performance-critical paths where Option overhead matters
/// - Code paths that are architecturally guaranteed to run after scheduler init
pub fn current_task_unchecked() -> Arc<Task> {
    current_task().expect("current_task_unchecked() called before scheduler init")
}
```

---

## Migration Strategy

### Phase 4a: Add Safe API

1. Rename current `current_task()` to `current_task_unchecked()`
2. Add new `current_task()` that returns `Option`
3. Compile - this should succeed (all callers use the unchecked variant)

### Phase 4b: Migrate Call Sites

For each call site, decide:
- **Keep unchecked**: If architecturally guaranteed to run after scheduler init (e.g., syscall handlers)
- **Use Option**: If there's any doubt, or if the code path could be reached early

Most syscall handlers can stay with `current_task_unchecked()` since they're only called from userspace after the scheduler is running.

---

## Steps

### Step 1: Add current_task Option API

**File**: `phase-4-step-1.md`

1. Modify `src/task/mod.rs`
2. Add `current_task()` returning `Option<Arc<Task>>`
3. Rename old function to `current_task_unchecked()`
4. Update re-exports if any

**UoW size**: 1 file, small change.

### Step 2: Audit Call Sites

**File**: `phase-4-step-2.md`

1. Search for all `current_task()` usages
2. Categorize: guaranteed-safe vs needs-option
3. Update calls that need `Option` handling
4. Leave syscall handlers on `_unchecked` (documented decision)

**UoW size**: Many files, but mechanical changes.

---

## Exit Criteria

- [ ] `current_task()` returns `Option`
- [ ] `current_task_unchecked()` available for perf-critical paths
- [ ] All call sites audited and updated
- [ ] Build passes
- [ ] Behavior tests pass
