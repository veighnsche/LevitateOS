# Buddy Allocator Questions

**Created By**: TEAM_042 (Plan Review)
**Related Plan**: `docs/planning/buddy-allocator/`

---

## Q1: Initial Heap Allocation Size

**Context**: Phase 2 specifies 32MB, Phase 3 specifies 16MB for the initial `LockedHeap` region.

**Question**: What should the initial heap size be?

- **Option A**: 16MB (smaller footprint, may need expansion logic)
- **Option B**: 32MB (more headroom, simpler)
- **Option C**: Dynamic based on available RAM (e.g., 1/32 of total)

**Answer**: **Option C (Dynamic)** with bounds.

**Decision (TEAM_042, 2026-01-04)**:
- Formula: `total_ram / 128`, clamped to `[16MB, 64MB]`
- QEMU 1GB → 16MB heap (minimum)
- Pixel 6 8GB → 64MB heap (maximum)
- Rationale: Scales appropriately without over-allocating on small systems or under-allocating on target hardware. No expansion logic needed initially since 64MB is generous for kernel heap.

---

## Q2: mem_map Size Cap

**Context**: The `mem_map` array requires ~24 bytes per 4KB page. For 4GB RAM, this is ~24MB of metadata.

**Question**: Should there be a maximum supported RAM size to cap `mem_map` overhead?

- **Option A**: No cap (support arbitrary RAM sizes)
- **Option B**: Cap at 4GB (24MB overhead max)
- **Option C**: Cap at 1GB for now (6MB overhead), expand later

**Answer**: **Option A (No cap)**.

**Decision (TEAM_042, 2026-01-04)**:
- Pixel 6 has 8GB RAM → 2,097,152 pages × 24 bytes = **48MB metadata**
- This is ~0.6% of total RAM — acceptable overhead
- Options B/C would break the target hardware
- Implementation: Parse all RAM from DTB, allocate full `mem_map` at boot

---

## Q3: PT_POOL Replacement Mechanism

**Context**: The plan removes `PT_POOL` but doesn't specify how `mmu.rs` will request pages from the Buddy Allocator.

**Question**: How should MMU obtain new page tables post-Buddy?

- **Option A**: MMU calls `buddy::alloc(0)` directly (adds kernel → HAL coupling)
- **Option B**: Pass a callback/closure to MMU at init time
- **Option C**: Keep `PT_POOL` for MMU, only use Buddy for heap/userspace

**Answer**: **Option B (Callback/trait injection)**.

**Decision (TEAM_042, 2026-01-04)**:
- HAL must remain decoupled from kernel internals (correct dependency direction)
- Define trait in HAL: `pub trait PageAllocator { fn alloc_page() -> Option<PhysAddr>; }`
- Kernel implements this trait using Buddy Allocator
- Pass `&'static dyn PageAllocator` to `mmu::init()` or store via `set_allocator()`
- Fallback: Keep `PT_POOL` as boot-time allocator before Buddy is ready
- Rationale: Clean architecture, testable (can mock allocator), no reverse dependencies

---

## Q4: Edge Case Behavior

**Context**: Plan constraints mention memory holes but implementation steps don't address them.

**Question**: What should happen if:

1. DTB reports zero RAM regions?
2. All RAM is marked reserved?
3. `mem_map` doesn't fit in any contiguous hole?

- **Option A**: Panic with clear error message
- **Option B**: Boot with degraded functionality (no dynamic allocation)

**Answer**: **Option A (Panic with clear error)**.

**Decision (TEAM_042, 2026-01-04)**:
- A kernel cannot operate without memory — "degraded functionality" is meaningless
- Clear panic messages are critical for Pixel 6 bringup (no serial console initially)
- Specific panic strings:
  1. `"PANIC: No RAM regions found in DTB"`
  2. `"PANIC: All RAM is reserved — no free memory"`
  3. `"PANIC: Cannot allocate mem_map — insufficient contiguous memory"`
- Memory holes (MMIO gaps): Skip non-RAM regions during `add_memory()` — not an error, just skip
- Rationale: Fail fast with actionable errors. Silent failures waste hours on new hardware.

