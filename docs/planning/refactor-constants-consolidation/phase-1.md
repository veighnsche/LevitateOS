# Phase 1: Discovery & Safeguards

## Refactor Summary

### What
Consolidate duplicated constants and hardcoded magic numbers throughout the kernel
into a single source of truth.

### Why
The TEAM_461 audit found 18 issues where the same constant is defined in multiple
places or magic numbers are scattered throughout the codebase. This creates:

1. **Desync bugs**: If one definition is updated and others aren't, subtle bugs appear
2. **Maintenance burden**: Finding all instances of "4096" or "0xFFF" is error-prone
3. **Code review difficulty**: Hard to verify consistency across files

### Pain Points
- PAGE_SIZE defined in 4+ different files
- Page alignment masks (0xFFF) used 25+ times as raw literals
- Stack/heap sizes duplicated between mm and sched crates
- GDT selectors duplicated between hal and arch crates
- Custom syscall numbers duplicated between aarch64 and x86_64

## Success Criteria

### Before
```rust
// hal/src/allocator/buddy.rs
pub const PAGE_SIZE: usize = 4096;

// hal/src/x86_64/mem/mmu.rs
pub const PAGE_SIZE: usize = 4096;

// mm/src/user/mapping.rs
if addr & 0xFFF != 0 { ... }  // Magic number

// sched/src/user.rs
pub const USER_STACK_SIZE: usize = 2 * 1024 * 1024;

// mm/src/user/layout.rs
pub const STACK_SIZE: usize = 2 * 1024 * 1024;  // Same value, different name!
```

### After
```rust
// Single source of truth in los_hal::mem::constants
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

#[inline]
pub const fn page_align_down(addr: usize) -> usize {
    addr & !PAGE_MASK
}

#[inline]
pub const fn page_align_up(addr: usize) -> usize {
    (addr + PAGE_MASK) & !PAGE_MASK
}

// All other files import:
use los_hal::mem::constants::{PAGE_SIZE, page_align_down, page_align_up};
```

## Behavioral Contracts (APIs that must not change)

| API | Location | Contract |
|-----|----------|----------|
| `PAGE_SIZE` | everywhere | Must equal 4096 |
| Page alignment | mm, loader | Must align to 4096 boundaries |
| GDT selectors | x86_64 | 0x08/0x10/0x1b/0x23 (kernel code/data, user code/data) |
| User stack size | sched, mm | 2 MiB |
| User heap max | sched, mm | 256 MiB |

## Golden/Regression Tests to Lock In

1. **Behavior tests**: `cargo xtask test behavior`
   - Verifies boot sequence, syscalls, shell interaction
   - Must pass before and after refactor

2. **Unit tests**: `cargo xtask test unit`
   - All existing tests must pass
   - Add new tests for alignment helpers

3. **Build tests**: Both architectures must compile
   - `cargo xtask build kernel`
   - `cargo xtask --arch aarch64 build kernel`

## Current Architecture Notes

### Dependency Graph (relevant crates)
```
levitate (binary)
├── los_hal (hardware abstraction)
│   ├── allocator/ (buddy, slab)
│   ├── aarch64/ (mmu, exceptions)
│   └── x86_64/ (mmu, gdt, idt)
├── los_mm (memory management)
│   ├── heap.rs
│   ├── user/layout.rs
│   └── user/mapping.rs
├── los_sched (scheduler)
│   └── user.rs
├── los_arch_aarch64 (syscall numbers)
└── los_arch_x86_64 (syscall numbers, gdt constants)
```

### Key Couplings
1. `los_hal` defines low-level PAGE_SIZE but so do other crates
2. `los_mm` and `los_sched` both define stack/heap sizes
3. `los_arch_*` crates duplicate custom syscall numbers
4. `los_hal::x86_64::cpu::gdt` and `los_arch_x86_64` both define GDT selectors

## Constraints

1. **No circular dependencies**: Cannot have hal depend on mm or vice versa
2. **Architecture independence**: Constants module must work for both aarch64 and x86_64
3. **Compile-time evaluation**: All constants must be `const` for static allocation
4. **Breaking changes OK**: Per Rule 5, we fix call sites rather than add compatibility shims
