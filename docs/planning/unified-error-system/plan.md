# Unified Error System for LevitateOS

**Author:** TEAM_149  
**Created:** 2026-01-06  
**Status:** Draft  
**Related:** `.teams/TEAM_149_investigate_error_handling.md`

## Problem Statement

LevitateOS lacks a unified error handling architecture. Current state:
- 14+ files with inconsistent error handling
- Mix of `&'static str`, custom enums, POSIX-like codes, and panics
- No error numbering system for debugging
- Lost error context during conversions

## Goals

1. **Type-safe errors** - All errors use proper Rust enums
2. **Numeric error codes** - Every error has a unique code for debugging
3. **Context preservation** - Nested errors retain inner cause
4. **Zero panics in recoverable paths** - Rule 6 compliance
5. **Human-readable output** - `Display` impls for logging

## Non-Goals

- Linux errno compatibility (we have custom ABI)
- Stack traces (no_std limitation)
- Error recovery/retry logic (separate concern)

---

## Phase 1: Design Error Code System

### Error Code Format

```
0xSSCC where:
  SS = Subsystem (00-FF)
  CC = Error code within subsystem (00-FF)
```

### Subsystem Allocation

| Range | Subsystem | Description |
|-------|-----------|-------------|
| `0x00xx` | Core | Generic kernel errors |
| `0x01xx` | MMU | Memory management |
| `0x02xx` | ELF | ELF loader |
| `0x03xx` | Process | Process/task management |
| `0x04xx` | Syscall | System call errors |
| `0x05xx` | FS | Filesystem |
| `0x06xx` | Block | Block device |
| `0x07xx` | Net | Network |
| `0x08xx` | GPU | Graphics |
| `0x09xx` | FDT | Device tree |
| `0x0Axx` | PCI | PCI bus |
| `0x0Bxx` | VirtIO | VirtIO devices |

### Example Codes

```rust
// MMU errors (0x01xx)
pub const E_MMU_ALLOC_FAILED: u16 = 0x0101;
pub const E_MMU_NOT_MAPPED: u16 = 0x0102;
pub const E_MMU_INVALID_VA: u16 = 0x0103;
pub const E_MMU_MISALIGNED: u16 = 0x0104;

// ELF errors (0x02xx)
pub const E_ELF_TOO_SHORT: u16 = 0x0201;
pub const E_ELF_BAD_MAGIC: u16 = 0x0202;
pub const E_ELF_NOT_64BIT: u16 = 0x0203;
// ... etc
```

---

## Phase 2: Create Core Error Infrastructure

### New Crate: `levitate-error`

```
levitate-error/
├── Cargo.toml
└── src/
    ├── lib.rs       # KernelError enum, traits
    ├── codes.rs     # Error code constants
    └── subsystems/  # Per-subsystem error types
        ├── mod.rs
        ├── mmu.rs
        ├── elf.rs
        ├── process.rs
        └── ...
```

### Core Types

```rust
/// Kernel-wide error type with error code
#[derive(Debug, Clone)]
pub struct KernelError {
    code: u16,
    kind: ErrorKind,
}

/// Error categories
#[derive(Debug, Clone)]
pub enum ErrorKind {
    Mmu(MmuError),
    Elf(ElfError),
    Process(ProcessError),
    Syscall(SyscallError),
    Fs(FsError),
    // ... etc
}

impl KernelError {
    /// Get numeric error code for debugging
    pub fn code(&self) -> u16 { self.code }
    
    /// Get human-readable name
    pub fn name(&self) -> &'static str { ... }
}

impl core::fmt::Display for KernelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "E{:04X}: {}", self.code, self.name())
    }
}
```

---

## Phase 3: Migrate Existing Code

### 3.1 levitate-hal/src/mmu.rs

**Current:**
```rust
pub fn map_page(...) -> Result<(), &'static str>
```

**After:**
```rust
pub fn map_page(...) -> Result<(), MmuError>
```

### 3.2 levitate-hal/src/fdt.rs

Already uses `FdtError` enum - just add error codes.

### 3.3 kernel/src/loader/elf.rs

**Current:** `ElfError` enum without codes
**After:** Add `code()` method returning `0x02xx` values

### 3.4 kernel/src/task/process.rs

**Current:** `SpawnError` loses inner `ElfError`
**After:**
```rust
pub enum SpawnError {
    Elf(ElfError),           // Preserves inner error
    PageTableCreation(MmuError),
    StackSetup(MmuError),
}
```

### 3.5 kernel/src/task/user_mm.rs

Replace all `&'static str` with `MmuError` variants.

### 3.6 kernel/src/block.rs

**Current:** Panics on error
**After:** Return `Result<(), BlockError>`

### 3.7 kernel/src/fs/*.rs

Replace `&'static str` with `FsError` enum.

### 3.8 kernel/src/syscall.rs

Keep existing errno values for ABI compatibility, but add internal error types.

---

## Phase 4: Testing

1. **Unit tests** for each error type's `Display` and `code()` 
2. **Integration test** verifying error propagation
3. **Regression tests** ensuring existing behavior unchanged
4. **Golden test update** if error output format changes

---

## Work Breakdown

| UoW | Task | Est. Lines | Risk |
|-----|------|------------|------|
| 1 | Create `levitate-error` crate skeleton | ~100 | Low |
| 2 | Define error codes and subsystem types | ~200 | Low |
| 3 | Migrate `levitate-hal/mmu.rs` | ~50 | Medium |
| 4 | Migrate `levitate-hal/fdt.rs` | ~20 | Low |
| 5 | Migrate `kernel/loader/elf.rs` | ~40 | Low |
| 6 | Migrate `kernel/task/*.rs` | ~80 | Medium |
| 7 | Migrate `kernel/fs/*.rs` | ~60 | Medium |
| 8 | Migrate `kernel/block.rs` (remove panics) | ~30 | High |
| 9 | Migrate `levitate-virtio/hal_impl.rs` (remove panics) | ~40 | Medium |
| 10 | Migrate remaining kernel modules | ~50 | Low |
| 11 | Improve boot.rs panic messages | ~20 | Low |
| 12 | Add tests and documentation | ~100 | Low |

**Total:** ~790 lines across ~17 files

---

## Appendix: Panic Inventory

See `.teams/TEAM_149_investigate_error_handling.md` for full inventory.

### Must Fix (P0)
- `kernel/src/block.rs`: 4 panics on I/O errors
- `levitate-virtio/src/hal_impl.rs`: 4 panics on DMA allocation

### Keep But Improve (P1)
- `kernel/src/boot.rs`: 9 panics - unrecoverable, improve messages
- `kernel/src/memory/mod.rs`: 3 panics - unrecoverable

### Acceptable (Invariant Violations)
- `levitate-hal/allocator/*.rs`: ~29 panics - correct per Rule 14
- `kernel/src/loader/elf.rs`: 14 unwraps - infallible after size check

---

## Open Questions

1. **Should syscall errno stay negative?** Current ABI uses -1 to -4. Options:
   - Keep negative for userspace ABI, use positive internally
   - Unify to single scheme

2. **Error context depth?** Should we support error chains (cause of cause)?

3. **no_std Display?** Need `core::fmt::Display` without alloc - use static strings only.

---

## Success Criteria

- [ ] All `&'static str` errors replaced with typed enums
- [ ] All errors have unique numeric codes
- [ ] `block.rs` panics replaced with Result
- [ ] All tests pass
- [ ] Error output includes codes: `E0102: Page not mapped`
