# Phase 1 — Discovery

**Feature**: Reference Kernel Improvements  
**Team**: TEAM_043  
**Status**: Ready for execution

---

## Feature Summary

**Short Description**: Adopt best practices from production-quality reference kernels (Redox, Theseus, Tock) to improve LevitateOS's device handling, interrupt management, and hardware abstraction.

**Problem Statement**: LevitateOS currently uses hardcoded device addresses and manual register manipulation. This makes the kernel less portable, harder to maintain, and unable to properly detect hardware variations (e.g., GICv2 vs GICv3).

**Who Benefits**:
- Kernel developers: Cleaner, more maintainable code
- Future hardware targets: Better portability through FDT
- Testing: More reliable device detection

---

## Success Criteria

1. **FDT Parsing**: Kernel can parse Device Tree Blob to discover devices
2. **GICv3 Detection**: Kernel correctly detects GIC version from FDT
3. **InterruptHandler Trait**: IRQ handlers registered via trait instead of raw functions
4. **bitflags! Usage**: Register manipulation uses type-safe bitflags
5. **VHE Detection**: Timer uses optimal mode based on VHE availability

**Acceptance Criteria**:
- [ ] All existing tests continue to pass
- [ ] GICv3 mode works on QEMU with `gic-version=3`
- [ ] Pixel 6 profile can use cluster topology with GICv3
- [ ] No behavioral regressions

---

## Current State Analysis

### How the System Works Today

| Component | Current Approach | Issue |
|-----------|------------------|-------|
| Device Addresses | Hardcoded constants | Not portable |
| GIC Version | Attempted PIDR2 read (fails) | Can't detect v3 |
| IRQ Handlers | Raw function pointers | No type safety |
| Registers | Manual bit manipulation | Error-prone |
| Timer | Uses virtual timer always | May not be optimal |

### Existing Workarounds

- GICv3: Fallback to GICv2 API (works in legacy mode)
- Device mapping: Extended range to include GICR speculatively

---

## Codebase Reconnaissance

### Modules Likely Touched

| Module | Purpose | Files |
|--------|---------|-------|
| GIC | Interrupt controller | `levitate-hal/src/gic.rs` |
| Timer | System timer | `levitate-hal/src/timer.rs` |
| MMU | Memory mapping | `levitate-hal/src/mmu.rs` |
| Kernel main | Device init | `kernel/src/main.rs` |
| Exceptions | IRQ handling | `kernel/src/exceptions.rs` |

### New Modules to Create

| Module | Purpose |
|--------|---------|
| `levitate-hal/src/fdt.rs` | Device Tree parsing |
| `levitate-hal/src/irq.rs` | InterruptHandler trait |

### Tests That May Be Impacted

- `tests/golden_boot.txt` - Boot log format may change
- Unit tests in `levitate-hal` - New tests for FDT parsing
- Behavior tests - GIC detection output

### Reference Code Locations

| Pattern | Redox Location |
|---------|---------------|
| FDT Parsing | `src/dtb/mod.rs` |
| GIC Version | `src/arch/aarch64/device/irqchip/gicv3.rs:38` |
| Timer | `src/arch/aarch64/device/generic_timer.rs` |
| PHYS_OFFSET | `src/arch/aarch64/consts.rs:37` |

---

## Constraints

1. **No External Dependencies for FDT**: Must use existing `fdt` crate or implement minimal parser
2. **Backward Compatibility**: Must still work on QEMU without FDT
3. **Performance**: FDT parsing happens once at boot, not critical path
4. **Memory**: FDT parsing should not require large heap allocation

---

## Phase 1 Steps

### Step 1 — Capture Feature Intent
**Status**: Complete (this document)

### Step 2 — Analyze Current State  
**Status**: Complete (documented above)

### Step 3 — Source Code Reconnaissance
**Status**: Complete (documented above from TEAM_042 study)

---

## Open Questions from Discovery

None critical. The reference kernel patterns are well-understood from the TEAM_042 study.

---

## Next Phase

Proceed to **Phase 2 — Design** to define:
- FDT parsing API
- InterruptHandler trait definition
- GICv3 detection flow
- Integration points
