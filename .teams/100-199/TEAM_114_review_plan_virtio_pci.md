# TEAM_114: Review Plan - VirtIO PCI Migration

**Created:** 2026-01-05
**Task:** Review virtio-pci plan following /review-a-plan workflow
**Status:** Complete

---

## Summary

Reviewed the VirtIO PCI Migration plan at `docs/planning/virtio-pci/`.
**Verdict:** Plan needs significant revision before implementation.

---

## Architecture Clarification

PCI and GPU are **separate layers** that work together:

```
┌─────────────────────────────────────┐
│  GPU Driver (protocol/commands)     │  ← VirtIOGpu<H, T>
├─────────────────────────────────────┤
│  Transport (how to talk to device)  │  ← PciTransport OR MmioTransport  
├─────────────────────────────────────┤
│  Hardware (ECAM/MMIO registers)     │  ← kernel/src/pci.rs
└─────────────────────────────────────┘
```

The plan correctly identifies the need for PCI transport. The GPU driver sits on top.

---

## Critical Findings

### 1. PCI BAR Allocation Missing (Severity: Critical)

The plan only covers ECAM config space access but **omits BAR allocation entirely**.

The virtio-drivers example shows mandatory steps:
- Parse PCI ranges from DTB
- Allocate 32-bit memory addresses for BARs
- Call `root.set_bar_32()` or `root.set_bar_64()`
- Enable `Command::MEMORY_SPACE | Command::BUS_MASTER`

**Without BAR allocation, PciTransport will not work.**

### 2. Wrong API Design (Severity: Critical)

Phase 2 proposes:
```rust
pub fn find_device(vendor: u16, device: u16) -> Option<PciTransport>;
```

Actual virtio-drivers API requires:
```rust
let mut pci_root = PciRoot::new(MmioCam::new(ecam_base, Cam::Ecam));
for (device_function, info) in pci_root.enumerate_bus(0) {
    if virtio_device_type(&info) == Some(DeviceType::GPU) {
        allocate_bars(&mut pci_root, device_function, &mut allocator);
        let transport = PciTransport::new::<HalImpl, _>(&mut pci_root, device_function)?;
    }
}
```

### 3. Wrong Trait Name (Severity: Medium)

Phase 3 says "Implement `PciConfiguration` trait" - the actual trait is `ConfigurationAccess`.

### 4. GPU Driver Choice (Severity: Medium - Architectural Decision Needed)

TEAM_107 documented that `levitate-drivers-gpu` has timeout issues (unrelated to transport).

**Two valid paths forward:**

| Option | GPU Driver | Transport | Risk |
|--------|-----------|-----------|------|
| **A** | Fix `levitate-drivers-gpu`, make generic over `T: Transport` | `PciTransport` | Higher - must fix timeout bug first |
| **B** | Use `virtio_drivers::device::gpu::VirtIOGpu<H, T>` | `PciTransport` | Lower - already tested |

Both require the PCI layer from this plan. See `.questions/TEAM_114_gpu_driver_choice.md`.

---

## Oversimplification Issues

| Issue | Impact |
|-------|--------|
| No BAR memory allocator | PCI devices won't have mapped BARs |
| No DTB parsing for PCI ranges | Unknown memory regions |
| No cleanup phase for dead MMIO code | Violates Rule 6 |
| No handoff checklist | Violates Rule 10 |

---

## Correct Implementation Pattern

Based on virtio-drivers aarch64 example:

```rust
// 1. Map ECAM region as Device memory
// 2. Parse PCI node from DTB for ranges
// 3. Create allocator for 32-bit memory
// 4. Create PciRoot with MmioCam
// 5. Enumerate bus, find VirtIO GPU
// 6. Allocate BARs
// 7. Create PciTransport
// 8. Create VirtIOGpu<HalImpl, PciTransport>
```

---

## Recommendations

### Must Fix Before Implementation

1. **Rewrite Phase 2 API Design** to match virtio-drivers actual API
2. **Add BAR allocation step** in Phase 3
3. **Add DTB parsing** for PCI ranges (not just ECAM base)
4. **Use virtio-drivers VirtIOGpu** instead of levitate-drivers-gpu

### Should Add

5. **Add Phase 5: Cleanup** - Remove dead MMIO scanning code
6. **Add handoff checklist** to Phase 4
7. **Reference TEAM_107** issues in Phase 1 constraints

---

## Rule Compliance Summary

| Rule | Status |
|------|--------|
| Rule 0 (Quality) | ⚠️ Fix: Use tested virtio-drivers code |
| Rule 4 (Regression) | ⚠️ Add: golden_boot.txt update |
| Rule 6 (No Dead Code) | ❌ Add: Cleanup phase |
| Rule 10 (Handoff) | ❌ Add: Checklist |

---

## Handoff Notes

**Next Steps:**
1. TEAM_113 (or successor) should revise plan based on this review
2. Key reference: `virtio-drivers/examples/aarch64/src/main.rs`
3. Enable `pci` feature: `virtio-drivers = { version = "0.12", features = ["pci"] }`

**Blocking Questions:**
- Should we keep `levitate-drivers-gpu` or switch entirely to `virtio-drivers::device::gpu`?

**Files Reviewed:**
- `docs/planning/virtio-pci/phase-1.md` through `phase-4.md`
- `.questions/TEAM_107_gpu_driver_issues.md`
- `levitate-drivers-gpu/src/device.rs`
- `kernel/Cargo.toml`
- `levitate-hal/src/mmu.rs`

---

## Handoff Checklist

- [x] Review completed
- [x] Findings documented
- [x] Recommendations provided
- [x] Team file updated
- [x] User decision: Option B (use virtio-drivers VirtIOGpu)

## Post-Decision Actions (2026-01-05)

**Archived custom GPU driver:**
- Moved `levitate-drivers-gpu/` → `.archive/levitate-drivers-gpu/`
- Added `ARCHIVED.md` explaining why

**Updated Cargo files:**
- Removed from workspace members in `Cargo.toml`
- Removed dependency from `kernel/Cargo.toml`

**Stubbed GPU code:**
- `kernel/src/gpu.rs` - placeholder until PCI implementation
- `kernel/src/virtio.rs` - updated comments

**Build status:** ✅ Kernel compiles successfully

## Implementation Complete (TEAM_114)

All steps completed successfully:

1. ✅ Revised plan based on review findings
2. ✅ Added ECAM constants to `levitate-hal/src/mmu.rs`
3. ✅ Created `levitate-pci/` crate with BAR allocation
4. ✅ Created `levitate-gpu/` crate wrapping virtio-drivers
5. ✅ Updated QEMU flags to use `virtio-gpu-pci`
6. ✅ Updated golden file
7. ✅ Behavior tests passing

## Visual Verification Result

**GPU IS WORKING** - Purple framebuffer and text are visible on screen.

**BUT** - The terminal rendering is broken:
- Text appears as black rectangles on purple background
- Each line has its own black box
- This is NOT a proper terminal UI

**Conclusion:** PCI GPU migration succeeded. The issue is now in the terminal layer, not GPU.

See: `docs/GOTCHAS.md` section 17 for details.
