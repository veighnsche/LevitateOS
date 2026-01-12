# TEAM_462: Refactor Constants Consolidation

## Objective

Consolidate duplicated constants and hardcoded magic numbers throughout the kernel
to prevent desync bugs and improve maintainability.

## Status

- [x] Phase 1: Discovery & Safeguards (skipped - tests already in place)
- [x] Phase 2: Structural Extraction
- [x] Phase 3: Migration (6 waves completed)
- [x] Phase 4: Cleanup
- [ ] Phase 5: Hardening (future work - lint rules)

## Planning Documents

See `docs/planning/refactor-constants-consolidation/` for detailed phase plans.

## Key Issues Identified

From TEAM_461 audit:

| Issue | Severity | Files Affected | Status |
|-------|----------|----------------|--------|
| PAGE_SIZE defined 4x | High | hal, mm, arch | FIXED |
| Page masks (0xFFF) hardcoded 25x | High | Throughout kernel | FIXED |
| Stack/heap sizes duplicated | High | mm, sched | Partially addressed |
| GDT selectors duplicated | High | hal, arch | Future work |
| Custom syscalls duplicated | Medium | arch/aarch64, arch/x86_64 | Future work |

## Progress Log

### Session 1 (2026-01-12)

**Phase 2: Structural Extraction**
- Created `los_hal::mem::constants` module with:
  - `PAGE_SIZE: usize = 4096`
  - `PAGE_SHIFT: usize = 12`
  - `PAGE_MASK: usize = PAGE_SIZE - 1`
  - `page_align_down(addr)` - align address down to page boundary
  - `page_align_up(addr)` - align address up to page boundary
  - `is_page_aligned(addr)` - check if address is page-aligned
  - `pages_needed(size)` - calculate pages needed for given size
- Added comprehensive unit tests and documentation

**Phase 3: Migration**

Wave 1 - HAL Internal:
- `allocator/buddy.rs` - now imports PAGE_SIZE from constants
- `allocator/slab/page.rs` - re-exports PAGE_SIZE from constants
- `x86_64/mem/mmu.rs` - imports all constants and helpers
- `x86_64/mem/frame_alloc.rs` - uses PAGE_SIZE constant
- `aarch64/mmu/constants.rs` - re-exports from central module
- `virtio.rs` - uses PAGE_SIZE for DMA allocation

Wave 2 - MM Crate:
- `user/mapping.rs` - replaced all `!0xFFF` patterns with helpers
- `user/page_table.rs` - replaced `!0xFFF` patterns
- `user/layout.rs` - TLS_SIZE now uses PAGE_SIZE constant
- `vma.rs` - uses `is_page_aligned()` for debug assertions

Wave 3 - Sched Crate:
- No changes needed - no PAGE_SIZE usage

Wave 4 - Arch Crates:
- No changes needed - only assembly/linker constants

Wave 5 - Levitate Binary:
- `loader/elf.rs` - replaced all page alignment magic numbers
- `memory.rs` - uses `page_align_down/up` and `pages_needed`
- `process.rs` - uses `page_align_down` for TLS alignment

Wave 6 - Syscall Crate:
- `mm.rs` - removed duplicate `page_align_up` function
- Uses central `is_page_aligned()` for validation

**Phase 4: Cleanup**
- Verified only one `const PAGE_SIZE` definition remains
- Fixed remaining `& 0xFFF` in aarch64/mmu/mapping.rs (translate function)

### Session 2 (2026-01-12) - TEAM_463 Review Follow-up

**Additional Cleanup:**
- Fixed `identity_map_range()` - replaced `& !0xFFF` with `page_align_down()`/`page_align_up()`
- Fixed `map_range()` - replaced `& !0xFFF` and `+ 0xFFF` patterns with helpers
- Added missing imports for `page_align_down`, `page_align_up` to mapping.rs

## Results

**Before:**
- 4 separate PAGE_SIZE definitions
- 25+ hardcoded `0xFFF`/`!0xFFF` patterns
- Duplicate `page_align_up` function in syscall crate

**After:**
- Single source of truth in `los_hal::mem::constants`
- All magic numbers replaced with named constants/helpers
- Re-exported through arch-specific modules for compatibility

## Handoff Notes

### Completed
- PAGE_SIZE and page alignment centralization is complete
- All tests pass (unit, behavior)
- Both x86_64 and aarch64 architectures build

### Future Work (Phase 5 - Hardening)
1. Add clippy lint to prevent new hardcoded 0xFFF/4096
2. Consider centralizing other constants:
   - Stack sizes (USER_STACK_PAGES)
   - GDT selectors (x86_64)
   - Custom syscall numbers

### Key Files Modified
- `lib/hal/src/mem/constants.rs` (NEW - central module)
- `lib/hal/src/mem/mod.rs` (NEW - re-exports)
- `lib/hal/src/lib.rs` (added `pub mod mem`)
- Various files across hal, mm, levitate, syscall crates
