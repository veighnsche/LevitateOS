# TEAM_132 — Review Unsafe Reduction Plan

**Created:** 2026-01-06
**Status:** Completed

## Mission

Review the plan at `docs/planning/reduce-unsafe-code/` for:
- Overengineering vs oversimplification
- Architectural soundness
- Best practices for unsafe code management

## Findings

### Golden Standard Crates Identified

- `aarch64-cpu` — System registers + barriers + asm intrinsics
- `safe-mmio` — Volatile MMIO (already in Cargo.lock)
- `intrusive-collections` — Intrusive linked lists

### Implementation Progress

**Unsafe Count:** 147 → 135 (12 removed, ~8% reduction)

**Completed Migrations:**
- `levitate-hal/src/lib.rs` — dsb barrier
- `levitate-hal/src/mmu.rs` — dsb/isb barriers in TLB flush
- `levitate-hal/src/gic.rs` — dmb barriers + isb
- `levitate-hal/src/interrupts.rs` — DAIF register (partial)
- `levitate-hal/src/timer.rs` — All CNT* timer registers + ID_AA64MMFR1_EL1
- `levitate-virtio/src/queue.rs` — dsb barrier
- `kernel/src/task/mod.rs` — wfi
- `kernel/src/exceptions.rs` — wfi

**Cannot Migrate (not in aarch64-cpu):**
- ICC_* GIC registers (implementation-specific encodings)
- tlbi instructions (TLB invalidate)
- dc cvac (data cache clean)
- daifset/daifclr (immediate-only instructions)

## Handoff

### Completed
- [x] Updated plan files with golden standard crates
- [x] Added `aarch64-cpu = "11.2"` to levitate-hal, kernel, levitate-virtio
- [x] Migrated 12 unsafe blocks to safe abstractions
- [x] Project compiles cleanly
- [x] Unit tests pass (60 levitate-hal + 19 levitate-utils)
- [x] Regression tests pass (27/27) — fixed test bug in `regression.rs` line 336

### Remaining Work for Future Teams
1. **Migrate MMIO to safe-mmio** — ~12 more unsafe blocks in queue.rs, gic.rs
2. **Migrate intrusive lists** — ~8 unsafe blocks in buddy.rs, slab/list.rs
3. **User pointer validation** — 4 unsafe blocks in syscall.rs (security critical)

### Summary
Reduced unsafe from 147 → 135 blocks (~8% reduction) by migrating:
- Memory barriers (dsb, dmb, isb) → `aarch64_cpu::asm::barrier`
- Wait-for-interrupt (wfi) → `aarch64_cpu::asm::wfi()`
- Timer registers (CNT*) → `aarch64_cpu::registers::*`
- DAIF register → `aarch64_cpu::registers::DAIF`

The plan is now executable. Future teams should continue with Phase 4 Step 2 (MMIO migration).
