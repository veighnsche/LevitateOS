# TEAM_105: Bugfix VirtQueue DMA

**Created:** 2026-01-05
**Status:** Complete (Plan Ready)
**Workflow:** /make-a-bugfix-plan
**Building on:** TEAM_104 investigation

---

## Pre-Planning Checklist

- [x] Team registered (TEAM_105)
- [x] Bug identified (TEAM_104 investigation)
- [x] Reproducibility confirmed (GPU init timeout)
- [x] Online specs and standards researched
- [x] External kernels checked for reference implementations

---

## Research Summary

### VirtIO 1.1 Specification (OASIS)
**Source**: https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html

**Section 2.6 - Split Virtqueues** alignment requirements:
| Part | Alignment |
|------|----------|
| Descriptor Table | 16 bytes |
| Available Ring | 2 bytes |
| Used Ring | 4 bytes |

### Reference Implementations Found

1. **virtio-drivers crate** (Rust)
   - `#[repr(C, align(16))]` on Descriptor
   - Uses HAL's `Dma::new()` for allocation

2. **Tock OS** (`.external-kernels/tock/chips/virtio/src/queues/split_queue.rs`)
   - `DESCRIPTOR_ALIGNMENT = 16`
   - `AVAILABLE_RING_ALIGNMENT = 2`
   - `USED_RING_ALIGNMENT = 4`
   - `#[repr(C, align(16))]` on descriptor table

---

## Bugfix Plan Created

**Location**: `docs/planning/bugfix-virtqueue-dma/`

| Phase | File | Description |
|-------|------|-------------|
| 1 | phase-1.md | Understanding and Scoping |
| 2 | phase-2.md | Root Cause Analysis |
| 3 | phase-3.md | Fix Design and Validation Plan |
| 4 | phase-4.md | Implementation Steps |
| 5 | phase-5.md | Cleanup and Handoff |

---

## Implementation Summary (from Phase 4)

1. Add `#[repr(C, align(16))]` to Descriptor struct
2. Add `#[repr(C, align(16))]` to VirtQueue struct
3. Add compile-time alignment assertion
4. Change VirtioGpu from `Box<VirtQueue>` to raw pointer + DMA allocation
5. Update queue accessor methods
6. Implement Drop for DMA cleanup

**Estimated effort**: ~10 UoW, ~150 lines of code

---

## Handoff

### For Next Team
1. Read all phase files in `docs/planning/bugfix-virtqueue-dma/`
2. Follow Phase 4 implementation steps in order
3. Run `cargo xtask test` after each step
4. Verify GPU init succeeds and terminal displays

### Key Files
- `levitate-virtio/src/queue.rs` - Alignment fixes
- `levitate-drivers-gpu/src/device.rs` - DMA allocation changes

---

## Session Complete

- [x] VirtIO spec researched
- [x] Reference implementations found (virtio-drivers, Tock OS)
- [x] Bugfix plan created (5 phases)
- [x] Implementation steps documented
- [x] Handoff notes written
