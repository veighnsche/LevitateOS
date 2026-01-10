# Phase 1: Discovery — x86_64 MMU Completion

## Feature Summary

**Problem Statement:** TEAM_267 implemented the core x86_64 MMU infrastructure (multiboot2 parsing, PMO expansion, huge pages, buddy allocator integration), but left three TODOs that prevent the x86_64 kernel from being fully functional:

1. **Missing linker symbols** — The refined kernel segment permissions code references symbols (`__text_start`, `__rodata_start`, etc.) that don't exist in the x86_64 linker script
2. **Missing heap initialization** — `kernel_main` calls `init_x86_64()` which uses heap allocations before the heap is initialized
3. **Pre-existing test crash** — `test_irq_safe_lock_behavior` crashes with SIGSEGV

**Who Benefits:** Anyone working on x86_64 support for LevitateOS.

## Success Criteria

- [ ] x86_64 kernel builds without linker errors
- [ ] x86_64 kernel boots and initializes memory correctly
- [ ] HAL unit tests pass (including `test_irq_safe_lock_behavior`)
- [ ] Kernel segment permissions are correctly applied

---

## Current State Analysis

### TODO 1: Linker Script Symbols

**Current State:**
- `crates/hal/src/x86_64/mmu.rs` references these symbols:
  - `__text_start`, `__text_end`
  - `__rodata_start`, `__rodata_end`
  - `__data_start`, `__data_end`
  - `__bss_start`, `__bss_end`
- These are used in `init_kernel_mappings_refined()` for per-segment permissions
- The aarch64 linker script (`linker.ld`) defines similar symbols

**Workaround:** Currently using `init_kernel_mappings()` which maps everything as KERNEL_DATA.

### TODO 2: Heap Initialization Order

**Current State:**
- `kernel_main` in `kernel/src/arch/x86_64/mod.rs` calls:
  1. `los_hal::x86_64::init()` — HAL initialization
  2. Multiboot2 parsing
  3. PMO expansion
  4. `crate::memory::init_x86_64()` — Uses `println!` and allocator
- Problem: `init_x86_64()` needs the heap, but heap isn't initialized yet
- aarch64 uses `linked_list_allocator::LockedHeap` initialized in `kmain`

**Workaround:** None — this will crash if `init_x86_64()` allocates.

### TODO 3: Pre-existing Test Crash

**Current State:**
- `tests::test_irq_safe_lock_behavior` crashes with SIGSEGV
- Confirmed pre-existing (crashes without TEAM_267 changes)
- Test is in `crates/hal/src/lib.rs` lines 162-178
- Tests `IrqSafeLock` which calls `interrupts::disable()`/`interrupts::restore()`

**Likely Cause:** The interrupt mock implementation may have issues when running on host.

---

## Codebase Reconnaissance

### Files Involved

| File | Purpose | TODOs |
|------|---------|-------|
| `linker.ld` (root) | aarch64 linker script | Reference for symbols |
| `kernel/linker.ld` | Alternative linker location | Check if used |
| `kernel/src/arch/x86_64/mod.rs` | x86_64 entry point | TODO 2 |
| `kernel/src/arch/x86_64/boot.rs` | Boot stubs | TODO 2 |
| `crates/hal/src/x86_64/mmu.rs` | MMU implementation | TODO 1 |
| `crates/hal/src/interrupts.rs` | Interrupt mock | TODO 3 |
| `crates/hal/src/lib.rs` | IrqSafeLock tests | TODO 3 |

### Existing Patterns

**aarch64 linker script pattern:**
```ld
.text : {
    __text_start = .;
    *(.text .text.*)
    __text_end = .;
}
```

**aarch64 heap initialization pattern:**
```rust
// In kmain
static ALLOCATOR: LockedHeap = LockedHeap::empty();
unsafe { ALLOCATOR.lock().init(heap_start, heap_size); }
```

---

## Constraints

1. **No breaking changes to aarch64** — x86_64 work must not regress aarch64
2. **Consistent patterns** — Use same patterns as aarch64 where possible
3. **Test isolation** — Host tests must not depend on real hardware features

---

## Phase 1 Exit Criteria

- [x] All three TODOs documented with current state
- [x] Relevant files identified
- [x] Existing patterns documented
- [x] Constraints identified
- [ ] → Proceed to Phase 2: Design
