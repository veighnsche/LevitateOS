# TEAM_181: Investigate LosAllocator Edge Cases

## Bug Report
- **Component**: `LosAllocator` (bump allocator) + `sbrk` syscall
- **Task**: Proactively find and fix edge cases/bugs
- **Source**: User marked as complete, requested investigation
- **Status**: ✅ COMPLETED - Fixed 5 edge cases

## Files Modified
- `userspace/ulib/src/alloc.rs` - LosAllocator implementation
- `kernel/src/syscall/mm.rs` - sys_sbrk kernel implementation  

## Bugs Found & Fixed

### Bug 1: Integer overflow in `alloc()` - FIXED
**Location**: `userspace/ulib/src/alloc.rs:85-86`
**Problem**: `*head + align - 1` and `aligned + layout.size()` could overflow on large allocations
**Fix**: Added `checked_add()` for all arithmetic operations

### Bug 2: Zero-size allocation returns null - FIXED
**Location**: `userspace/ulib/src/alloc.rs:79-105`
**Problem**: When `head=0` and `layout.size()=0`, the function returned `ptr=0` (null)
**Fix**: Added early return for zero-size allocations: `return layout.align() as *mut u8`
**Contract**: GlobalAlloc requires zero-size allocations to return non-null, well-aligned pointer

### Bug 3: Integer overflow in `grow()` - FIXED
**Location**: `userspace/ulib/src/alloc.rs:49-50`
**Problem**: `min_size + PAGE_SIZE - 1` could overflow, also `pages_needed * PAGE_SIZE`
**Fix**: Added `checked_add()` and `checked_mul()` with early return on overflow

### Bug 4: Missing bounds check after grow - FIXED
**Location**: `userspace/ulib/src/alloc.rs:89-99`
**Problem**: After `grow()`, code recomputed `aligned/new_head` but didn't verify `new_head <= *end`
**Fix**: Added explicit bounds check after grow: `if new_head > *end { return null_mut(); }`

### Bug 5: Kernel-side integer overflow - FIXED
**Location**: `kernel/src/syscall/mm.rs:15`
**Problem**: `new_break + PAGE_SIZE - 1` could overflow when calculating page count
**Fix**: Added `checked_add()` with rollback on overflow

## Hypotheses Evaluated

| Hypothesis | Status | Notes |
|------------|--------|-------|
| H1: Large alignment overflow | CONFIRMED | Fixed with checked arithmetic |
| H2: Zero-size allocation | CONFIRMED | Fixed with early return for non-null dangling ptr |
| H3: sbrk return 0 ambiguity | NOT A BUG | Heap base comes from ELF brk, always > 0 |
| H4: Negative sbrk shrinking | DEFERRED | Works at kernel level, allocator never shrinks (by design) |
| H5: Integer overflow | CONFIRMED | Fixed in both alloc() and grow() |

## Verification
- [x] Full build succeeds (`cargo xtask build all`)
- [x] All crate tests pass (los_error, los_hal, los_pci, los_gpu)
- [x] No regressions introduced

## Handoff Notes
The bump allocator is now hardened against:
- Integer overflow in alignment calculations
- Integer overflow in size calculations  
- Zero-size allocation edge case (GlobalAlloc contract compliance)
- Kernel-side overflow in page mapping loop

Future considerations:
- The heap can never shrink (bump allocator limitation) - acceptable for now
- Memory is only reclaimed on process exit - documented behavior

