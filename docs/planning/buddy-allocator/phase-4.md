# Phase 4: Integration and Testing

**Status**: [ ] Pending Implementation | [x] In Review | [ ] Approved
**Owner**: TEAM_041
**Reviewed By**: TEAM_042 (2026-01-04)
**Target Hardware**: Pixel 6 (8GB RAM)

## 1. Test Strategy

### 1.1 Boot Validation (Critical)
- **Scenario**: Boot with different RAM sizes in QEMU (`-m 1G`, `-m 2G`).
- **Check**: usage logs show correct available frames.
- **Check**: `mem_map` is placed correctly (not overwriting kernel).

### 1.2 Allocator Logic Tests
- **Unit Tests** (Host-side if possible, or isolated kernel test):
  - Alloc Order 0 -> Check flags.
  - Alloc Order 0 -> Check Buddy split (Order 1 was split).
  - Free -> Check coalescing (Order 0 + Order 0 -> Order 1).
  - Alloc Max Order -> Fail if fragmented.

### 1.3 Stress Testing
- **Torture Test**:
  - Allocate all RAM in random order sizes.
  - Free all RAM.
  - Verify that we can allocate Max Order again (perfect coalescing).
  - *Note*: Any leak or failure to coalesce will fail this test.

## 2. Integration Verification
- Verify `LockedHeap` works correctly on top of the allocated region.
- Verify `mmu` can request pages for new page tables (if migrated).

## 3. Regression Baselines (TEAM_042 Addition)

### 3.1 Boot Behavior Baseline
Create golden output for:
- **1GB RAM**: Expected free frames after init (~250,000 frames minus kernel/initrd/mem_map)
- **2GB RAM**: Expected free frames proportionally scaled
- **Boot log**: Must include `mem_map` placement address and size

### 3.2 Edge Case Tests
- **Scenario**: DTB with no `/memory` node → Expected: Panic with "No RAM regions found"
- **Scenario**: All RAM reserved → Expected: Panic with "No free memory available"
- **Scenario**: RAM < 32MB → Expected: Panic with "Insufficient memory for kernel"

### 3.3 Host-Side Unit Tests
Strategy for testing without `std`:
- Gate tests with `#[cfg(all(test, feature = "std"))]` (like `mmu.rs` does)
- Test buddy logic with mock `mem_map` arrays
- Test split/merge/coalesce with predetermined scenarios
