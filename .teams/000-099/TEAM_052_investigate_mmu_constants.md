# TEAM_052 Investigation: MMU Compilation Failure

**Team:** TEAM_052  
**Created:** 2026-01-04  
**Bug Summary:** Missing MAIR_VALUE and TCR_VALUE constants in MMU module

---

## Bug Report

### Environment
- **Platform:** LevitateOS (ARM64 kernel)
- **Target:** aarch64-unknown-none
- **Discovered during:** cargo xtask test behavior (Step 2 of slab allocator verification)

### Error Messages
```
error[E0425]: cannot find value `MAIR_VALUE` in this scope
   --> levitate-hal/src/mmu.rs:344:21
    |
344 |             in(reg) MAIR_VALUE,
    |                     ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `TCR_VALUE` in this scope
   --> levitate-hal/src/mmu.rs:351:21
    |
351 |             in(reg) TCR_VALUE,
    |                     ^^^^^^^^^ not found in this scope
```

### Reproduction Steps
1. Run `cargo build --target aarch64-unknown-none`
2. Compilation fails with above errors
3. Confirmed via `git diff levitate-hal/src/mmu.rs` - no recent changes to this file
4. Searched codebase: `grep -r "const.*MAIR_VALUE"` - no definitions found

### Impact
- Blocks kernel compilation for aarch64 target
- Prevents behavioral testing
- Unrelated to slab allocator work (pre-existing issue)

---

## Phase 1: Understand the Symptom

### Symptom Description
**Expected:** MMU module should define MAIR_VALUE and TCR_VALUE constants for AArch64 MMU initialization  
**Actual:** Constants are referenced but never defined  
**Delta:** Missing constant definitions

### Location
- **File:** `levitate-hal/src/mmu.rs`
- **Lines:** 344 (MAIR_VALUE), 351 (TCR_VALUE)
- **Function:** `mmu::init()` (lines 338-358)

### Context
From code inspection:
```rust
#[cfg(target_arch = "aarch64")]
pub fn init() {
    unsafe {
        // Configure MAIR_EL1
        core::arch::asm!(
            "msr mair_el1, {}",
            in(reg) MAIR_VALUE,  // <-- UNDEFINED
            options(nostack)
        );

        // Configure TCR_EL1
        core::arch::asm!(
            "msr tcr_el1, {}",
            in(reg) TCR_VALUE,  // <-- UNDEFINED
            options(nostack)
        );
    }
}
```

### Codebase Evidence
- Comments at line 80-82 mention MAIR configuration but don't define the value
- No `const MAIR_VALUE` or `const TCR_VALUE` anywhere in codebase

---

## Phase 2: Form Hypotheses

### Hypothesis 1: Code was refactored and constants were removed
**Confidence:** Medium  
**Evidence needed:** Git history of mmu.rs  
**Would confirm:** Recent commits show deletion of these constants  
**Would refute:** No such deletion in git log

### Hypothesis 2: Constants were never implemented (incomplete feature)
**Confidence:** High  
**Evidence needed:** Git history shows mmu::init() was added without constants  
**Would confirm:** Init function added with placeholder references  
**Would refute:** Constants existed in earlier commits

### Hypothesis 3: Constants should be defined elsewhere and imported
**Confidence:** Low  
**Evidence needed:** Other ARM64 register constant definitions in codebase  
**Would confirm:** Pattern of importing register values from another module  
**Would refute:** All other register values are defined inline

**Priority:** Test Hypothesis 2 first (most likely), then Hypothesis 1

---

## Phase 3: Test Hypotheses

### Testing Hypothesis 2: Never implemented
**Status:** REFUTED

### Testing Hypothesis 1: Code was refactored and constants were removed  
**Status:** ✅ CONFIRMED

**Evidence from git log:**
```
commit 88c75b0 (most recent)
feat: Implement buddy allocator with dynamic memory mapping...

Deleted lines 83-131 from mmu.rs:
- const MAIR_VALUE: u64 = ...
- const TCR_VALUE: u64 = ...
- All constituent constants (TCR_T0SZ, TCR_TG0_4KB, etc.)
```

**Key finding:** Commit message says "Reference value - MMU setup done in assembly" but the Rust `mmu::init()` function (lines 338-358) was **not updated** to reflect this change.

**Assembly setup location:** `kernel/src/main.rs` lines 140-165 has inline assembly that configures MAIR_EL1 and TCR_EL1 directly.

---

## Phase 4: Root Cause

### Root Cause
**Location:** `levitate-hal/src/mmu.rs`, function `init()` (lines 338-358)  
**Issue:** Function references deleted constants `MAIR_VALUE` and `TCR_VALUE`

### Causal Chain
1. Commit 88c75b0 removed MAIR_VALUE and TCR_VALUE constants (lines 83-131)
2. Reason: MMU initialization moved to inline assembly in kernel bootstrap
3. **But:** The Rust `mmu::init()` wrapper function was not updated
4. Result: `mmu::init()` tries to use undefined constants
5. Symptom: Compilation failure when building for aarch64 target

### Related Code Paths
- `kernel/src/main.rs:140-165` - Assembly MMU setup (working)
- `mmu::init()` - Rust wrapper (broken, unused?)

### Question for Investigation
Is `mmu::init()` ever called, or is it dead code?

---

## Phase 5: Decision - Fix or Plan

### Analysis

**Complexity:** ~2 Units of Work  
**Lines of code:** ~25 lines (either restore constants or mark function as stub)  
**Risk:** Low (either restore known-good constants or stub unused function)  
**Confidence:** High (root cause is certain)

**Criteria met for immediate fix:** ✅ Yes

### Fix Options

#### Option A: Restore the deleted constants
- Restore lines 83-131 from commit before 88c75b0
- Pro: Function works as originally intended
- Con: Duplicates assembly configuration

#### Option B: Make init() a no-op stub  
- Comment says "MMU setup done in assembly"
- Change function body to empty (no-op)
- Pro: Matches current architecture (assembly setup)
- Con: Function exists but does nothing (confusing)

#### Option C: Remove init() entirely
- Delete the function if it's dead code
- Pro: Cleanest solution if unused
- Con: Might break external callers

**Recommended:** Check if `mmu::init()` is called anywhere. If not, choose Option C. If called, choose Option B (stub).

---

## Fix Implementation

### Decision: Option B - Stub the Function

**Rationale:**
- `mmu::init()` IS called from `kernel/src/main.rs:285`
- Cannot remove (Option C) - would break caller
- Stubbing is correct - MMU registers already configured in assembly

### Changes Made

**File:** `levitate-hal/src/mmu.rs` (lines 337-350)

**Before:**
```rust
#[cfg(target_arch = "aarch64")]
pub fn init() {
    unsafe {
        core::arch::asm!("msr mair_el1, {}", in(reg) MAIR_VALUE, ...);
        core::arch::asm!("msr tcr_el1, {}",  in(reg) TCR_VALUE, ...);
    }
}
```

**After:**
```rust
/// Initialize MMU registers (MAIR, TCR). Does NOT enable MMU.
/// 
/// # TEAM_052: Stubbed Function
/// This function is a no-op because MAIR_EL1 and TCR_EL1 are configured
/// in the assembly bootstrap code (kernel/src/main.rs lines 148-165).
#[cfg(target_arch = "aarch64")]
pub fn init() {
    // MMU registers already configured by assembly bootstrap
}
```

---

## Verification

### Compilation Test
```bash
$ cargo build --target aarch64-unknown-none
✅ SUCCESS - No compilation errors
```

### Behavioral Test
```bash
$ cargo xtask test behavior
✅ SUCCESS: Current behavior matches Golden Log.
```

---

## Handoff Checklist

- [x] Root cause identified and documented
- [x] Fix implemented and tested
- [x] Kernel compiles successfully
- [x] Behavioral tests pass
- [x] Team file updated with full investigation
- [x] Breadcrumbs placed in code (TEAM_052 comment in mmu.rs)

---

## Summary

**Bug:** Missing MAIR_VALUE and TCR_VALUE constants caused compilation failure  
**Root Cause:** Constants deleted in commit 88c75b0 but `mmu::init()` not updated  
**Fix:** Stubbed function body since MMU config done in assembly  
**Impact:** Unblocks kernel compilation and behavioral testing  
**Related Work:** Slab allocator integration can now proceed with Step 2 verification

**Status:** ✅ RESOLVED
