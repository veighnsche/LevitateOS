# Phase 1: Understanding and Scoping

**Bug:** Inconsistent error handling across LevitateOS  
**Author:** TEAM_151  
**Status:** Complete (via TEAM_149 investigation)

---

## Bug Summary

LevitateOS has fragmented, inconsistent error handling that makes debugging difficult and violates Rust idioms (Rule 6: Robust Error Handling).

**Severity:** HIGH  
**Impact:**
- Kernel stability (panics in recoverable paths)
- Debugging difficulty (no error codes)
- Technical debt accumulation

---

## Reproduction Status

**Reproducible:** Yes - observable in code review

**Evidence:**
1. `kernel/src/block.rs` panics on I/O errors
2. `kernel/src/task/user_mm.rs` returns `&'static str` errors
3. `kernel/src/loader/elf.rs` has `ElfError` without codes
4. Error context lost in `SpawnError::from(ElfError)`

---

## Context

### Affected Code Areas

| Module | Current Error Type | Issues |
|--------|-------------------|--------|
| `kernel/src/block.rs` | `panic!` | ✅ FIXED by TEAM_150 |
| `kernel/src/loader/elf.rs` | `ElfError` enum | No error codes |
| `kernel/src/task/process.rs` | `SpawnError` enum | Loses inner error |
| `kernel/src/task/user_mm.rs` | `&'static str` | No type safety |
| `kernel/src/fs/*.rs` | `&'static str` | No type safety |
| `kernel/src/syscall.rs` | `errno` constants | Only 4 codes |
| `levitate-hal/src/mmu.rs` | `&'static str` | No type safety |
| `levitate-hal/src/fdt.rs` | `FdtError` enum | Good pattern |
| `levitate-virtio/src/hal_impl.rs` | `panic!` | Unrecoverable |

### Recent Related Changes

- TEAM_150: Fixed `block.rs` panics, established `BlockError` pattern with codes

---

## Constraints

1. **no_std compatibility** - Cannot use `std::error::Error`, must use `core::error::Error`
2. **ABI stability** - Syscall errno values (-1 to -4) must remain for userspace
3. **Performance** - Error handling must be zero-cost when no error occurs
4. **Backwards compatibility** - Existing callers must still compile (with updates)

---

## Open Questions

| # | Question | Answer |
|---|----------|--------|
| 1 | Should syscall errno stay negative? | **Yes** - keep negative for ABI, internal codes are positive |
| 2 | Error context depth? | **Single level** - inner error preserved but no chains |
| 3 | New crate or inline? | **Inline first** - add error types to existing modules |

---

## Scope Definition

### In Scope
- Add error codes to all kernel error types
- Replace `&'static str` with typed errors
- Implement `Display` and `Error` traits
- Preserve error context in conversions
- Update callers to handle new types

### Out of Scope
- New `levitate-error` crate (deferred - inline first)
- Stack traces (no_std limitation)
- Error recovery/retry logic
- Changing syscall ABI values

---

## Exit Criteria for Phase 1

- [x] Bug fully characterized
- [x] All affected code areas identified  
- [x] Constraints documented
- [x] Scope defined
- [x] Open questions answered
