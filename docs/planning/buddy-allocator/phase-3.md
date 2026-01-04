# Phase 3: Buddy Allocator Implementation

**Status**: [ ] Pending Design Approval
**Owner**: TEAM_041

## 1. Implementation Overview

### Step 1: Core Data Structures
- Create `levitate-kernel/src/memory/buddy.rs`.
- Implement `FreeBlock` (intrusive list node).
- Implement `BuddyAllocator` struct.
- Unit tests for logic (alloc/split/merge) using dummy memory arrays.

### Step 2: Global Instance
- Create `static ALLOCATOR: WrappedLock<BuddyAllocator>` in `kernel/src/memory.rs` (or similar).
- Initialize it in `kmain`.

### Step 3: Region Injection
- Update `kmain` to calculate free RAM ranges (end of kernel -> end of RAM).
- Feed these ranges into the allocator.

### Step 4: MMU Integration
- Update `levitate-hal/src/mmu.rs` to use `ALLOCATOR` instead of `PT_POOL`?
- *Note*: This introduces a dependency from HAL to Kernel, which might violate architecture.
- *Correction*: The Allocator should perhaps live in `levitate-hal` or `levitate-utils`?
  - `linked_list_allocator` is external.
  - Implementation likely belongs in `levitate-mm` (new crate?) or `kernel`. Use of Traits can invert dependency.

## 2. Dependencies
- Resolution of Phase 2 Design (Location of code).
