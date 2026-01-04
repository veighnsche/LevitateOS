# Phase 3 — Implementation

**Feature**: Reference Kernel Improvements  
**Team**: TEAM_043  
**Status**: ✅ READY FOR IMPLEMENTATION  
**Parent**: phase-2.md

---

## Implementation Order

Improvements implemented in dependency order:

```
1. FDT Parsing (foundation)
   ↓
2. GICv3 Detection (depends on FDT)
   ↓
3. InterruptHandler Trait (can parallel with 2)
   ↓
4. bitflags! Crate (independent)
   ↓
5. VHE Detection (independent)
```

---

## Step 1: FDT Parsing

**File**: `phase-3-step-1.md` (or UoWs if too large)

### Tasks

1. ~~Add `fdt` crate to `levitate-hal/Cargo.toml`~~ ✅ DONE (TEAM_039)
2. ~~Create `levitate-hal/src/fdt.rs` module~~ ✅ DONE (TEAM_039)
3. ~~Implement `parse()` function~~ ✅ DONE (uses `Fdt::new()` from crate)
4. **Implement `find_compatible()` helper** ← REMAINING
5. **Implement `get_reg()` helper** ← REMAINING
6. Add unit tests for new helpers
7. ~~Integrate FDT parsing in `kernel/src/main.rs`~~ ✅ DONE (initrd detection)

### Acceptance Criteria

- [ ] Can parse QEMU-provided DTB
- [ ] Can find GIC node by compatible string
- [ ] Can extract base address from reg property
- [ ] Unit tests pass

### Estimated Complexity

Medium - requires understanding FDT format

---

## Step 2: GICv3 Detection via FDT (CRITICAL)

**File**: `phase-3-step-2.md`

**Note**: GICv3 infrastructure already exists (`init_v3()`, `init_redistributor()`, sysreg module).
Only missing: FDT-based detection to replace broken PIDR2 method.

### Tasks

1. Add `init_from_fdt()` to `gic.rs`
2. Use `fdt::find_compatible()` (from Step 1) to detect version
3. ~~Create `GicConfig` struct~~ → `GicVersion` enum already exists
4. Replace `detect_gic_version()` (PIDR2-based) with FDT-based detection
5. Keep fallback to hardcoded addresses (for QEMU without FDT)
6. Test with GICv3 QEMU profile (`gic-version=3`)

### Acceptance Criteria

- [ ] GICv3 correctly detected on `gic-version=3`
- [ ] GICv2 correctly detected on default virt
- [ ] Fallback works when no FDT
- [ ] Pixel 6 profile works with cluster topology

### Estimated Complexity

Medium - builds on Step 1

---

## Step 3: InterruptHandler Trait (OPTIONAL)

**File**: `phase-3-step-3.md`

**Note**: Current `IrqId` enum + `fn()` pointers in `gic.rs` work adequately.
This step is optional for cleaner architecture but not blocking.

### Tasks

1. ~~Create `levitate-hal/src/irq.rs` module~~ → Add trait to `gic.rs` instead
2. Define `InterruptHandler` trait in `gic.rs`
3. ~~Implement handler registry~~ ✅ EXISTS (`HANDLERS` array)
4. Create `TimerHandler` struct implementing trait
5. Create `UartHandler` struct implementing trait
6. Migrate kernel IRQ registration to new API
7. Update `handle_irq()` in exceptions.rs

### Acceptance Criteria

- [ ] Timer interrupts work with new trait
- [ ] UART interrupts work with new trait
- [ ] No behavioral changes from user perspective
- [ ] Code is cleaner and more type-safe

### Estimated Complexity

Medium - refactoring existing code

---

## Step 4: bitflags! Crate Integration

**File**: `phase-3-step-4.md`

### Tasks

1. ~~Add `bitflags` crate to `levitate-hal/Cargo.toml`~~ ✅ DONE
2. ~~Define `TimerCtrl` flags in `timer.rs`~~ ✅ DONE (`TimerCtrlFlags`)
3. **Define `GicdCtrl` flags in `gic.rs`** ← REMAINING
4. ~~Update timer init to use typed flags~~ ✅ DONE
5. Update GIC init to use typed flags
6. Verify no behavioral changes

### Acceptance Criteria

- [ ] Timer works with bitflags
- [ ] GIC works with bitflags
- [ ] Code is more readable
- [ ] All tests pass

### Estimated Complexity

Low - straightforward refactoring

---

## Step 5: VHE Detection for Timer

**File**: `phase-3-step-5.md`

### Tasks

1. Implement `vhe_present()` in `timer.rs`
2. Add physical timer register access functions
3. Modify timer init to choose based on VHE
4. Test on QEMU (VHE not typically present)
5. Document behavior difference

### Acceptance Criteria

- [ ] VHE detection works correctly
- [ ] Timer uses appropriate mode
- [ ] No regression on existing behavior
- [ ] Works on QEMU (virtual timer mode)

### Estimated Complexity

Low - isolated change

---

## Implementation Guidelines

### Code Comments

All changes must include TEAM_043 markers:

```rust
// TEAM_043: Added FDT parsing for device discovery
```

### Testing Strategy

1. **Unit tests**: Add for each new module
2. **Behavior test**: Verify boot log unchanged (or update golden)
3. **Manual test**: GICv3 + cluster topology on Pixel 6 profile

### Rollback Plan

Each step should be independently revertable. Use feature flags if needed:

```rust
#[cfg(feature = "fdt")]
pub fn init_from_fdt(fdt: &Fdt) -> &'static Gic { ... }
```

---

## Dependencies

| Step | Depends On | Can Parallel With |
|------|------------|-------------------|
| 1 (FDT) | None | - |
| 2 (GIC) | Step 1 | Step 3, 4, 5 |
| 3 (Trait) | None | Step 1, 4, 5 |
| 4 (bitflags) | None | Step 1, 3, 5 |
| 5 (VHE) | None | Step 1, 3, 4 |

---

## Next Phase

After implementation, proceed to **Phase 4 — Integration and Testing** for full system verification.
