# TEAM_332: VirtIO Driver Reorganization Refactor

**Date:** 2026-01-09  
**Status:** ✅ PLAN COMPLETE — Reviewed and strengthened by TEAM_333  
**Type:** Refactor

## Objective

Reorganize scattered VirtIO driver code into a clean, modular crate structure with proper abstractions.

## Pain Points

1. **Scattered code** - Drivers split between `kernel/src/` and `crates/`
2. **No transport abstraction** - MMIO vs PCI handling duplicated in each driver
3. **No driver trait** - No common interface for VirtIO drivers
4. **Dead code** - `crates/virtio/` is unused reference code
5. **Inconsistent patterns** - Each driver handles discovery differently

## Success Criteria

- All VirtIO drivers in dedicated crates under `crates/drivers/`
- Unified transport abstraction (MMIO/PCI)
- Device trait crates (`StorageDevice`, `InputDevice`, `NetworkDevice`) — Theseus pattern
- Common driver trait for initialization and device access
- Dead code removed
- All tests pass

## Design Decisions (Answered by TEAM_333)

1. **Transport approach:** WRAP `virtio-drivers` transports (not replace)
2. **Testing support:** YES, driver crates support `std` via feature flag
3. **PCI for block/net:** DEFER to avoid scope creep

## Planning Location

`docs/planning/virtio-driver-refactor/`

## Related Teams

- TEAM_333: Reviewed plan, added Theseus patterns
- TEAM_331: Added PCI input support (exposed the scattered organization)
- TEAM_114: Original VirtIO GPU refactor

