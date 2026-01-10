# Phase 5: Cleanup and Hardening

**Status**: Ready for execution
**TEAM**: TEAM_415
**Dependencies**: Phase 4

---

## Purpose

Final cleanup: replace obvious invariant `expect()` with `unwrap_unchecked()` where safe, add `#[track_caller]` for better backtraces, and fix `unimplemented!()`.

---

## Target Items

### 5.1 Replace Invariant expect() with unwrap_unchecked()

Where the type system or prior checks guarantee success, use `unsafe { unwrap_unchecked() }` with a safety comment:

```rust
// Before
reserved[next_idx].as_ref().expect("Checked None above");

// After
// SAFETY: Loop condition ensures reserved[next_idx] is Some
unsafe { reserved[next_idx].as_ref().unwrap_unchecked() }
```

**Candidates**:
- `src/memory/mod.rs:69` - Reserved memory checked None above
- `hal/src/virtio.rs:35-36` - Layout from constants
- `hal/src/allocator/intrusive_list.rs:114` - NonNull invariant

### 5.2 Add #[track_caller]

Add `#[track_caller]` to functions that may panic, for better backtraces:

```rust
#[track_caller]
pub fn current_task_unchecked() -> Arc<Task> { ... }
```

### 5.3 Fix unimplemented!()

```rust
// src/arch/x86_64/mod.rs:564
unimplemented!("x86_64 exception_return");
```

Options:
1. Implement the function
2. Make it a compile-time error: `compile_error!("x86_64 exception_return not implemented")`
3. If it's truly dead code, remove it

---

## Steps

### Step 1: Audit and Replace unwrap_unchecked Candidates

**File**: `phase-5-step-1.md`

For each candidate:
1. Verify the invariant holds (read surrounding code)
2. Add safety comment
3. Replace with `unsafe { unwrap_unchecked() }`

**UoW size**: 3-5 changes across files.

### Step 2: Add #[track_caller] Annotations

**File**: `phase-5-step-2.md`

Add `#[track_caller]` to:
- `current_task_unchecked()`
- Any other functions that panic on programmer error

**UoW size**: Small mechanical changes.

### Step 3: Fix x86_64 unimplemented!()

**File**: `phase-5-step-3.md`

1. Investigate `exception_return` - is it called?
2. If never called, consider compile_error! or removal
3. If called, implement or document why panic is acceptable

**UoW size**: 1 decision + 1 change.

---

## Dead Code Removal (Rule 6)

During this phase, remove any dead code discovered:
- Unused functions
- Commented-out code
- "Kept for reference" logic

---

## Exit Criteria

- [ ] All safe invariant `expect()` replaced with `unwrap_unchecked()`
- [ ] `#[track_caller]` added to panic-prone functions
- [ ] `unimplemented!()` resolved
- [ ] Build passes
- [ ] All behavior tests pass
- [ ] No dead code
