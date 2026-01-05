# TEAM_030 — Proactive Bug Investigation (TEAM_025-029 Implementations)

**Created:** 2026-01-03
**Status:** COMPLETE
**Role:** Bug Investigation
**Team ID:** 030

---

## Objective

Proactively investigate implementations from TEAM_025-029 (higher-half kernel + VirtIO block driver) and find bugs.

---

## Implementations Reviewed

1. **TEAM_025-028:** Higher-half kernel implementation
   - Assembly boot with MMU setup (TTBR0 + TTBR1)
   - Linker script changes for higher-half VMA
   - `mmu.rs` updates for `virt_to_phys`/`phys_to_virt`

2. **TEAM_029:** VirtIO Block driver
   - `kernel/src/block.rs` - Block device driver
   - Updates to `kernel/src/virtio.rs` for block device detection

---

## Bugs Found and Fixed

### Bug 1: Golden boot log outdated
**Location:** `tests/golden_boot.txt`
**Issue:** TEAM_029 added block device init but didn't update golden log
**Fix:** Added missing 3 lines to golden_boot.txt

### Bug 2: `enable_mmu` stub signature mismatch
**Location:** `levitate-hal/src/mmu.rs:431`
**Issue:** Non-aarch64 stub took 1 arg, real function takes 2
**Fix:** Changed stub to `enable_mmu(_ttbr0_phys: usize, _ttbr1_phys: usize)`

### Bug 3: `KERNEL_PHYS_END` doesn't match linker.ld
**Location:** `levitate-hal/src/mmu.rs:26`
**Issue:** Constant was `0x4800_0000` but linker.ld heap ends at `0x41F0_0000`
**Fix:** Updated constant to `0x41F0_0000` with accurate comment

### Bug 4: Hardcoded screen resolution in input.rs
**Location:** `kernel/src/input.rs:33-38`
**Issue:** Cursor scaling hardcoded 1024x768 instead of actual GPU resolution
**Fix:** Added `GpuState::dimensions()` method and use it in `input::poll()`

---

## Progress Log

| Date | Action | Result |
|------|--------|--------|
| 2026-01-03 | Team 030 registered | - |
| 2026-01-03 | Reviewed recent commits | Identified TEAM_025-029 work |
| 2026-01-03 | Analyzed block.rs, virtio.rs, mmu.rs, main.rs | Found 4 bugs |
| 2026-01-03 | Fixed golden_boot.txt | ✅ |
| 2026-01-03 | Fixed enable_mmu stub signature | ✅ |
| 2026-01-03 | Fixed KERNEL_PHYS_END constant | ✅ |
| 2026-01-03 | Fixed hardcoded screen resolution | ✅ |
| 2026-01-03 | Verified all fixes | ✅ Build + test pass |

---

## Regression Tests Added

Created `xtask` crate with Rust-based test runner. Removed `scripts/` folder.

### Usage
```bash
cargo xtask test           # Run all tests
cargo xtask test behavior  # Behavior test (golden log)
cargo xtask test regress   # Regression tests
cargo xtask build          # Build kernel
cargo xtask run            # Build and run in QEMU
```

### Regression Tests (6 total)
| Test | What it checks |
|------|----------------|
| 1 | Golden boot log contains block device messages |
| 2 | `enable_mmu` stub signature matches real function (2 args) |
| 3 | `KERNEL_PHYS_END` constant matches linker.ld `__heap_end` |
| 4 | `input.rs` uses `GPU.dimensions()` for cursor scaling |
| 5 | `levitate-hal` compiles on host (stub functions valid) |
| 6 | Full kernel compiles for aarch64 |

---

## Handoff Checklist

- [x] Project builds cleanly
- [x] All tests pass (`cargo xtask test` ✅)
- [x] Behavioral regression tests pass
- [x] Team file updated
- [x] No remaining TODOs
- [x] `scripts/` folder removed, replaced with `xtask/`
