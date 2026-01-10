# Phase 3: Implementation - x86_64 Behavior Test Completion

## Prerequisites

Before starting implementation:
- [ ] Phase 2 questions answered
- [ ] All tests currently pass
- [ ] Design decisions documented

## Implementation Steps

### Step 1: Debug Limine Request Detection

**Goal**: Understand why `BASE_REVISION.is_supported()` returns false.

**File**: `kernel/src/boot/limine.rs`

**Tasks**:
1. Add diagnostic logging for request static addresses
2. Check if requests are at expected physical locations
3. Verify Limine is scanning the correct section

**UoW Size**: Small (1 session)

---

### Step 2: Fix Limine Request Section

**Goal**: Ensure Limine finds and fills request responses.

**Files**: 
- `kernel/src/arch/x86_64/linker.ld`
- `Cargo.toml` (limine crate version)

**Tasks**:
1. Verify `.requests` section alignment (8-byte minimum)
2. Check `limine` crate version compatibility
3. Ensure section is within Limine's scan range

**UoW Size**: Small (1 session)

---

### Step 3: Implement HHDM-Based MMIO Access

**Goal**: Access PCI ECAM and APIC via Limine's HHDM.

**Files**:
- `kernel/src/boot/limine.rs` - export `hhdm_offset()`
- `crates/pci/src/lib.rs` - use HHDM for ECAM
- `crates/hal/src/x86_64/mod.rs` - use HHDM for APIC

**Tasks**:
1. Make `hhdm_offset()` robust (panic with message if unavailable)
2. Modify PCI ECAM access to use HHDM on Limine boot
3. Modify APIC access to use HHDM on Limine boot
4. Re-enable APIC init for Limine (currently skipped)

**UoW Size**: Medium (1-2 sessions)

---

### Step 4: Create x86_64 Golden File Infrastructure

**Goal**: Support arch-specific golden files for behavior tests.

**Files**:
- `xtask/src/tests/behavior.rs`
- `tests/golden_boot_x86_64.txt` (new)

**Tasks**:
1. Modify `behavior.rs` to select golden file by arch
2. Run x86_64 behavior test to capture output
3. Create initial golden file from captured output
4. Mask any variable content (addresses, timestamps)

**UoW Size**: Small (1 session)

---

### Step 5: Verify Full Boot Path

**Goal**: Kernel boots fully on x86_64 via Limine.

**Tasks**:
1. Run behavior test, verify no page faults
2. Confirm all boot stages complete
3. Verify GPU/terminal init works (or gracefully skipped)
4. Confirm shell prompt reached

**UoW Size**: Small (1 session)

---

## Implementation Order

```
Step 1 ─→ Step 2 ─→ Step 3 ─→ Step 4 ─→ Step 5
  │          │          │          │
  └── Debug ─┴── Fix ───┴── Use ───┴── Verify
```

Steps 1-3 are sequential (each depends on previous).
Step 4 can begin once Step 3 is partially working.
Step 5 is final verification.

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Limine crate incompatibility | Pin to known working version, document |
| HHDM offset incorrect | Add sanity checks, test with known physical addresses |
| Golden file too strict | Use masking for variable content |
| Changes break aarch64 | Run both arch tests after each change |

## Rollback Plan

If implementation fails:
1. Revert to TEAM_286 state (working Limine boot to Stage 3)
2. Document what was learned
3. Create question file for architectural guidance
