# TEAM_101: Investigate Crate Organization

**Date:** 2026-01-05  
**Role:** Architecture Investigator  
**Focus:** Why are there 2 GPU crates? Are abstractions correct?

---

## Bug/Issue Report

User questions:
1. Why do we have `levitate-gpu` AND `levitate-virtio-gpu`?
2. Should they be consolidated?
3. How should crates be organized (drivers vs abstractions)?
4. Are the current abstractions correct?

---

## Phase 1: Current Crate Inventory

### Crate Purposes

| Crate | Purpose | Dependencies | Role |
|-------|---------|--------------|------|
| `levitate-utils` | Core utilities (Spinlock, RingBuffer, CPIO, hex) | None | **Foundation** |
| `levitate-hal` | Hardware Abstraction Layer (MMU, GIC, Timer, UART, Console, FDT) | levitate-utils, levitate-virtio, virtio-drivers | **Platform HAL** |
| `levitate-virtio` | VirtIO transport layer (VirtQueue, MmioTransport) | bitflags | **Transport** |
| `levitate-gpu` | GPU driver using virtio-drivers crate | levitate-hal, virtio-drivers, embedded-graphics | **Driver (legacy)** |
| `levitate-virtio-gpu` | GPU driver using levitate-virtio | levitate-virtio, embedded-graphics | **Driver (new)** |
| `levitate-terminal` | Platform-agnostic terminal emulator | levitate-utils, embedded-graphics | **Subsystem** |
| `levitate-kernel` | Main kernel binary | All of the above + more | **Binary** |

---

## Phase 2: Dependency Analysis

```
levitate-kernel
├── levitate-hal ─────────────┬── levitate-utils
│                             ├── levitate-virtio
│                             └── virtio-drivers (external)
├── levitate-gpu ─────────────┬── levitate-hal
│                             ├── virtio-drivers (external)
│                             └── embedded-graphics
├── levitate-virtio-gpu ──────┬── levitate-virtio
│                             └── embedded-graphics
├── levitate-terminal ────────┬── levitate-utils
│                             └── embedded-graphics
└── virtio-drivers (external, direct dependency!)
```

---

## Phase 3: Problems Identified

### Problem 1: **Two GPU Crates** (CONFIRMED)

**Why do we have two GPU crates?**

1. **`levitate-gpu`** - Uses external `virtio-drivers` crate directly
   - Works correctly
   - Has less code visibility (black box)
   - Quick to implement but limits control

2. **`levitate-virtio-gpu`** - Uses our own `levitate-virtio` transport
   - Written for full protocol visibility
   - Currently broken (VirtQueue DMA bugs)
   - Goal was to remove `virtio-drivers` dependency

**Root Cause:** The new driver was created to replace the old one but was never finished.

**Verdict:** ❌ Should NOT have two. Must consolidate.

---

### Problem 2: **Muddled HAL Responsibilities**

`levitate-hal` does too much:
- ✅ CPU-level HAL (MMU, GIC, Timer, UART) - **Correct**
- ❌ VirtIO HAL impl (`src/virtio.rs`) - **Wrong layer**
- ❌ Depends on `virtio-drivers` external crate - **Should not be here**

**HAL should only contain platform-specific, non-driver code.**

---

### Problem 3: **Kernel Has Direct External Dependencies**

The kernel directly depends on:
- `virtio-drivers` - Should go through a driver crate
- `embedded-graphics` - Should go through display/terminal crate
- `embedded-sdmmc` - No abstraction crate
- `ext4-view` - No abstraction crate

**Kernel should only depend on internal crates, not external libraries.**

---

### Problem 4: **No Clear Driver vs Subsystem Naming**

The naming is inconsistent:
- `levitate-gpu` - Is it a driver or abstraction?
- `levitate-terminal` - Clearly a subsystem ✅
- `levitate-virtio` - Transport layer ✅

---

## Phase 4: Correct Abstraction Layers

Reference architecture (from Tock, Theseus, Redox):

```
┌─────────────────────────────────────────────────────────┐
│                    KERNEL BINARY                        │
└─────────────────┬───────────────────────────────────────┘
                  │
┌─────────────────┴───────────────────────────────────────┐
│               SUBSYSTEMS (high-level)                   │
│   terminal, filesystem, network-stack, task-scheduler   │
└─────────────────┬───────────────────────────────────────┘
                  │
┌─────────────────┴───────────────────────────────────────┐
│               DRIVERS (device-specific)                 │
│   virtio-gpu, virtio-blk, virtio-net, virtio-input     │
│   uart, gic, timer                                      │
└─────────────────┬───────────────────────────────────────┘
                  │
┌─────────────────┴───────────────────────────────────────┐
│               TRANSPORT / BUS LAYERS                    │
│   virtio-transport, pci, mmio                           │
└─────────────────┬───────────────────────────────────────┘
                  │
┌─────────────────┴───────────────────────────────────────┐
│               HAL (CPU/Platform specific)               │
│   mmu, exceptions, cpu-features, memory-map             │
└─────────────────┬───────────────────────────────────────┘
                  │
┌─────────────────┴───────────────────────────────────────┐
│               UTILS (no dependencies)                   │
│   spinlock, ringbuffer, data-structures                 │
└─────────────────────────────────────────────────────────┘
```

---

## Phase 5: Recommendations

### Immediate Actions

1. **Delete `levitate-gpu`** - It's just a wrapper around `virtio-drivers`
2. **Fix `levitate-virtio-gpu`** - Make it the canonical GPU driver
3. **Move VirtIO HAL impl** from `levitate-hal` to `levitate-virtio`

### Proposed Crate Reorganization

| Current | Proposed | Notes |
|---------|----------|-------|
| `levitate-utils` | `levitate-utils` | Keep as-is ✅ |
| `levitate-hal` | `levitate-hal` | Remove virtio.rs, keep pure HAL |
| `levitate-virtio` | `levitate-virtio` | Add HAL impl from levitate-hal |
| `levitate-gpu` | **DELETE** | Replace with fixed levitate-virtio-gpu |
| `levitate-virtio-gpu` | `levitate-drivers-gpu` | Rename for clarity |
| `levitate-terminal` | `levitate-terminal` | Keep as-is ✅ |
| (new) | `levitate-drivers-block` | Extract from kernel |
| (new) | `levitate-drivers-net` | Extract from kernel |
| (new) | `levitate-drivers-input` | Extract from kernel |
| (new) | `levitate-fs` | Filesystem abstractions |

### Naming Convention

```
levitate-<category>[-<subcategory>]

Categories:
- utils     : Foundation utilities
- hal       : CPU/platform HAL
- virtio    : VirtIO transport
- drivers   : Device drivers (drivers-gpu, drivers-blk, etc.)
- terminal  : Terminal subsystem
- fs        : Filesystem subsystem
- net       : Networking subsystem (future)
```

---

## Decision Required

**Before proceeding with reorganization, USER must decide:**

1. **Fix levitate-virtio-gpu or keep levitate-gpu?**
   - Option A: Fix VirtQueue bugs, delete levitate-gpu (cleaner, more work)
   - Option B: Keep levitate-gpu, delete levitate-virtio-gpu (pragmatic, less control)

2. **How aggressive should the refactor be?**
   - Minimal: Just consolidate GPU crates
   - Medium: + Move virtio HAL, extract drivers from kernel
   - Full: Complete reorganization per recommendations

3. **Timeline?**
   - This is significant work. Create a planning document?

---

## Session End Checklist

- [x] Project builds cleanly
- [x] All tests pass
- [x] Investigation complete
- [x] Root cause identified (incomplete refactor)
- [x] Recommendations documented
- [x] USER decision on path forward

---

## USER Decision (2026-01-05)

1. **GPU Consolidation:** Option A - Fix levitate-virtio-gpu, delete levitate-gpu
2. **Refactor Scope:** FULL reorganization
3. **Planning:** Created comprehensive refactor plan

## Refactor Plan Created

Location: `docs/planning/crate-reorganization/`

| Document | Description |
|----------|-------------|
| README.md | Overview and status tracking |
| phase-1.md | Discovery and Safeguards |
| phase-2.md | Structural Extraction (7 steps) |
| phase-3.md | Migration |
| phase-4.md | Cleanup |
| phase-5.md | Hardening and Handoff |
| phase-2-step-1.md | Fix VirtQueue DMA Bugs (critical!) |
| phase-2-step-2.md | Move VirtIO HAL |
| phase-2-step-3.md | Rename GPU Crate |
| phase-2-step-4.md | Extract Block Driver |
| phase-2-step-5.md | Extract Net Driver |
| phase-2-step-6.md | Extract Input Driver |
| phase-2-step-7.md | Create Filesystem Crate |

### Critical Path

```
Phase 2 Step 1 (Fix VirtQueue) 
    → Phase 2 Step 2 (Move VirtIO HAL)
    → Phase 2 Step 3 (Rename GPU)
    → Phase 3 Step 1 (Migrate GPU)
    → Phase 4 Step 1 (Delete levitate-gpu)
```

### Estimated Total Effort

~15-20 Units of Work across all phases

---

## Status

**TEAM_101:** Investigation complete, refactor plan created.

Next team should begin with Phase 1 verification, then Phase 2 Step 1.
