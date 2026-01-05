# TEAM_078: Implement Device MMIO via TTBR1

## Status
**COMPLETE** - Device MMIO now mapped via TTBR1

## Mission
Implement the device MMIO remapping fix from `docs/planning/bugfix-device-mmio-ttbr1/phase-4.md`.

## Context
- **Bug:** Device MMIO uses identity mapping via TTBR0, causing hang on userspace switch
- **Root Cause:** Confirmed by TEAM_076
- **Plan:** Created by TEAM_077
- **This Team:** Implements the fix

## Work Log

### 2026-01-04
- [x] Step 1: Define device VA constants in mmu.rs
- [x] Step 2: Update phys_to_virt for devices  
- [x] Step 3: Map device regions in TTBR1 during boot
- [x] Step 4: Update console driver
- [x] Step 5: Update VirtIO drivers
- [x] Step 6: Update GIC driver
- [x] Step 7: Add device mapping to assembly boot (critical fix)
- [x] Step 8: Test all device paths
- [x] Remove TEAM_076 breadcrumb

## Summary
Mapped device regions (UART, GIC, VirtIO) via TTBR1 at high VA so they remain
accessible when TTBR0 is switched for userspace execution.

## Files Modified
- `levitate-hal/src/mmu.rs` - Added device VA constants, updated phys_to_virt
- `levitate-hal/src/console.rs` - Use mmu::UART_VA
- `levitate-hal/src/gic.rs` - Use mmu::GIC_*_VA constants
- `kernel/src/virtio.rs` - Use mmu::VIRTIO_MMIO_VA
- `kernel/src/main.rs` - Assembly boot: map devices to high VA, kmain: map devices via TTBR1
- `kernel/src/task/process.rs` - Removed TEAM_076 breadcrumb

## Verification
- [x] Kernel boots and prints to console
- [x] All devices work (GIC, VirtIO block/net/input, GPU)
- [x] Console prints work after switch_ttbr0()
- [x] Userspace demo starts (exception is separate bug in user code)

## References
- `@/home/vince/Projects/LevitateOS/docs/planning/bugfix-device-mmio-ttbr1/phase-4.md`
- `@/home/vince/Projects/LevitateOS/.teams/TEAM_076_investigate_userspace_hang.md`
