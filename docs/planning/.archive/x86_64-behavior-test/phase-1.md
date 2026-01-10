# Phase 1: Discovery - x86_64 Behavior Test Completion

## Feature Summary

**Problem**: The x86_64 behavior test currently fails due to:
1. Limine HHDM not accessible (requests not detected)
2. PCI ECAM page fault (address not mapped)
3. No x86_64-specific golden file

**Prior Work**: TEAM_286 fixed serial silence, kernel now boots to Stage 3 with 24 lines of output before crashing on PCI ECAM access.

**Goal**: Complete x86_64 behavior test with passing golden file comparison.

## Success Criteria

1. `cargo xtask test --behavior` passes on x86_64
2. x86_64 has its own golden file (`tests/golden_boot_x86_64.txt`)
3. Kernel boots fully without ECAM/APIC page faults
4. Limine HHDM is usable for physical memory access

## Current State Analysis

### What Works
- Limine ISO boot via `-cdrom levitate.iso -boot d`
- Serial output through Limine
- Memory map parsing (16 regions, 476 MB usable)
- Boot Stages 1-3 complete

### What Fails
- `BASE_REVISION.is_supported()` returns false
- `HHDM_REQUEST.get_response()` returns None
- PCI ECAM access at VA 0xFFFFFFFF40000000 causes page fault
- APIC/IOAPIC init skipped due to similar mapping issues

### Workarounds in Place
- Limine detected via `multiboot_magic == 0` instead of BASE_REVISION
- CR3 switch skipped for Limine boot
- APIC init skipped for Limine boot

## Codebase Reconnaissance

### Files Likely to be Modified

| File | Purpose |
|------|---------|
| `kernel/src/boot/limine.rs` | Limine request handling |
| `kernel/src/arch/x86_64/linker.ld` | Section placement for requests |
| `crates/pci/src/lib.rs` | PCI ECAM access |
| `crates/hal/src/x86_64/mod.rs` | x86_64 HAL initialization |
| `xtask/src/tests/behavior.rs` | Behavior test runner |
| `tests/golden_boot_x86_64.txt` | New golden file |

### Key Constraints

1. **No CR3 switch on Limine** - Kernel uses Limine's page tables
2. **HHDM required** - Physical memory access needs HHDM offset
3. **PCI ECAM at 0xB0000000** - Standard x86_64 ECAM base (physical)
4. **APIC at 0xFEE00000** - Standard local APIC physical address

## Phase 1 Steps

### Step 1: Verify Limine Version Compatibility (UoW-ready)
- Check which `limine` crate version is in use
- Compare with Limine bootloader version in ISO build
- Verify request struct format matches

### Step 2: Debug Request Detection (UoW-ready)
- Add diagnostic logging in `limine.rs` 
- Print addresses of request statics
- Verify `.requests` section is at expected physical address

### Step 3: Trace Page Table State (UoW-ready)
- Log CR3 value on Limine entry
- Dump page table entries for ECAM region
- Confirm Limine's HHDM mapping range

## Discovery Questions

These should be answered during Phase 1:

1. What version of `limine` crate is being used?
2. What version of Limine bootloader is in the ISO?
3. Are the request statics actually in the `.requests` section?
4. What is the HHDM offset Limine intends to provide?
5. Does Limine's page table include the ECAM region?
