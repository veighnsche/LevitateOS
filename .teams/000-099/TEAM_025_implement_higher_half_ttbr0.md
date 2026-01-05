# TEAM_025 — Implement Higher-Half Kernel (TTBR0 Approach)

**Created:** 2026-01-03
**Status:** INCOMPLETE — Restored working state
**Role:** Implementation Team
**Plan Source:** TEAM_024 review recommendations

---

## Objective

Implement the higher-half kernel using the TTBR0-only approach as recommended by TEAM_024's reference implementation analysis.

---

## Executive Summary

**Outcome:** Implementation attempt unsuccessful after extensive debugging. Working identity-mapped kernel restored. Detailed learnings documented for future teams.

**Root Issue:** The higher-half kernel implementation is more complex than initially planned. Multiple blocking issues discovered that require architectural changes beyond the original plan scope.

---

## Detailed Findings

### What Worked
1. **Page table setup compiles** — L0[511] → L1_high → L2_high structure was created
2. **MMU enables successfully** — Boot prints ABCD, MMU enable completes without fault
3. **Data reads from high VA work** — Reading memory at high virtual address succeeds
4. **Identity mapping works** — Low addresses (0x40000000+) translate correctly

### What Failed
1. **Code execution from high VA fails** — "Undefined Instruction" exception at correct high address
2. **Root cause unclear** — Data reads work but instruction fetch fails, suggesting:
   - Page table execute permission issue (PXN/UXN), OR
   - Instruction cache coherency problem, OR
   - SCTLR_EL1 configuration missing required bits

### Blocking Issues Discovered

#### Issue 1: QEMU Sparse Binary Loading
- objcopy creates flat binary with gaps for 2MB-aligned kernel
- QEMU `-kernel` doesn't load sparse binaries correctly
- **Workaround attempted:** Removed 2MB alignment, used contiguous loading
- **Result:** Kernel loaded correctly but execution still failed

#### Issue 2: Execute Permission Mystery
- Page table flags set correctly (no PXN/UXN bits)
- I-cache and D-cache enabled in SCTLR_EL1
- TLB and I-cache invalidated before jump
- Data access to same address works, but instruction fetch fails
- ESR shows "Unknown reason" (EC=0), not a clear permission fault

#### Issue 3: Linker Script Complexity
- VMA/LMA calculations for higher-half are complex
- Dynamic offset calculation required when not using fixed alignment
- Symbol addresses must match page table mapping exactly

---

## Code Changes Attempted (All Reverted)

1. **linker.ld:** Split BOOT/KERNEL memory regions, AT() for LMA
2. **main.rs assembly:** Full page table setup in boot code
3. **TCR_EL1:** EPD1=1 to disable TTBR1 walks
4. **SCTLR_EL1:** M=1, C=1, I=1 for MMU and caches
5. **Various cache/TLB invalidation sequences**

---

## Recommendations for Future Teams

### Option A: Theseus-Style Bootloader Approach
Theseus relies on a bootloader (UEFI/Multiboot) to set up initial page tables. Consider:
1. Use a proper bootloader that handles higher-half setup
2. Have the bootloader map both identity and higher-half before kernel entry
3. Kernel enters already running at high address

### Option B: Two-Stage Boot
1. **Stage 1:** Identity-mapped boot code sets up page tables with BOTH mappings
2. **Stage 2:** Trampoline function at known identity address jumps to high
3. **Stage 3:** High code removes identity mapping

### Option C: Debug the Execute Permission Issue
If continuing current approach, focus on:
1. Why data reads work but instruction fetch fails
2. Check if QEMU cortex-a53 has specific requirements
3. Try with `-cpu max` or other CPU models
4. Add QEMU `-d mmu` logging to see translation details

### Minimum Viable Next Steps
1. **Create isolated test:** Small assembly-only kernel that does ONLY the higher-half jump
2. **Use QEMU debug:** `-d int,mmu,guest_errors` for detailed fault info
3. **Compare with working kernel:** Theseus or Redox QEMU configs

---

## Progress Log

| Date | Action |
|------|--------|
| 2026-01-03 | Team created. Starting implementation. |
| 2026-01-03 | Restored working state, implemented page table setup |
| 2026-01-03 | MMU enables, data reads work, but code execution fails |
| 2026-01-03 | Multiple debug iterations, issue not resolved |
| 2026-01-03 | Restored working identity-mapped kernel |
| 2026-01-03 | Documented learnings for future teams |

---

## Handoff Checklist

- [x] Project builds cleanly
- [x] All tests pass
- [x] Behavioral regression tests pass
- [x] Team file updated
- [x] Remaining TODOs documented

---

## Blockers for Next Team

1. **Higher-half kernel NOT implemented** — requires dedicated investigation
2. **Execute permission issue** — needs QEMU debug logging to diagnose
3. **Plan needs expansion** — current UoW too large, split into smaller steps
