# Phase 4 — Implementation and Tests

**TEAM_131 → TEAM_132 → TEAM_133** | Reduce Unsafe Code via Safe Abstractions

## Implementation Steps

### Step 1: Add Dependencies (Low complexity) ✅ COMPLETED

**Files:** `levitate-hal/Cargo.toml`, `kernel/Cargo.toml`

**Tasks:**
1. ✅ Add `aarch64-cpu = "11.2"` to levitate-hal — TEAM_132
2. ✅ Add `intrusive-collections = "0.10"` to levitate-hal — TEAM_133
3. ✅ Verify `safe-mmio` is available
4. ✅ Run `cargo check --all-targets`

**UoW Size:** ~10 lines, 1 session

---

### Step 2: Migrate Barriers (Low complexity) ✅ COMPLETED

**Files:** `levitate-hal/src/gic.rs`, `levitate-hal/src/mmu.rs`, `kernel/src/exceptions.rs`

**Tasks:** — All completed by TEAM_132
1. ✅ Replace `asm!("dsb sy")` with `barrier::dsb(SY)` in mmu.rs
2. ✅ Replace `asm!("isb")` with `barrier::isb(SY)` in mmu.rs, gic.rs
3. ✅ Replace `asm!("wfi")` with `aarch64_cpu::asm::wfi()` in exceptions.rs
4. ✅ Run `cargo check --all-targets`

**UoW Size:** ~20 lines changed, 1 session

---

### Step 3: Migrate System Registers (Medium complexity) 🔄 PARTIAL

**Files:** Multiple in levitate-hal and kernel

**Tasks:**
1. ✅ Migrate `levitate-hal/src/interrupts.rs` (DAIF) — TEAM_132
2. ⏸️ `levitate-hal/src/mmu.rs` — Inherently unsafe, sequence-critical (TTBR+ISB atomic)
3. ⏸️ `levitate-hal/src/timer.rs` — Future work
4. ⏸️ `levitate-hal/src/gic.rs` (ICC_* registers) — Not in aarch64-cpu
5. ✅ `kernel/src/exceptions.rs` (ESR_EL1, ELR_EL1, VBAR_EL1) — TEAM_133
6. ✅ Run full test suite

**Notes:**
- TTBR/SCTLR operations are inherently unsafe (sequence-critical with ISB)
- ICC_* GICv3 system registers are not in aarch64-cpu crate
- Timer registers (CNT*) are future work

**UoW Size:** ~100 lines changed, 2-3 sessions

---

### Step 4: Migrate Intrusive Lists (Medium complexity) ⏸️ DEFERRED

**Files:** `levitate-hal/src/allocator/buddy.rs`, `levitate-hal/src/allocator/slab/list.rs`

**Tasks:**
1. Add `LinkedListLink` to `Page` struct
2. Create adapter with `intrusive_adapter!` macro
3. Replace manual linked list ops with `LinkedList` API
4. Run allocator unit tests

**UoW Size:** ~150 lines changed, 2 sessions

**⚠️ TEAM_133 Finding:** This step is more complex than estimated:
- `LinkedList::new()` with `UnsafeRef` adapters has const initialization issues
- Pages live in static memory map, not owned via Box
- Requires complete rewrite of add_to_list/remove_from_list operations
- **Recommendation:** Needs dedicated session, consider lazy initialization pattern

---

## Test Plan

### Required Test Commands

After **each implementation step**, run:

```bash
# 1. Unit tests (must pass)
cargo xtask test unit

# 2. Regression tests (check for new regressions only)
cargo xtask test regress

# 3. Behavior tests (golden file verification)
cargo xtask test behavior
```

### Pass Criteria
- All existing tests must pass after each step
- No NEW regressions introduced (pre-existing failures are documented)
- Behavior tests in `tests/golden_boot.txt` must match

### New Tests

| Module | Test |
|--------|------|
| `barrier` | Compile-only (barriers have no observable effect in tests) |
| `volatile` | Unit tests for read/write semantics |
| `sysreg` | Compile-only for aarch64, stub for host |
| `intrusive_list` | Unit tests for push/pop/remove operations |

### Unsafe Audit

After each step, run:
```bash
grep -rn "unsafe {" kernel/ levitate-hal/ levitate-virtio/ --include="*.rs" | wc -l
```

Track reduction from baseline of 148.

**Progress:** 135 → 133 (as of TEAM_133)

---

## Step Dependencies

```
Step 1 (deps) ──> Step 2 (barriers) ──> Step 3 (sysregs) ──> Step 4 (intrusive lists)
```

Steps are sequential — each depends on previous completing successfully.
