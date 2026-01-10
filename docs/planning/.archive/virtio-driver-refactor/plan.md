# VirtIO Driver Reorganization Plan

**TEAM_332** | Created: 2026-01-09  
**TEAM_333** | Reviewed: 2026-01-09 — Strengthened with Theseus kernel patterns

## Overview

Reorganize scattered VirtIO driver code into a clean, modular crate structure with proper abstractions.

**Reference:** Patterns inspired by [Theseus OS](https://github.com/theseus-os/Theseus) device architecture (see `.external-kernels/theseus/`).

## Phases

| Phase | Name | Status | Description |
|-------|------|--------|-------------|
| 1 | [Discovery and Safeguards](phase-1.md) | 📋 TODO | Map behavior, lock in tests |
| 2 | [Structural Extraction](phase-2.md) | 📋 TODO | Create new crates |
| 3 | [Migration](phase-3.md) | 📋 TODO | Move call sites |
| 4 | [Cleanup](phase-4.md) | 📋 TODO | Remove dead code |
| 5 | [Hardening and Handoff](phase-5.md) | 📋 TODO | Final verification |

## Estimated Effort

| Phase | Steps | Est. UoWs | Complexity |
|-------|-------|-----------|------------|
| 1 | 3 | 3-5 | Low |
| 2 | 6 | 10-14 | High |
| 3 | 5 | 5-8 | Medium |
| 4 | 4 | 4-6 | Low |
| 5 | 4 | 4-5 | Low |
| **Total** | **22** | **26-38** | |

## Key Deliverables

### Core Crates

1. **`crates/virtio-transport/`** - Unified MMIO/PCI transport abstraction
2. **`crates/drivers/virtio-input/`** - Input driver crate
3. **`crates/drivers/virtio-blk/`** - Block driver crate
4. **`crates/drivers/virtio-net/`** - Network driver crate
5. **`crates/drivers/virtio-gpu/`** - GPU driver crate (moved from `crates/gpu/`)
6. **`kernel/src/drivers/`** - Thin kernel integration layer

### New: Device Trait Crates (Theseus Pattern)

7. **`crates/traits/storage-device/`** - `StorageDevice` trait for block devices
8. **`crates/traits/input-device/`** - `InputDevice` trait for keyboards/mice
9. **`crates/traits/network-device/`** - `NetworkDevice` trait for NICs

> **Why trait crates?** Following Theseus's `storage_device` pattern, separating traits from implementations enables non-VirtIO drivers (PS2 keyboard, AHCI, etc.) to implement the same interfaces. See `.external-kernels/theseus/kernel/storage_device/src/lib.rs`.

## Design Decisions (Answered Questions)

### Q1: Should `virtio-transport` wrap or replace `virtio-drivers` transports?

**Answer: WRAP**

```rust
// crates/virtio-transport/src/lib.rs
pub enum Transport {
    Mmio(virtio_drivers::transport::mmio::MmioTransport<'static>),
    Pci(virtio_drivers::transport::pci::PciTransport),
}
```

**Rationale:** Wrapping is simpler — avoids rewriting the complex `Transport` trait implementations that `virtio-drivers` already provides. Our `Transport` enum delegates to the inner transport while providing a unified API.

### Q2: Should driver crates support `std` for unit testing?

**Answer: YES, via feature flag**

```toml
# crates/drivers/virtio-blk/Cargo.toml
[features]
default = []
std = []  # Enables host-side unit testing
```

**Rationale:** Following Theseus's approach, driver crates should be testable on the host. The `std` feature enables mock transports and property-based testing without affecting `no_std` kernel builds.

### Q3: Add PCI support to block/net drivers in this refactor or defer?

**Answer: DEFER**

**Rationale:** Scope creep risk. The primary goal is reorganization. PCI support for block/net can be a follow-up task once the structure is clean.

## Architecture Pattern: Device Manager

**Inspired by:** `.external-kernels/theseus/kernel/device_manager/src/lib.rs`

Instead of scattering device discovery in `virtio.rs` + individual `init_*()` functions, adopt a centralized pattern:

```rust
// kernel/src/drivers/mod.rs (Phase 2 Step 6 - NEW)

/// Central device manager coordinating all driver initialization
pub fn init() {
    // 1. Early devices (GPU for terminal)
    gpu::init();
    
    // 2. Enumerate PCI bus and dispatch to drivers
    for dev in los_pci::pci_device_iter() {
        match (dev.vendor_id, dev.device_id, dev.class) {
            // VirtIO devices
            (0x1AF4, 0x1000..=0x107F, _) => match dev.device_id {
                0x1001 => virtio_blk::init_pci(dev),
                0x1000 => virtio_net::init_pci(dev),
                0x1052 => virtio_input::init_pci(dev),
                _ => {}
            },
            // Future: AHCI, NVMe, etc.
            _ => {}
        }
    }
    
    // 3. MMIO devices (aarch64)
    #[cfg(target_arch = "aarch64")]
    mmio::scan_and_init();
}
```

## Architecture Pattern: Device Registry

**Inspired by:** Theseus's `StorageDeviceRef = Arc<Mutex<dyn StorageDevice + Send>>`

Instead of module-level statics, drivers register with a central registry:

```rust
// crates/traits/storage-device/src/lib.rs
pub type StorageDeviceRef = Arc<Mutex<dyn StorageDevice + Send>>;

// kernel/src/drivers/registry.rs (Phase 2 Step 6 - NEW)
static STORAGE_DEVICES: Mutex<Vec<StorageDeviceRef>> = Mutex::new(Vec::new());
static INPUT_DEVICES: Mutex<Vec<InputDeviceRef>> = Mutex::new(Vec::new());

pub fn register_storage_device(dev: StorageDeviceRef) {
    STORAGE_DEVICES.lock().push(dev);
}

pub fn storage_devices() -> impl Iterator<Item = StorageDeviceRef> {
    STORAGE_DEVICES.lock().clone().into_iter()
}
```

> **Note:** This pattern is optional for Phase 1-3. Current module statics work. Consider for Phase 5 hardening or future work.

## Dependencies

- Phase 2 depends on Phase 1 completion
- Phase 3 depends on Phase 2 (each step can start as corresponding Phase 2 step completes)
- Phase 4 depends on Phase 3 completion
- Phase 5 depends on Phase 4 completion

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Break x86_64 display | Medium | High | Screenshot tests catch early |
| Break aarch64 boot | Low | High | Golden log tests catch |
| Keyboard input regression | Medium | High | Manual testing on both arches |
| Long migration time | Medium | Medium | Incremental migration per driver |
| Trait crate over-abstraction | Low | Medium | Keep traits minimal, add methods as needed |

## Success Criteria

- [ ] All VirtIO drivers in dedicated crates
- [ ] Unified transport abstraction works for MMIO and PCI
- [ ] Device trait crates created (`StorageDevice`, `InputDevice`, `NetworkDevice`)
- [ ] All tests pass (screenshot, behavior, unit)
- [ ] Dead code removed (`crates/virtio/`, old kernel drivers)
- [ ] Architecture documentation updated
- [ ] No behavior regressions

## Future Work (Out of Scope)

Items intentionally deferred from this refactor:

1. **PCI support for block/net** — Add after structure is clean
2. **Device registry pattern** — Optional, consider for Phase 5
3. **Downcast support** — Add `downcast_rs` for debugging when needed
4. **Async driver support** — Current drivers are sync/polling
5. **DMA abstraction** — Using `virtio-drivers` DMA directly
6. **Hot-plug support** — Devices discovered at boot only
7. **Interrupt affinity API** — `fn preferred_cpu(&self) -> Option<CpuId>`

---

## Quick Start

To begin implementation:

1. Read `phase-1.md` thoroughly
2. Run baseline tests: `cargo xtask test levitate && cargo xtask test behavior`
3. Start with Phase 1 Step 1 tasks

## Reference Materials

- **Theseus device_manager:** `.external-kernels/theseus/kernel/device_manager/src/lib.rs`
- **Theseus storage_device trait:** `.external-kernels/theseus/kernel/storage_device/src/lib.rs`
- **Theseus interrupt_controller:** `.external-kernels/theseus/kernel/interrupt_controller/src/lib.rs`
