# TEAM_108: Investigate GPU Crate Alignment

**Created:** 2026-01-05
**Status:** Complete
**Workflow:** /investigate-a-bug

---

## Purpose

Investigate why there are still 2 GPU crates and whether current state aligns with original decisions and plans.

---

## Team Files Read

| Team | Role | Key Findings |
|------|------|-------------|
| TEAM_092 | GPU Observability Refactor | Decoupled `levitate-gpu` and `levitate-terminal` |
| TEAM_094 | Plan Reviewer | Approved custom VirtIO approach (like Tock OS) |
| TEAM_098 | Implementer | Created `levitate-virtio` + `levitate-virtio-gpu` |
| TEAM_100 | Implementation Reviewer | Found VirtQueue bugs, **REVERTED to levitate-gpu** |
| TEAM_101 | Architecture Investigator | Created crate reorganization plan |
| TEAM_102 | Plan Reviewer | Approved reorganization plan |
| TEAM_103 | Implementer | Renamed crate to `levitate-drivers-gpu`, found VirtQueue still broken |
| TEAM_104 | Bug Investigator | Identified VirtQueue DMA alignment issues |
| TEAM_105 | Bugfix Planner | Created VirtQueue DMA fix plan |
| TEAM_106 | Implementer | Implemented VirtQueue DMA fix |
| TEAM_107 | Reviewer | Attempted migration, **STILL BROKEN**, reverted |

---

## Historical Timeline

```
TEAM_092: Decoupled levitate-gpu from levitate-terminal
    ↓
TEAM_094: Approved plan to replace virtio-drivers with custom implementation
    ↓
TEAM_098: Created levitate-virtio + levitate-virtio-gpu crates
    ↓
TEAM_100: Found bugs, REVERTED kernel to levitate-gpu (working)
    ↓
TEAM_101: Created full crate reorganization plan:
          Decision: "Option A - Fix levitate-virtio-gpu, delete levitate-gpu"
    ↓
TEAM_103: Started implementation, renamed to levitate-drivers-gpu
          Found VirtQueue still broken, deferred migration
    ↓
TEAM_104-106: Investigated and fixed VirtQueue DMA alignment
    ↓
TEAM_107: Attempted migration again, STILL BROKEN (additional issues)
          Reverted to levitate-gpu
```

---

## The Original Decision (TEAM_101)

**Location:** `docs/planning/crate-reorganization/README.md`

```markdown
| Decision | Choice |
|----------|--------|
| GPU Consolidation | **Option A**: Fix `levitate-virtio-gpu`, delete `levitate-gpu` |
| Refactor Scope | **FULL**: Complete reorganization |
```

**The plan was:**
1. Fix VirtQueue DMA bugs (Phase 2 Step 1)
2. Migrate kernel to levitate-drivers-gpu (Phase 3)
3. Delete levitate-gpu (Phase 4)

---

## Current State vs Plan

| Plan Phase | Status | Notes |
|------------|--------|-------|
| Phase 1: Discovery | ✅ Complete | TEAM_101 |
| Phase 2 Step 1: Fix VirtQueue | ⚠️ **PARTIALLY COMPLETE** | DMA fix done, but driver still broken |
| Phase 2 Step 2: Move VirtIO HAL | ✅ Complete | TEAM_103 |
| Phase 2 Step 3: Rename GPU crate | ✅ Complete | TEAM_103 (now levitate-drivers-gpu) |
| Phase 3: Migration | ❌ **BLOCKED** | Driver still times out |
| Phase 4: Delete levitate-gpu | ❌ **BLOCKED** | Depends on Phase 3 |

---

## Why Two GPU Crates Exist

### `levitate-gpu`
- Uses external `virtio-drivers` crate
- Works correctly
- Used by kernel currently
- **TEMPORARY** - Was supposed to be deleted

### `levitate-drivers-gpu`
- Custom implementation using `levitate-virtio`
- Has VirtQueue DMA fix (TEAM_106)
- **STILL BROKEN** - times out on GPU commands
- Additional issues beyond DMA alignment

---

## Root Cause: Why We're Stuck

### Phase 2 Step 1 (Fix VirtQueue) is NOT COMPLETE

TEAM_106 fixed the known issues:
1. ✅ Added `#[repr(C, align(16))]` to Descriptor
2. ✅ Added `#[repr(C, align(16))]` to VirtQueue
3. ✅ Changed to `dma_alloc()` instead of Box

But TEAM_107 discovered these fixes were **necessary but not sufficient**.

**Remaining issues in levitate-drivers-gpu:**
- MmioTransport implementation differences
- VirtQueue command flow differences  
- GPU protocol handling differences

See `.questions/TEAM_107_gpu_driver_issues.md` for details.

---

## ARE WE FOLLOWING THE PLAN?

### Answer: **YES, but stuck at a blocker**

We are still following the original decision (Option A: Fix and migrate).
But Phase 2 Step 1 is not fully complete.

**The fallback plan exists (from TEAM_102 review):**
> "If VirtQueue DMA fix proves infeasible within 1 week of effort:
> 1. Revisit Option B: Keep levitate-gpu as canonical driver
> 2. Delete levitate-virtio-gpu instead"

---

## Decision Required

**USER must decide:**

### Option A: Continue with original plan
- Keep investigating levitate-drivers-gpu issues
- Compare implementations line-by-line with virtio-drivers
- More debugging work needed
- End goal: Delete levitate-gpu

### Option B: Invoke fallback plan  
- Accept that levitate-gpu (virtio-drivers) works
- Delete levitate-drivers-gpu and levitate-virtio
- Keep virtio-drivers as dependency
- Simpler but less control

### Option C: Hybrid approach
- Keep levitate-gpu for GPU
- Use levitate-virtio for other drivers (block, net, input) later
- Defer GPU migration indefinitely

---

## Recommendation

Based on investigation, the custom GPU driver has **fundamental issues** beyond DMA alignment. The effort to fix it may exceed the benefit.

**TEAM_108 recommends Option B or C** unless debugging the custom driver is a priority.

---

## Session Complete

- [x] Read all relevant team files
- [x] Traced history of decisions
- [x] Identified why two crates exist
- [x] Determined we ARE following plan (stuck at blocker)
- [x] Documented findings
- [x] Presented options to user
