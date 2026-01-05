# TEAM_043: Reference Kernel Improvements Feature

**Created**: 2026-01-04
**Status**: Ready for Implementation
**Feature**: Adopt best practices from reference kernels (Redox, Theseus, Tock)
**Goal**: Emulate Pixel 6 hardware as tightly as possible

## Team Mission

Implement improvements identified from studying reference kernels to bring LevitateOS closer to production-quality patterns, with focus on accurate Pixel 6 (Tensor GS101) emulation.

## Identified Improvements (Priority Order)

1. **FDT Parsing** - CRITICAL: Device Tree parsing for hardware discovery
2. **GICv3 Support** - CRITICAL: Pixel 6 Tensor uses GICv3
3. **InterruptHandler Trait** - NICE: Cleaner IRQ handler registration
4. **bitflags! Crate** - NICE: Cleaner register manipulation
5. **VHE Detection** - NICE: Timer optimization for virtualization

## Key Decisions (Answered 2026-01-04)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| FDT crate | `fdt = "0.1"` (repnop) | Standard no_std parser |
| GIC fallback | Hardcoded addresses | Backward compatible |
| Handler timing | After GIC init | ARM best practice |
| bitflags | no_std mode | Kernel-safe |
| FDT source | x0 + memory scan | Flexible |

## Planning Location

All phase files: `docs/planning/reference-kernel-improvements/`

## Related Work

- TEAM_042: Initial GICv3 implementation attempt, MMU mapping fix
- Reference: `.external-kernels/redox-kernel/`

## Progress Log

### 2026-01-04
- Created feature plan based on reference kernel study
- Identified 5 major improvements from Redox kernel patterns
- Researched best practices online for FDT, GIC, ARM timer
- Answered all 6 design questions with Pixel 6 focus
- Phase 2 complete, Phase 3 ready for implementation
