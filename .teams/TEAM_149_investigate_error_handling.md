# TEAM_149: Investigate Error Handling

**Status:** Complete  
**Created:** 2026-01-06  
**Task:** Investigate poor error handling across LevitateOS kernel

## Bug Report

User complaint: "We need WAAAAY BETTER ERROR HANDLING!!! MAYBE EVEN AN ENTIRE NUMBER SYSTEM!!!"

## Phase 1: Symptom Understanding

### Expected Behavior
- Unified error type system across the kernel
- Consistent error codes/numbers for debugging
- Proper error propagation with context
- Type-safe error handling following Rust idioms

### Actual Behavior (Initial Observations)
From code search, identified multiple fragmented error patterns:

1. **`&'static str` errors** - Many functions return `Result<T, &'static str>`
   - `fs/mod.rs`, `fs/fat.rs` - "Failed to open FAT volume", etc.
   - `task/user_mm.rs` - "Virtual address not in user space"

2. **Scattered error enums** - Each module defines its own:
   - `syscall.rs::errno` - POSIX-style negative i64 values
   - `loader/elf.rs::ElfError` - parsing errors
   - `task/process.rs::SpawnError` - process creation
   - `net.rs::NetError` - network errors

3. **No unified error numbering** - Different subsystems use different schemes

## Phase 2: Investigation Plan

- [ ] Catalog all error types across kernel
- [ ] Identify patterns and anti-patterns
- [ ] Assess scope of changes needed
- [ ] Determine if immediate fix or bugfix plan needed

## Findings

### Error Type Inventory

| Location | Error Type | Scheme | Issues |
|----------|-----------|--------|--------|
| `syscall.rs::errno` | `i64` constants | POSIX-like negatives (-1 to -4) | Only 4 codes, no enum |
| `loader/elf.rs` | `enum ElfError` | Rust enum (9 variants) | No numeric codes, no Display |
| `task/process.rs` | `enum SpawnError` | Rust enum (3 variants) | Loses inner error context |
| `net.rs` | `enum NetError` | Rust enum (3 variants) | No numeric codes |
| `fs/mod.rs` | `&'static str` | String literals | No type safety |
| `fs/fat.rs` | `&'static str` | String literals | No type safety |
| `task/user_mm.rs` | `&'static str` | String literals (11 variants) | No type safety |
| `levitate-hal/mmu.rs` | `&'static str` | String literals | No type safety |
| `levitate-hal/fdt.rs` | `enum FdtError` | Rust enum (2 variants) | Good pattern |

### Anti-Patterns Identified

1. **`&'static str` for errors** (11+ locations)
   - No type safety
   - No error codes for debugging
   - Can't match on specific errors
   - Examples: `"Virtual address not in user space"`, `"Page not mapped"`

2. **Context loss in error conversions**
   ```rust
   impl From<ElfError> for SpawnError {
       fn from(_e: ElfError) -> Self {
           SpawnError::ElfError  // LOSES the actual ElfError variant!
       }
   }
   ```

3. **Inconsistent error handling**
   - `block.rs`: Uses `panic!` on errors (Rule 6 violation)
   - `net.rs`: Returns custom enum
   - `fs/fat.rs`: Uses `Option` and `&'static str` inconsistently

4. **No unified numbering system**
   - `errno` has -1 to -4
   - `ElfError` has no codes
   - Debugging requires string matching

### Root Cause

**No unified error architecture was designed upfront.** Each subsystem invented its own approach organically, leading to:
- Fragmented error types
- Lost error context
- String-based errors without type safety
- No error codes for debugging/tracing

### Severity: **HIGH**

- Affects kernel stability (panics in block.rs)
- Makes debugging difficult (no error codes)
- Violates Rust idioms (Rule 6: Robust Error Handling)
- Technical debt accumulating across modules

## Decision

**Create a bugfix plan.** This is:
- **> 5 Units of Work** - Requires touching 14+ files across 2 crates
- **> 50 lines of code** - Full error type system + conversions + Display impls
- **Medium-high risk** - Touches core kernel error paths
- **Needs design first** - Error numbering scheme must be decided

### Recommended Architecture (for bugfix plan)

1. **Unified `KernelError` enum** with numeric error codes
2. **Error code numbering system** (e.g., 0x1xxx for ELF, 0x2xxx for MMU, etc.)
3. **Proper `From` impls** preserving inner error context
4. **`Display` and `Debug` impls** for logging
5. **Replace all `&'static str`** with typed errors
6. **Remove panics** from recoverable error paths

## Panic Point Inventory

### Summary

| Category | Count | Fixable? |
|----------|-------|----------|
| **Kernel panics** | 6 | 4 yes, 2 no |
| **HAL panics** | 52 | ~10 yes, rest in tests/invariants |
| **levitate-virtio** | 6 | 4 yes |
| **Test code** | ~25 | N/A (tests) |

### CRITICAL: Production Panics That MUST Be Fixed

#### 1. `kernel/src/block.rs` - **4 panics** ⚠️ HIGH PRIORITY
```rust
// Line 41: Err(e) => panic!("Failed to read block {}: {:?}", block_id, e)
// Line 44: panic!("Block device not initialized")
// Line 59: Err(e) => panic!("Failed to write block {}: {:?}", block_id, e)
// Line 62: panic!("Block device not initialized")
```
**Fix:** Return `Result<(), BlockError>` instead of panicking.

#### 2. `levitate-virtio/src/hal_impl.rs` - **4 panics** ⚠️ HIGH PRIORITY
```rust
// Line 20: Layout::from_size_align(...).unwrap()
// Line 23: panic!("VirtIO DMA allocation failed")
// Line 27: NonNull::new(ptr).unwrap()
// Line 31: Layout::from_size_align(...).unwrap()
// Line 38: NonNull::new(...).unwrap()
```
**Fix:** DMA failures could return error codes through VirtioHal trait.

#### 3. `kernel/src/boot.rs` - **5 unwrap + 4 expect** ⚠️ MEDIUM PRIORITY
```rust
// Lines 282, 297, 307, 319, 329: .unwrap() on mmu::map_range
// Lines 248, 258, 269, 334: .expect() on mapping functions
```
**Assessment:** Boot failures are unrecoverable - **KEEP these panics** but improve messages.

#### 4. `kernel/src/memory/mod.rs` - **1 panic + 2 expect**
```rust
// Line 139: panic!("Failed to allocate physical memory for mem_map!")
// Line 27: .expect("Invalid DTB for memory init")
// Line 221: .expect("Checked None above")  // Safe - guarded by if
```
**Assessment:** Memory init failure is unrecoverable - **KEEP** but improve messages.

#### 5. `kernel/src/task/mod.rs` - **1 expect**
```rust
// Line 96: .expect("current_task() called before scheduler init")
```
**Assessment:** Programming error if called too early - **KEEP** (debug_assert style).

### Acceptable Panics (Invariant Violations)

Per Rule 14 (Fail Loud, Fail Fast), these are CORRECT:

| File | Count | Reason |
|------|-------|--------|
| `levitate-hal/allocator/buddy.rs` | 15 | Corrupted allocator state = invariant violation |
| `levitate-hal/allocator/slab/*.rs` | 13 | Corrupted slab state = invariant violation |
| `levitate-hal/allocator/intrusive_list.rs` | 1 | Impossible null from NonNull |
| `kernel/src/task/user.rs:104` | 1 | Architecture stub - impossible on aarch64 |

### Safe Unwraps (Infallible Operations)

| File | Count | Reason |
|------|-------|--------|
| `kernel/src/loader/elf.rs` | 14 | `try_into().unwrap()` after size check - infallible |
| `kernel/src/virtio.rs:49` | 1 | NonNull on known non-null constant address |

### Test Code (Ignore)

~25 instances in `#[cfg(test)]` modules - panics in tests are fine.

---

### Fixability Assessment

| Priority | File | Issue | Fix Complexity |
|----------|------|-------|----------------|
| **P0** | `block.rs` | Panics on I/O error | Low - return Result |
| **P0** | `levitate-virtio/hal_impl.rs` | Panics on DMA fail | Medium - trait change |
| **P1** | `boot.rs` | Unwrap on map fail | Low - just improve messages |
| **P2** | `memory/mod.rs` | Panic on OOM | Keep - unrecoverable |

### Recommendation

**Yes, we can fix most critical panics!**

1. **`block.rs`** - Easy fix (~30 lines)
   - Change return type to `Result<(), BlockError>`
   - Callers handle errors gracefully

2. **`levitate-virtio/hal_impl.rs`** - Medium fix
   - VirtioHal trait doesn't support errors well
   - May need wrapper or custom panic handler

3. **Boot/Memory panics** - Keep but improve
   - These ARE unrecoverable
   - Add better error messages with context

---

## Handoff

**Plan created:** `docs/planning/unified-error-system/plan.md`

Next team should:
1. Review the plan with USER
2. Run `/make-a-bugfix-plan` workflow to refine if needed
3. Begin implementation starting with `levitate-error` crate

### Handoff Checklist

- [x] Project builds cleanly
- [x] Investigation complete
- [x] Team file updated
- [x] Bugfix plan created
- [x] No code changes made (investigation only)
