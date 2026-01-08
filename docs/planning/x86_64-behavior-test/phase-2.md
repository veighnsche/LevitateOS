# Phase 2: Design - x86_64 Behavior Test Completion

## Proposed Solution

### Overview

Fix x86_64 behavior test by:
1. **Fixing Limine request detection** - Enable HHDM access
2. **Using HHDM for MMIO** - Access PCI ECAM and APIC via HHDM offsets
3. **Creating x86_64 golden file** - Separate baseline for x86_64 boot output

### Solution Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Limine Boot Path                          │
├─────────────────────────────────────────────────────────────┤
│ 1. Limine loads kernel, sets up page tables with HHDM       │
│ 2. Kernel detects Limine via magic=0                         │
│ 3. HHDM offset obtained from HHDM_REQUEST                    │
│ 4. PCI/APIC access uses: HHDM_OFFSET + PHYS_ADDR            │
│ 5. Kernel boots fully, behavior test captures output         │
│ 6. Output compared against x86_64 golden file                │
└─────────────────────────────────────────────────────────────┘
```

## Design Decisions

### Decision 1: HHDM-Based MMIO Access

**Current State**: PCI uses hardcoded VA 0xFFFFFFFF40000000 for ECAM.

**Proposed Change**: For Limine boot, use HHDM offset + physical address.

```rust
// Current (broken on Limine)
const ECAM_VIRT: u64 = 0xFFFFFFFF40000000;

// Proposed (works with Limine HHDM)
fn ecam_address(hhdm_offset: u64) -> u64 {
    const ECAM_PHYS: u64 = 0xB0000000; // Standard x86_64 ECAM
    hhdm_offset + ECAM_PHYS
}
```

### Decision 2: Fix Request Detection

**Root Cause Hypothesis**: Limine crate version mismatch or section alignment issue.

**Investigation Path**:
1. Check `limine` crate version in Cargo.toml
2. Verify `.requests` section alignment (must be 8-byte aligned)
3. Add debug output for request static addresses

### Decision 3: Arch-Specific Golden Files

**Current State**: Single `tests/golden_boot.txt` for aarch64.

**Proposed Change**: 
- `tests/golden_boot.txt` → remains for aarch64
- `tests/golden_boot_x86_64.txt` → new file for x86_64
- `behavior.rs` selects based on target arch

## Behavioral Contracts

### HHDM Access

| Scenario | Expected Behavior |
|----------|-------------------|
| HHDM available | Use `hhdm_offset()` for all physical memory access |
| HHDM unavailable | Fall back to identity-mapped regions or panic |
| Invalid HHDM offset | Panic with descriptive error |

### PCI ECAM Access

| Scenario | Expected Behavior |
|----------|-------------------|
| Limine boot | Use `hhdm_offset + 0xB0000000` |
| Multiboot boot | Use mapped VA at 0xFFFFFFFF40000000 |
| ECAM not accessible | Panic with "PCI ECAM not mapped" |

### Golden File Selection

| Target Arch | Golden File |
|-------------|-------------|
| aarch64 | `tests/golden_boot.txt` |
| x86_64 | `tests/golden_boot_x86_64.txt` |

## Open Questions (Require User Input)

### Q1: Limine Version Requirement
**Question**: Should we require a specific minimum Limine version, or support multiple versions?

**Options**:
- A) Require Limine 7.x+ (current in ISO build)
- B) Support fallback for older Limine versions
- C) Document version requirement, fail fast if incompatible

**Recommendation**: Option C - Document and fail fast.

### Q2: ECAM Base Address Source
**Question**: How should we determine the PCI ECAM base address on x86_64?

**Options**:
- A) Hardcode 0xB0000000 (standard, works for QEMU)
- B) Parse ACPI MCFG table for dynamic discovery
- C) Add boot parameter to configure

**Recommendation**: Option A for now, with TODO for ACPI MCFG in future.

### Q3: Golden File Content Scope
**Question**: How much of the boot output should the x86_64 golden file cover?

**Options**:
- A) Full boot through shell prompt (like aarch64)
- B) Only until "System Ready" message
- C) Minimal - just confirm kernel boots

**Recommendation**: Option A - Full parity with aarch64 test.

### Q4: Limine Request Section Fix
**Question**: If request detection fails due to section placement, how should we fix it?

**Options**:
- A) Adjust linker script to ensure proper alignment
- B) Use different section name that Limine expects
- C) Pin `limine` crate to known working version

**Recommendation**: Investigate root cause first, then decide.

### Q5: APIC Access Strategy
**Question**: Should APIC/IOAPIC be accessed via HHDM or custom mapping?

**Options**:
- A) Always use HHDM for APIC (0xFEE00000)
- B) Set up dedicated APIC mapping after boot
- C) Skip APIC for now, use PIT timer

**Recommendation**: Option A - HHDM is simpler and works.

## Phase 2 Steps

### Step 1: Answer Open Questions
- Present questions to user
- Document decisions

### Step 2: Finalize HHDM Integration Design
- Define `hhdm_offset()` API contract
- Plan changes to PCI crate for HHDM support

### Step 3: Design Golden File Infrastructure
- Define arch-specific selection logic
- Plan test harness changes

### Step 4: Review Design Against Architecture
- Ensure changes are minimal and focused
- Verify no impact on aarch64 boot path
