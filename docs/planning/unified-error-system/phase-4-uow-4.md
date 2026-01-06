# UoW 4: Migrate user_mm.rs to MmuError

**Phase:** 4 - Implementation  
**Parent:** `phase-4.md`  
**Dependencies:** UoW 3 (MmuError must exist)  
**Estimated Lines:** ~40

---

## Objective

Replace `&'static str` errors in `kernel/src/task/user_mm.rs` with `MmuError`.

---

## Target File

`kernel/src/task/user_mm.rs`

---

## Current String Errors

```rust
"Virtual address not in user space"
"Mapping extends beyond user space"
"Failed to allocate stack page"
"Failed to allocate user page"
"Cannot allocate zero bytes"
"Pointer not in user space"
"Range exceeds user space"
"Pointer range overflow"
"Page not valid"
"Page not accessible to user"
"Page not writable"
"Page not mapped"
```

---

## Changes Required

### 1. Import MmuError

```rust
use levitate_hal::mmu::{self, MmuError, PAGE_SIZE, PageAllocator, PageFlags, PageTable};
```

### 2. Update function signatures

```rust
pub unsafe fn map_user_page(...) -> Result<(), MmuError>
pub unsafe fn map_user_range(...) -> Result<(), MmuError>
pub unsafe fn setup_user_stack(...) -> Result<usize, MmuError>
pub unsafe fn alloc_and_map_user_range(...) -> Result<usize, MmuError>
pub fn validate_user_buffer(...) -> Result<(), MmuError>
```

### 3. Map string errors to MmuError variants

| String Error | MmuError Variant |
|--------------|------------------|
| "Virtual address not in user space" | `InvalidVirtualAddress` |
| "Mapping extends beyond user space" | `InvalidVirtualAddress` |
| "Failed to allocate stack page" | `AllocationFailed` |
| "Failed to allocate user page" | `AllocationFailed` |
| "Cannot allocate zero bytes" | `InvalidVirtualAddress` |
| "Pointer not in user space" | `InvalidVirtualAddress` |
| "Range exceeds user space" | `InvalidVirtualAddress` |
| "Pointer range overflow" | `InvalidVirtualAddress` |
| "Page not valid" | `NotMapped` |
| "Page not accessible to user" | `NotMapped` |
| "Page not writable" | `NotMapped` |
| "Page not mapped" | `NotMapped` |

---

## Verification

1. `cargo build --release`
2. Check callers in `kernel/src/loader/elf.rs` compile

---

## Exit Criteria

- [ ] All `&'static str` replaced with `MmuError`
- [ ] Function signatures updated
- [ ] Build passes
- [ ] Callers updated as needed
