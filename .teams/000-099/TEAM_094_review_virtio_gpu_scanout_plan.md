# TEAM_094 Log: Review of virtio-gpu-scanout Plan

**Date:** 2026-01-05  
**Role:** Plan Reviewer  
**Target:** `docs/planning/virtio-gpu-scanout/`

---

## Review Summary

**REVISED ASSESSMENT:** ✅ PLAN DIRECTION IS CORRECT — After deeper analysis, the refactor approach is justified.

**Key Finding:** The `virtio-drivers` crate is a **black box** that prevents debugging. Tock OS builds their VirtIO GPU driver from scratch with explicit protocol structs and state machine. LevitateOS should follow the same approach.

---

## Phase 1 — Questions and Answers Audit

**No `.questions/` files found for this plan.**

Questions file should be created to track:
1. Should we completely replace `virtio-drivers` with custom protocol implementation?
2. What level of compatibility with virtio-drivers HAL traits should we maintain?
3. Should the new crate be `levitate-virtio-gpu` or extend existing `levitate-gpu`?

---

## Phase 2 — Scope and Complexity Check

### ✅ JUSTIFIED Architecture Changes

| Component | Justification |
|-----------|---------------|
| **New `protocol.rs` module** | `virtio-drivers` hides protocol details — can't debug what we can't see |
| **New `resource.rs` RAII module** | Need explicit resource lifetime management for debugging |
| **Command Queue Manager** | `virtio-drivers` uses blocking `add_notify_wait_pop` — no async, no visibility |
| **Structured Observability** | Current telemetry only counts flushes — need command-level tracing |
| **State Machine** | Tock uses explicit `VirtIOGPUState` enum — we should too |

### The Real Problem: `virtio-drivers` is a Black Box

**What `virtio-drivers` hides:**
```rust
// This is ALL we can see:
self.gpu.flush()  // Returns Ok(()) or Err — no details
```

**What Tock exposes:**
```rust
// Tock has explicit state machine:
enum VirtIOGPUState {
    Uninitialized,
    InitializingResourceCreate2D,
    InitializingResourceAttachBacking,
    InitializingSetScanout,
    ...
}
// And custom protocol structs with full visibility
```

### 2 Days of Debugging = Simple Fixes Already Tried

The "simple fixes" (timer flush, virtio-vga) have been attempted. The problem is we **can't see what's actually happening** inside `virtio-drivers`.

---

## Phase 3 — Architecture Alignment

### Current Architecture (BROKEN)

```
kernel/src/gpu.rs         → Thin re-export (no value)
levitate-gpu/src/gpu.rs   → Wraps virtio-drivers (black box)
virtio-drivers            → High-level API, no debugging access
```

### Reference: Tock Architecture (CORRECT)

```
messages/                 → Protocol structs (ctrl_header, set_scanout, etc.)
  ctrl_header.rs          → CtrlType enum, CtrlHeader struct
  set_scanout.rs          → SetScanoutReq, SetScanoutResp
  resource_flush.rs       → ResourceFlushReq, ResourceFlushResp
  ...
mod.rs                    → VirtIOGPU driver with explicit state machine
```

### Proposed Architecture (CORRECT)

```
levitate-virtio/          → General VirtIO transport (queue, MMIO)
levitate-virtio-gpu/      → GPU protocol structs + driver
  protocol/
    mod.rs                → VirtIOGPUReq, VirtIOGPUResp traits
    ctrl_header.rs        → CtrlType, CtrlHeader
    set_scanout.rs        → SetScanoutReq/Resp
    resource_flush.rs     → ResourceFlushReq/Resp
    ...
  driver.rs               → State machine, command queue
  display.rs              → DrawTarget impl
levitate-gpu/             → High-level API (optional, wraps virtio-gpu)
```

**This follows Tock's pattern exactly.**

---

## Phase 4 — Global Rules Compliance

| Rule | Status | Notes |
|------|--------|-------|
| Rule 0 (Quality > Speed) | ✅ | Building proper debugging infrastructure |
| Rule 1 (SSOT) | ✅ | Plan in correct location |
| Rule 4 (Regression Protection) | ⚠️ | Add explicit test for SET_SCANOUT response parsing |
| Rule 5 (Breaking Changes) | ✅ | Breaking from virtio-drivers is correct — it's the wrong abstraction |
| Rule 6 (No Dead Code) | ⚠️ | Remove virtio-drivers dependency after migration |
| Rule 8 (Ask Questions Early) | ⚠️ | Create questions file for crate naming/structure |

---

## Phase 5 — Verification and References

### Verified: Tock's Approach Works

Tock OS has a working VirtIO GPU implementation with:
- Custom protocol structs in `messages/` module
- Explicit state machine (`VirtIOGPUState` enum)
- Full visibility into every command and response
- No dependency on `virtio-drivers` crate

Source: `.external-kernels/tock/chips/virtio/src/devices/virtio_gpu/`

### The Problem with `virtio-drivers`

1. **Blocking API** — `add_notify_wait_pop` blocks, no async support
2. **Hidden State** — Can't inspect what commands were sent
3. **No Tracing** — Only returns `Ok(())` or error code
4. **Hardcoded Constants** — `RESOURCE_ID_FB = 1`, `SCANOUT_ID = 0` — can't experiment

---

## Recommendations

### ✅ APPROVE Plan Direction with Modifications

The plan's direction is correct. Modifications needed:

### Phase 1: Create General-Purpose VirtIO Crates

**New crate: `levitate-virtio`**
- VirtQueue abstraction (not GPU-specific)
- MMIO transport wrapper
- General buffer management

**New crate: `levitate-virtio-gpu`**
- Protocol structs (following Tock's `messages/` pattern)
- State machine driver
- Command tracing/logging built-in

### Phase 2: Debugging Infrastructure

- Log every command sent with full struct contents
- Log every response received with parsed CtrlType
- Add QEMU QMP integration for display state queries

### Phase 3: Migration

- Port GpuState to use new driver
- Keep Display/DrawTarget API unchanged
- Add feature flag to switch between implementations

### Phase 4: Verification

- Compare command sequences between old and new
- Test SET_SCANOUT with various rect configurations
- Document what actually works

### Phase 5: Cleanup

- Remove `virtio-drivers` dependency from GPU path
- Update documentation
- Archive debugging logs

---

## Crate Structure Recommendation

```
levitate-virtio/          # General VirtIO (transport agnostic)
├── src/
│   ├── lib.rs
│   ├── queue.rs          # VirtQueue implementation
│   └── transport.rs      # Transport trait

levitate-virtio-gpu/      # GPU-specific protocol + driver
├── src/
│   ├── lib.rs
│   ├── protocol/
│   │   ├── mod.rs        # Traits: VirtIOGPUReq, VirtIOGPUResp
│   │   ├── ctrl_header.rs
│   │   ├── set_scanout.rs
│   │   ├── resource_create_2d.rs
│   │   ├── resource_flush.rs
│   │   └── transfer_to_host.rs
│   ├── driver.rs         # State machine, command queue
│   └── display.rs        # DrawTarget impl
```

This matches Tock's architecture exactly and provides full debugging visibility.

---

## Handoff

- [x] Team file created
- [x] Review complete  
- [x] **REVISED: Plan direction APPROVED**
- [ ] Update plan files with new crate structure
- [ ] Create questions file for open decisions

