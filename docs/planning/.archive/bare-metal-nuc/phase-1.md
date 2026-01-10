# Phase 1: Discovery - Bare Metal NUC Support

## Feature Summary
- **Description**: Enable LevitateOS to boot and run on bare metal Intel NUC7i3BNH hardware.
- **Problem Statement**: Current hardware support is heavily biased towards QEMU and VirtIO. Bare metal execution requires real hardware drivers and a more robust HAL that handles non-identity mapped MMIO and complex device topologies.
- **Benefits**: Demonstrates the portability and "real-world" viability of LevitateOS on actual silicon.

## Success Criteria
- [ ] Kernel boots to shell on Intel NUC7i3BNH.
- [ ] Display output via UEFI GOP (Graphics Output Protocol).
- [ ] Input via USB (XHCI and HID).
- [ ] Storage access via NVMe (M.2) or SATA.
- [ ] All tests (unit and integration) continue to pass in QEMU.

## Current State Analysis
- **AArch64**: Stable in QEMU and Pixel 6 (GICv3).
- **x86_64**: Runs in QEMU (q35/i440fx) using VirtIO drivers. Base HAL (GDT, IDT, Paging) exists but is "forgiving" regarding MMIO mapping.
- **Drivers**: Exclusively VirtIO-based (`virtio-gpu`, `virtio-blk`, `virtio-input`).
- **Bootloader**: Uses Limine, which supports both BIOS and UEFI, facilitating easy handoff of framebuffer info.

## Codebase Reconnaissance
- **Affected Areas**:
    - `crates/hal/src/x86_64`: MMIO mapping, APIC/IOAPIC stability, ACPI integration.
    - `crates/drivers`: New crates needed for `nvme`, `xhci`, `intel-gop` (or `generic-fb`).
    - `crates/pci`: Needs to be more generic for hardware ID matching.
- **Public APIs**: Driver traits in `crates/traits` (`storage-device`, `gpu`, `input-device`) should be reused to ensure consistency.
- **Constraints**: 
    - Must not break QEMU/VirtIO support.
    - Must handle MMIO safely (Rule 5: Breaking Changes > Fragile Compatibility).

## Steps and UoWs

### Step 1 – Capture Feature Intent
- [x] Write problem statement and benefits.
- [x] List success criteria.

### Step 2 – Analyze Current State
- [x] Document current x86_64 and VirtIO situation.

### Step 3 – Source Code Reconnaissance
- [x] Identify affected modules and APIs.
- [ ] Audit `crates/hal/src/x86_64/mmu.rs` for MMIO mapping gaps.
- [ ] Audit `crates/pci/src/lib.rs` for hardware-specific probe logic.
