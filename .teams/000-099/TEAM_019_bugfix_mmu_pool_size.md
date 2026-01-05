# TEAM_019 — Bugfix: MMU Page Table Pool Size

**Created:** 2026-01-03  
**Status:** In Progress

---

## Bug Summary

The static page table pool in `levitate-hal/src/mmu.rs` has only **8 tables**, but identity mapping the kernel memory range (0x40080000 to 0x48000000, ~128MB) with 4KB pages requires approximately **67 tables**:

- 1× L0 table (covers 512GB per entry)
- 1× L1 table (covers 1GB per entry)  
- 1× L2 table (covers 2MB per entry, ~64 entries used)
- ~64× L3 tables (each covers 512 × 4KB = 2MB)

**Impact:** `identity_map_range()` will fail with "Page table pool exhausted" before completing the kernel identity mapping, causing a kernel panic or inability to enable MMU.

---

## Root Cause

Line 344 in `mmu.rs`:
```rust
static mut PT_POOL: [PageTable; 8] = [const { PageTable::new() }; 8];
```

The pool size of 8 is insufficient for 4KB page mappings across 128MB.

---

## Fix Strategy Options

1. **Increase pool size** — Simple, but wastes memory if not all entries are used
2. **Implement 2MB block mappings** — More efficient, fewer tables needed (~4 total)
3. **Dynamic frame allocator** — Best long-term, but more complex

---

## Planning Documents

- `docs/planning/mmu-page-tables/bugfix-pool-size/phase-1.md` — Understanding & Scoping
- `docs/planning/mmu-page-tables/bugfix-pool-size/phase-2.md` — Root Cause Analysis
- `docs/planning/mmu-page-tables/bugfix-pool-size/phase-3.md` — Fix Design

---

## Progress Log

| Date | Action |
|------|--------|
| 2026-01-03 | Team created, bug analysis complete |
| 2026-01-03 | 2MB block mapping implemented and verified by TEAM_020 |
