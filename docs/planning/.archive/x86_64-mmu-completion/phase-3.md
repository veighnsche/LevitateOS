# Phase 3: Implementation — x86_64 MMU Completion

## Overview

This phase implements the three solutions designed in Phase 2:
1. x86_64 linker script with segment symbols
2. Heap initialization order fix
3. IrqSafeLock test fix

Each solution is a separate step with its own UoW.

---

## Step 1: Create x86_64 Linker Script

**File:** `phase-3-step-1.md`

### UoW 1.1: Create Linker Script

**Goal:** Create `kernel/src/arch/x86_64/linker.ld` with all required segment symbols.

**Tasks:**
1. Create `kernel/src/arch/x86_64/linker.ld` based on Phase 2 design
2. Define symbols: `__text_start`, `__text_end`, `__rodata_start`, `__rodata_end`, `__data_start`, `__data_end`, `__bss_start`, `__bss_end`
3. Define heap symbols: `__heap_start`, `__heap_end`
4. Set proper entry point and output format

**Exit Criteria:**
- [ ] Linker script file exists
- [ ] All required symbols defined
- [ ] Follows aarch64 pattern for consistency

### UoW 1.2: Integrate Linker Script with Build System

**Goal:** Update xtask to use the x86_64 linker script.

**Tasks:**
1. Check `xtask/src/build.rs` for linker script handling
2. Add conditional logic to select arch-specific linker script
3. Test that `cargo xtask build --arch x86_64` uses correct script

**Exit Criteria:**
- [ ] Build system selects correct linker script per architecture
- [ ] x86_64 build finds the new linker script

---

## Step 2: Fix Heap Initialization Order

**File:** `phase-3-step-2.md`

### UoW 2.1: Refactor kernel_main Initialization Order

**Goal:** Initialize heap before any allocating code.

**Tasks:**
1. In `kernel/src/arch/x86_64/mod.rs`, reorder `kernel_main`:
   - First: Serial init (no alloc)
   - Second: Heap init
   - Third: Full HAL init
   - Fourth: Multiboot2 parsing
   - Fifth: PMO expansion
   - Sixth: Buddy allocator init
2. Create `init_heap()` function using linker symbols
3. Ensure `ALLOCATOR` is accessible from boot code

**Exit Criteria:**
- [ ] `kernel_main` doesn't allocate before heap init
- [ ] Heap initialized from linker-defined region
- [ ] Serial output works before heap init

### UoW 2.2: Update boot.rs with Heap Allocator

**Goal:** Define and initialize the global allocator in x86_64 boot code.

**Tasks:**
1. Verify `ALLOCATOR` in `kernel/src/arch/x86_64/boot.rs` is correctly defined
2. Implement `init_heap()` function
3. Add linker symbol externs

**Exit Criteria:**
- [ ] `#[global_allocator]` correctly defined
- [ ] `init_heap()` initializes allocator from linker symbols

---

## Step 3: Fix IrqSafeLock Test

**File:** `phase-3-step-3.md`

### UoW 3.1: Investigate Test Crash

**Goal:** Identify root cause of SIGSEGV in `test_irq_safe_lock_behavior`.

**Tasks:**
1. Read `crates/hal/src/lib.rs` test code
2. Read `crates/hal/src/interrupts.rs` mock implementation
3. Run test with `RUST_BACKTRACE=1` to get crash location
4. Identify the unsafe operation causing the crash

**Exit Criteria:**
- [ ] Root cause identified
- [ ] Fix approach confirmed

### UoW 3.2: Fix Interrupt Mock Implementation

**Goal:** Make the interrupt mock safe for test execution.

**Tasks:**
1. Update `crates/hal/src/interrupts.rs` mock implementation
2. Ensure atomic operations are correct
3. Verify no undefined behavior in test context
4. Run `cargo test --package los_hal --features std` to verify fix

**Exit Criteria:**
- [ ] `test_irq_safe_lock_behavior` passes
- [ ] `test_irq_safe_lock_nested` passes
- [ ] All HAL tests pass

---

## Implementation Order

Execute UoWs in this order:

1. **UoW 3.1 + 3.2** (Test fix) — Unblocks verification
2. **UoW 1.1** (Linker script) — Creates required symbols
3. **UoW 1.2** (Build integration) — Enables x86_64 builds
4. **UoW 2.1 + 2.2** (Heap init) — Completes boot sequence

**Rationale:** Fix tests first so we can verify other changes.

---

## Phase 3 Exit Criteria

- [ ] All UoWs completed
- [ ] HAL tests pass
- [ ] x86_64 kernel builds without linker errors
- [ ] → Proceed to Phase 4: Integration & Testing
