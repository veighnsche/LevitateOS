# UoW 3.3: Add VmaList to TaskControlBlock

**Parent:** Phase 3, Step 3 (VMA Tracking)  
**File:** `kernel/src/task/mod.rs`  
**Depends On:** UoW 3.2  
**Complexity:** Low

---

## Context

UoW 3.2 created `VmaList`. Now we need to add it to each process's TaskControlBlock.

---

## Task

Add a `vmas` field to `TaskControlBlock` and initialize it in all constructors.

---

## Implementation

### Step 1: Add import

At the top of `kernel/src/task/mod.rs`, add:

```rust
use crate::memory::vma::VmaList;
```

### Step 2: Add field to TaskControlBlock

Find the `TaskControlBlock` struct and add the field:

```rust
pub struct TaskControlBlock {
    // ... existing fields ...
    
    /// TEAM_235: Virtual memory area tracking for munmap support
    pub vmas: IrqSafeLock<VmaList>,
}
```

### Step 3: Initialize in constructors

Find all places where `TaskControlBlock` is created and add initialization:

**In `TaskControlBlock::new_bootstrap()` or similar:**
```rust
vmas: IrqSafeLock::new(VmaList::new()),
```

**In `TaskControlBlock::new()` or `from()` methods:**
```rust
vmas: IrqSafeLock::new(VmaList::new()),
```

**In `TaskControlBlock::from(UserProcess)` if exists:**
```rust
vmas: IrqSafeLock::new(VmaList::new()),
```

---

## Files to Check

The constructors may be in:
- `kernel/src/task/mod.rs`
- `kernel/src/task/process.rs`
- `kernel/src/task/thread.rs`

Search for `TaskControlBlock {` or `Self {` to find all construction sites.

---

## Acceptance Criteria

- [ ] `vmas` field added to `TaskControlBlock`
- [ ] Field is `IrqSafeLock<VmaList>` type
- [ ] All constructors initialize with empty `VmaList`
- [ ] Code compiles without errors

---

## Testing

```bash
cargo build -p levitate-kernel
```

---

## Next UoW

Proceed to **UoW 3.4** to update sys_mmap to record VMAs.
