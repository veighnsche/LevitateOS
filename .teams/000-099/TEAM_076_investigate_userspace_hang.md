# TEAM_076: Investigate Userspace Process Hang

## Status
**ROOT CAUSE FOUND** - Requires bugfix plan for device MMIO remapping

## Bug Report

**Symptom:** Kernel hangs after `[SPAWN] Starting user process...`

**Expected:** User process executes and prints output (e.g., "Hello from userspace!")

**Actual:** System hangs indefinitely after spawning

**Reproduction:** `cargo xtask run` - boots kernel, loads ELF, creates user process, then hangs

---

## Phase 1: Symptom Analysis

### Last Known Good Output
```
[SPAWN] Created user process PID=1 entry=0x10000 sp=0x7fffffff0000
[SPAWN] Starting user process...
<hang>
```

### Code Areas to Investigate
1. `kernel/src/task/process.rs` - `run_from_initramfs()` 
2. `kernel/src/task/user.rs` - `UserTask` creation and execution
3. Context switch / ERET to userspace
4. Exception vectors (EL0 -> EL1 transitions)

---

## Hypotheses

### H1: Page table entry doesn't allow EL0 execution (HIGH confidence)
- **Evidence needed:** Check if USER_CODE flags are correctly set
- **Confirm:** Add debug print after ERET fails - if silent, likely page fault
- **Refute:** If syscall prints appear, code is executing

### H2: Entry point mismatch - code loaded at wrong VA (MEDIUM confidence)
- **Evidence needed:** Compare ELF entry point with actual mapped VA
- **Confirm:** Entry point vs `_start` location in ELF
- **Refute:** ELF parsing logs show correct entry

### H3: TTBR0 switch incomplete - stale TLB (LOW confidence)
- **Evidence needed:** Check if TLB flush is happening
- **Confirm:** `switch_ttbr0` includes `tlbi` instruction
- **Refute:** Code already has TLB invalidation

### H4: Silent page fault - no output from fault handler (HIGH confidence)
- **Evidence needed:** Check if exception from EL0 prints error
- **Confirm:** If we see "USER EXCEPTION" message, it's a page fault
- **Refute:** If syscall works, no fault

### H5: User code is stuck in infinite loop before syscall (LOW confidence)
- **Evidence needed:** Check if _start code has any issues
- **Confirm:** Unlikely given simple code
- **Refute:** Code looks straightforward

---

## Evidence Log

### Test 1: Debug prints around switch_ttbr0
**Output:**
```
[SPAWN] Starting user process...
[TASK] Before switch_ttbr0(0x48011000)
<hang>
```

**Conclusion:** The hang occurs DURING `switch_ttbr0()`, not after ERET.

### Test 2: Analyze UART address
- UART address: `0x0900_0000`
- `phys_to_virt(0x0900_0000)` returns `0x0900_0000` (identity mapping)
- Identity mappings use TTBR0 (VA < 0x8000_0000_0000)
- **After switch_ttbr0(), UART is unmapped**

### Root Cause Confirmed
The `println!` after `switch_ttbr0()` accesses UART at `0x0900_0000`.
This address is no longer mapped because TTBR0 now points to user page table.
The unmapped access causes a page fault → hang.

---

## Breadcrumbs

- `// TEAM_076 BREADCRUMB: CONFIRMED` in `kernel/src/task/process.rs:75` - Documents the root cause

---

## Root Cause

**File:** `levitate-hal/src/mmu.rs` and `levitate-hal/src/console.rs`

**Issue:** Device MMIO (UART at `0x0900_0000`) uses identity mapping via TTBR0.
When TTBR0 is switched to user page table, device access faults.

**Causal Chain:**
1. `switch_ttbr0(user_page_table)` is called
2. TTBR0 now points to user L0 table (no device mappings)
3. Next `println!` tries to write to UART at VA `0x0900_0000`
4. VA lookup via TTBR0 fails → Translation Fault
5. Exception handler tries to print → same fault → hang

---

## Solution Options

### Option A: Map devices via TTBR1 (Correct Fix)
Change device mapping to use high VA (e.g., `0xFFFF_8000_0900_0000`).
This keeps devices accessible regardless of TTBR0 state.

**Scope:** ~50-100 lines across mmu.rs, console.rs, and device drivers

### Option B: Add devices to user page tables (Workaround)
Map UART etc. in every user page table.

**Problem:** Security issue - user can access devices directly.

### Option C: Don't print after switch_ttbr0 (Temporary)
Remove debug prints, but syscall handler still needs console.

**Problem:** Doesn't fix the real issue.

---

## Recommendation

**Create a bugfix plan** for Option A (map devices via TTBR1).

This is > 5 UoW and touches multiple files. Requires:
1. Update `phys_to_virt()` for device addresses
2. Add device region to TTBR1 page tables during boot
3. Update all device drivers to use high VA
4. Test all device access paths

---

## Handoff

**Investigation complete.** Root cause confirmed.

**Bugfix plan created:** `docs/planning/bugfix-device-mmio-ttbr1/plan.md`

Next team should:
1. Review and approve the bugfix plan
2. Implement the fix (~11 UoW)
3. Verify userspace execution works
4. Remove the CONFIRMED breadcrumb after fix is verified
