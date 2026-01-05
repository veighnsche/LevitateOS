# TEAM_098: Implement VirtIO GPU Refactor

**Date:** 2026-01-05  
**Role:** Plan Implementer  
**Plan:** `docs/planning/virtio-gpu-scanout/`

---

## Objective

Implement the VirtIO GPU refactor plan to create proper crate-level separation:
- `levitate-virtio` — General VirtIO transport (queue, MMIO)
- `levitate-virtio-gpu` — GPU protocol structs + async driver

**Key Requirements (from User Answers):**
- Q1: Complete replacement of `virtio-drivers` ✅
- Q2: `levitate-virtio` + `levitate-virtio-gpu` naming ✅
- Q3: Define new HAL traits ✅
- Q4: **Async-first design** ("DO IT RIGHT FROM THE START!!!")

---

## Current Architecture (BROKEN)

```
kernel/src/gpu.rs         → Thin re-export
levitate-gpu/src/gpu.rs   → Wraps virtio-drivers (black box)
virtio-drivers            → External crate, no debugging access
```

## Target Architecture

```
levitate-virtio/          # General VirtIO transport
├── src/
│   ├── lib.rs
│   ├── queue.rs          # VirtQueue implementation
│   └── transport.rs      # Transport trait

levitate-virtio-gpu/      # GPU-specific protocol + driver
├── src/
│   ├── lib.rs
│   ├── protocol/
│   │   ├── mod.rs
│   │   ├── ctrl_header.rs
│   │   ├── commands.rs
│   │   └── formats.rs
│   ├── driver.rs         # Async state machine
│   └── display.rs        # DrawTarget impl

levitate-gpu/             # High-level API (minimal changes)
```

---

## Implementation Progress

### Phase 1: Discovery ✅ COMPLETE
- TEAM_097 confirmed driver is calling correct VirtIO commands
- Resolution is correctly 1280x800
- Issue is visibility/debugging in virtio-drivers black box

### Phase 2: Protocol Infrastructure ✅ COMPLETE
- [x] Step 1: Create `levitate-virtio` crate
  - VirtQueue implementation
  - Transport trait + MMIO implementation
  - Status and feature bit constants
- [x] Step 2: Create `levitate-virtio-gpu` crate with protocol structs
  - CtrlHeader, CtrlType, Rect, ResourceId
  - All 2D command structs
  - Pixel format definitions
- [x] Step 3: Define async command traits
  - GpuRequest/GpuResponse traits
  - PendingCommand with Waker support
  - CommandFuture for async/await

### Phase 3: Driver Implementation ✅ COMPLETE
- [x] Step 1: State machine driver (GpuDriver)
  - DriverState enum with full state tracking
  - DriverTelemetry for observability
  - Command building functions for all 2D operations
- [x] Step 2: Response handlers with error checking
- [x] Step 3: GpuRequest/GpuResponse trait implementations

### Phase 4: Integration (IN PROGRESS)
- [x] Step 1: Create VirtioHal trait in levitate-virtio
- [ ] Step 2: Connect GpuDriver to VirtQueue transport
- [ ] Step 3: Refactor GpuState to use new driver
- [ ] Step 4: Update kernel to use new crates

### Phase 5: Hardening
- [ ] Remove virtio-drivers dependency
- [ ] Cleanup dead code
- [ ] Documentation

---

## Handoff Notes

### What's Complete

1. **`levitate-virtio`** crate created:
   - `VirtQueue<SIZE>` - Split virtqueue implementation
   - `Transport` trait + `MmioTransport` implementation
   - `VirtioHal` trait for platform DMA/memory abstraction
   - Status and feature bit constants

2. **`levitate-virtio-gpu`** crate created:
   - `protocol/` - All VirtIO GPU 2D protocol structs
   - `command.rs` - Async command traits (GpuRequest/GpuResponse)
   - `driver.rs` - GpuDriver with state machine

### What Remains

**Phase 4 Gap:** The new `GpuDriver` builds command byte buffers, but needs to be connected to actual VirtQueue transport for sending/receiving.

Current flow (with virtio-drivers):
```
GpuState -> VirtIOGpu::flush() -> (hidden virtqueue)
```

Target flow (with new crates):
```
GpuState -> GpuDriver::build_flush() -> VirtQueue::add_buffer() -> MmioTransport::queue_notify()
```

**Key integration points:**
1. `levitate-hal::VirtioHal` provides DMA allocation
2. `levitate-virtio::VirtQueue` needs to use this for descriptor memory
3. `GpuDriver` response handlers need VirtQueue completion data

### Files Created

```
levitate-virtio/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── queue.rs
    └── transport.rs

levitate-virtio-gpu/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── command.rs
    ├── driver.rs
    └── protocol/
        ├── mod.rs
        ├── commands.rs
        └── formats.rs
```

### Build Status

- [x] `cargo build -p levitate-virtio` passes
- [x] `cargo build -p levitate-virtio-gpu` passes
- [x] `cargo build --release` (full workspace) passes

---

## Notes

- Reference: `docs/planning/virtio-gpu-scanout/VIRTIO_GPU_SPEC.md` for protocol details
- Keep `levitate-gpu` for now but migrate to new driver internally
- Pixel 6 end-goal means we'll later have `levitate-gpu-mali` too

---

## Session End Checklist

- [x] Project builds cleanly
- [x] No new test regressions (existing behavior unchanged)
- [x] Team file updated with progress
- [x] Handoff notes written
- [ ] Phase 4 integration pending (documented above)

