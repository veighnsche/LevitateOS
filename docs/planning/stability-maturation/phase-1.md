# LevitateOS Stability Maturation Plan

**TEAM_311**: Foundation Layer Audit and ABI/API Stability
**Status**: Analysis Complete - Awaiting Implementation Approval
**Date**: 2026-01-08

---

## Executive Summary

The codebase has grown organically with 310+ team contributions. While functional, the lowest-level components exhibit **tight coupling** and **fragile interfaces** that cause cascade failures when modified. This document identifies the root causes and proposes a maturation strategy.

---

## 1. Identified Fragility Zones

### 1.1 Syscall Number Definitions (CRITICAL)

**Problem**: Syscall numbers are defined in **3 separate places** that must stay synchronized:

| Location | Purpose |
|----------|---------|
| `kernel/src/arch/aarch64/mod.rs` | Kernel AArch64 syscall dispatch |
| `kernel/src/arch/x86_64/mod.rs` | Kernel x86_64 syscall dispatch |
| `userspace/libsyscall/src/sysno.rs` | Userspace syscall invocation |

**Risk**: Adding/changing a syscall requires editing 3 files. If any drift, userspace calls wrong syscall.

**Current State**:
- Custom syscalls (Spawn, SpawnArgs, etc.) use numbers 1000+
- Standard syscalls use Linux numbers but are duplicated per-arch
- `libsyscall/sysno.rs` imports from `linux-raw-sys` but custom syscalls are hardcoded

**Proposed Fix**: Create a **single source of truth** crate:
```
crates/abi/
├── src/
│   ├── lib.rs          # Re-exports
│   ├── syscall.rs      # Syscall numbers (shared by kernel + userspace)
│   ├── errno.rs        # Error codes
│   ├── stat.rs         # Stat structure (arch-specific layouts)
│   └── termios.rs      # Terminal structures
```

### 1.2 SyscallFrame / Context Structures (HIGH)

**Problem**: `SyscallFrame` is defined separately in:
- `kernel/src/arch/aarch64/mod.rs` (lines 414-460)
- `kernel/src/arch/x86_64/mod.rs` (lines 371-449)

Each has its own field layout, accessor methods, and padding requirements.

**Risk**: If assembly in exception handlers doesn't match struct layout, silent corruption occurs.

**Current State**:
- AArch64: Clean `regs[31]` array + `sp`, `pc`, `pstate`, `ttbr0`
- x86_64: Complex hybrid with named registers + redundant aliases + `regs[31]` array

**Proposed Fix**:
1. Move `SyscallFrame` to HAL traits with arch-specific implementations
2. Add compile-time assertions for struct size/alignment
3. Add `#[repr(C)]` with explicit padding documentation

### 1.3 Errno Definitions (MEDIUM)

**Problem**: Error codes defined in 3 places:
- `kernel/src/syscall/mod.rs` (errno module)
- `kernel/src/syscall/mod.rs` (errno_file module - duplicate!)
- `userspace/libsyscall/src/errno.rs`

**Risk**: Different error semantics between kernel and userspace.

**Proposed Fix**: Consolidate into `crates/abi/src/errno.rs`

### 1.4 HAL Global State (HIGH)

**Problem**: HAL uses `static mut` globals without proper synchronization:

```rust
// crates/hal/src/x86_64/mmu.rs:66
pub static mut PHYS_OFFSET: usize = 0xFFFF800000000000;
pub static mut KERNEL_PHYS_BASE: usize = 0x200000;

// crates/hal/src/aarch64/gic.rs:147
static mut HANDLERS: [Option<&'static dyn InterruptHandler>; MAX_HANDLERS] = [None; MAX_HANDLERS];
```

**Risk**: Race conditions if accessed from multiple cores or interrupt contexts.

**Proposed Fix**:
1. Use `AtomicUsize` for single-value globals
2. Use `IrqSafeLock` for handler arrays
3. Document single-core assumptions with `// SAFETY:` comments

### 1.5 Memory Management Coupling (HIGH)

**Problem**: MMU code tightly coupled to boot sequence:
- `EARLY_ALLOCATOR` vs `PAGE_ALLOCATOR_PTR` switching
- `phys_to_virt()` depends on runtime `PHYS_OFFSET` being set
- Functions fail silently if called before init

**Current Pattern** (fragile):
```rust
let alloc_fn = || unsafe {
    if let Some(alloc) = PAGE_ALLOCATOR_PTR {
        alloc.alloc_page()
    } else {
        EARLY_ALLOCATOR.alloc_page()
    }
};
```

**Proposed Fix**: 
1. Use type-state pattern to encode initialization phases
2. Create `BootPhase<Early>` / `BootPhase<Runtime>` types
3. Make `phys_to_virt` const where possible or require explicit init proof

### 1.6 Interrupt Handler Registration (MEDIUM)

**Problem**: Different patterns for AArch64 vs x86_64:
- AArch64 GIC: `unsafe static mut HANDLERS` array with index mapping
- x86_64 APIC: `Mutex<[Option<...>; 256]>` with vector-based indexing

**Risk**: Adding new IRQ types requires understanding both patterns.

**Proposed Fix**: Standardize on trait-based registration through `InterruptController` trait (already exists, needs enforcement)

---

## 2. Proposed Maturation Layers

### Layer 0: ABI Crate (New)
```
crates/abi/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── syscall.rs      # SyscallNumber enum (shared)
    ├── errno.rs        # Error codes
    ├── stat.rs         # Arch-specific stat layouts
    ├── termios.rs      # Terminal structures
    └── flags.rs        # O_*, PROT_*, MAP_* constants
```

**Benefits**:
- Single source of truth for kernel/userspace ABI
- Compile-time verification that both sides agree
- Documentation lives with code

### Layer 1: HAL Stabilization
- Add `#[stable_api]` marker attribute (documentation)
- Add version assertions for breaking changes
- Convert `static mut` to proper synchronization primitives

### Layer 2: Syscall Dispatch Refactor
- Generate dispatch table from ABI crate
- Add syscall tracing infrastructure
- Add argument validation layer

---

## 3. Implementation Priority

| Priority | Issue | Effort | Impact |
|----------|-------|--------|--------|
| P0 | Create `crates/abi` for syscall numbers | 2h | HIGH - Prevents ABI drift |
| P0 | Add compile-time size assertions to SyscallFrame | 1h | HIGH - Catches layout bugs |
| P1 | Consolidate errno definitions | 1h | MEDIUM - Prevents confusion |
| P1 | Fix `static mut` in HAL | 3h | HIGH - Race condition prevention |
| P2 | Type-state for boot phases | 4h | MEDIUM - Better init ordering |
| P2 | Standardize IRQ registration | 2h | MEDIUM - Cleaner HAL |

---

## 4. Migration Strategy

### Phase 1: Non-Breaking Additions (This Sprint)
1. Create `crates/abi` with types
2. Re-export from existing locations (backward compatible)
3. Add deprecation warnings to old locations

### Phase 2: Gradual Migration
1. Update kernel to import from `los_abi`
2. Update `libsyscall` to import from `los_abi`
3. Remove old definitions

### Phase 3: Enforcement
1. CI check that syscall numbers match
2. Add behavior tests for ABI compatibility
3. Document stable vs unstable APIs

---

## 5. Immediate Quick Wins

These can be done **today** without architectural changes:

1. **Add size assertions** to `SyscallFrame`:
```rust
const _: () = assert!(core::mem::size_of::<SyscallFrame>() == EXPECTED_SIZE);
```

2. **Add errno consistency test**:
```rust
#[test]
fn test_errno_consistency() {
    assert_eq!(kernel_errno::ENOENT, userspace_errno::ENOENT);
}
```

3. **Document ABI version** in `docs/specs/userspace-abi.md`

---

## 6. Open Questions for USER

1. **Scope**: Should we pursue full Linux ABI compatibility or define a stable LevitateOS ABI?
2. **Custom Syscalls**: Keep Spawn/SpawnArgs (1000+) or migrate to clone/execve?
3. **Breaking Changes**: What's the acceptable disruption window for migration?

---

## Appendix: File Reference

### Syscall Definition Locations
- `@/home/vince/Projects/LevitateOS/kernel/src/arch/aarch64/mod.rs:17-139` (AArch64 SyscallNumber)
- `@/home/vince/Projects/LevitateOS/kernel/src/arch/x86_64/mod.rs:30-141` (x86_64 SyscallNumber)
- `@/home/vince/Projects/LevitateOS/userspace/libsyscall/src/sysno.rs:1-59` (Userspace)
- `@/home/vince/Projects/LevitateOS/kernel/src/syscall/mod.rs:35-231` (Dispatch)

### HAL Static Globals
- `@/home/vince/Projects/LevitateOS/crates/hal/src/x86_64/mmu.rs:66-71` (PHYS_OFFSET, KERNEL_PHYS_BASE)
- `@/home/vince/Projects/LevitateOS/crates/hal/src/aarch64/gic.rs:147` (HANDLERS)
- `@/home/vince/Projects/LevitateOS/crates/hal/src/x86_64/mmu.rs:370` (PAGE_ALLOCATOR_PTR)
