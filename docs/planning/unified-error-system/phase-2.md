# Phase 2: Root Cause Analysis

**Bug:** Inconsistent error handling across LevitateOS  
**Author:** TEAM_151  
**Status:** Complete (via TEAM_149 investigation)

---

## Root Cause

**No unified error architecture was designed upfront.**

Each subsystem invented its own error handling approach organically:
- Some used `&'static str` (simple but no type safety)
- Some used enums (type-safe but no codes)
- Some used panic (violates Rule 6)
- None implemented a numbering system

---

## Hypotheses (All Confirmed)

| # | Hypothesis | Status | Evidence |
|---|-----------|--------|----------|
| H1 | No error code system exists | ✅ Confirmed | Only `errno` has codes (-1 to -4) |
| H2 | `&'static str` errors lose context | ✅ Confirmed | 11+ locations use string errors |
| H3 | Error conversions drop information | ✅ Confirmed | `SpawnError::from(ElfError)` discards variant |
| H4 | Panics used for recoverable errors | ✅ Confirmed | `block.rs` had 4 panics (now fixed) |

---

## Key Code Areas

### Priority 1: Must Fix (Panics in recoverable paths)
- ~~`kernel/src/block.rs`~~ ✅ Fixed by TEAM_150
- `levitate-virtio/src/hal_impl.rs` - 4 panics on DMA allocation

### Priority 2: Should Fix (String errors)
- `kernel/src/task/user_mm.rs` - 11 `&'static str` errors
- `levitate-hal/src/mmu.rs` - 8 `&'static str` errors
- `kernel/src/fs/mod.rs` - 3 `&'static str` errors
- `kernel/src/fs/fat.rs` - 1 `&'static str` error
- `kernel/src/fs/ext4.rs` - 1 `&'static str` error

### Priority 3: Should Improve (Enums without codes)
- `kernel/src/loader/elf.rs` - `ElfError` (9 variants, no codes)
- `kernel/src/task/process.rs` - `SpawnError` (3 variants, loses context)
- `kernel/src/net.rs` - `NetError` (3 variants, no codes)
- `levitate-hal/src/fdt.rs` - `FdtError` (2 variants, no codes)

### Priority 4: Keep (Acceptable)
- `kernel/src/boot.rs` - 9 panics (unrecoverable, but improve messages)
- `kernel/src/memory/mod.rs` - 3 panics (unrecoverable)
- `levitate-hal/allocator/*.rs` - ~29 panics (invariant violations, Rule 14)

---

## Data Flow Analysis

### Error Propagation Paths

```
ELF Loading:
  elf.rs::Elf::parse() -> ElfError
    -> process.rs::spawn_from_elf() -> SpawnError (LOSES VARIANT!)
      -> init.rs::spawn_init() -> prints "{:?}"

MMU Operations:
  mmu.rs::map_page() -> &'static str
    -> user_mm.rs::map_user_page() -> &'static str (PASSES THROUGH)
      -> elf.rs::Elf::load() -> ElfError::MappingFailed (LOSES MESSAGE!)

Block I/O:
  block.rs::read_block() -> BlockError (FIXED!)
    -> fs/fat.rs::read() -> BlockError (PROPAGATED!)
      -> fs/mod.rs::read_file() -> Option<Vec<u8>> (CONVERTED TO NONE)
```

### Information Loss Points

1. **`SpawnError::from(ElfError)`** - Loses which ElfError variant
2. **`ElfError::MappingFailed`** - Loses MMU error message
3. **`Option<Vec<u8>>`** - Loses all error information

---

## Investigation Strategy Applied

1. ✅ Grepped for `panic!`, `unwrap()`, `expect()`
2. ✅ Grepped for `&'static str` return types
3. ✅ Grepped for `enum.*Error` definitions
4. ✅ Traced error propagation paths
5. ✅ Categorized by fixability

---

## Root Cause Summary

| Category | Count | Root Cause |
|----------|-------|------------|
| String errors | 24+ | No error type designed |
| Missing codes | 4 enums | No numbering system |
| Context loss | 3+ conversions | No nested error support |
| Panics | 8 (4 fixed) | No Result return types |

---

## Exit Criteria for Phase 2

- [x] Root cause identified
- [x] All affected locations catalogued
- [x] Error propagation paths traced
- [x] Information loss points documented
- [x] Prioritization complete
