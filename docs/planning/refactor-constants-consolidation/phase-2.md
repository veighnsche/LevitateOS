# Phase 2: Structural Extraction

## Target Design

### New Module Layout

Create `los_hal::mem::constants` as the single source of truth for memory constants:

```
crates/kernel/lib/hal/src/
├── mem/
│   ├── mod.rs
│   └── constants.rs  ← NEW: Central constants module
├── allocator/
│   ├── buddy.rs      ← MODIFY: Import PAGE_SIZE
│   └── slab/
│       └── page.rs   ← MODIFY: Import PAGE_SIZE
├── aarch64/
│   └── mmu/
│       └── constants.rs  ← MODIFY: Import PAGE_SIZE, remove duplicate
└── x86_64/
    └── mem/
        ├── mmu.rs        ← MODIFY: Import PAGE_SIZE, remove duplicate
        └── frame_alloc.rs ← MODIFY: Use PAGE_SIZE instead of 4096
```

### Central Constants Module (`los_hal::mem::constants`)

```rust
//! Central memory constants for LevitateOS kernel
//!
//! TEAM_462: Single source of truth for PAGE_SIZE and related values.
//! All other modules must import from here.

/// Page size in bytes (4 KiB)
pub const PAGE_SIZE: usize = 4096;

/// Page size as a power of 2 (2^12 = 4096)
pub const PAGE_SHIFT: usize = 12;

/// Mask for page offset bits (lower 12 bits)
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

/// Align address down to page boundary
#[inline]
pub const fn page_align_down(addr: usize) -> usize {
    addr & !PAGE_MASK
}

/// Align address up to page boundary
#[inline]
pub const fn page_align_up(addr: usize) -> usize {
    (addr + PAGE_MASK) & !PAGE_MASK
}

/// Check if address is page-aligned
#[inline]
pub const fn is_page_aligned(addr: usize) -> bool {
    addr & PAGE_MASK == 0
}

/// Convert address to page number
#[inline]
pub const fn addr_to_page(addr: usize) -> usize {
    addr >> PAGE_SHIFT
}

/// Convert page number to address
#[inline]
pub const fn page_to_addr(page: usize) -> usize {
    page << PAGE_SHIFT
}

/// 2 MiB block size (for huge pages / L2 blocks)
pub const BLOCK_SIZE_2MB: usize = 2 * 1024 * 1024;

/// 1 GiB block size (for L1 blocks on aarch64)
pub const BLOCK_SIZE_1GB: usize = 1024 * 1024 * 1024;
```

### Process Limits Module (`los_mm::user::limits`)

Consolidate user process limits:

```rust
//! User process limits
//!
//! TEAM_462: Single source of truth for stack/heap sizes.

use los_hal::mem::constants::PAGE_SIZE;

/// Default user stack size (2 MiB)
pub const USER_STACK_SIZE: usize = 2 * 1024 * 1024;

/// Maximum user heap size (256 MiB)
pub const USER_HEAP_MAX: usize = 256 * 1024 * 1024;

/// TLS area size (one page)
pub const TLS_SIZE: usize = PAGE_SIZE;

/// Maximum file descriptors per process
pub const MAX_FDS: usize = 1024;

/// Maximum VMAs per process
pub const MAX_VMAS: usize = 65535;
```

### Custom Syscall Numbers Module

Create shared definition in `los_syscall` that both arch crates can reference:

```rust
//! Custom syscall numbers for LevitateOS extensions
//!
//! TEAM_462: Shared between aarch64 and x86_64 to prevent desync.

/// Base number for custom syscalls (well above Linux range)
pub const CUSTOM_SYSCALL_BASE: u64 = 1000;

/// Custom syscall: spawn process directly
pub const SYS_SPAWN: u64 = CUSTOM_SYSCALL_BASE + 0;

/// Custom syscall: spawn with arguments
pub const SYS_SPAWN_ARGS: u64 = CUSTOM_SYSCALL_BASE + 1;

/// Custom syscall: set foreground process
pub const SYS_SET_FOREGROUND: u64 = CUSTOM_SYSCALL_BASE + 2;

/// Custom syscall: get foreground process
pub const SYS_GET_FOREGROUND: u64 = CUSTOM_SYSCALL_BASE + 3;

/// Custom syscall: check if fd is a tty
pub const SYS_ISATTY: u64 = CUSTOM_SYSCALL_BASE + 10;
```

## Extraction Strategy

### Order of Operations

1. **Create new modules first** (additive, non-breaking)
   - Create `los_hal::mem::constants`
   - Create `los_mm::user::limits`
   - Create custom syscall module

2. **Add re-exports** (maintain compatibility temporarily)
   - Existing modules re-export from new location
   - Compiler will catch any missing imports

3. **Migrate call sites** (Phase 3)
   - One file at a time
   - Replace magic numbers with constants
   - Replace manual alignment with helper functions

4. **Remove duplicate definitions** (Phase 4)
   - Remove old constants after all call sites migrated
   - Remove re-exports

### Coexistence Period

During migration, both old and new constants exist:

```rust
// hal/src/allocator/buddy.rs (temporary)
// TEAM_462: Re-export for compatibility during migration
pub use crate::mem::constants::PAGE_SIZE;

// Old code still works:
use los_hal::allocator::buddy::PAGE_SIZE;

// New code prefers:
use los_hal::mem::constants::PAGE_SIZE;
```

## Module Organization Rules (Rule 7)

- Each module owns its constants - no deep imports like `los_hal::x86_64::cpu::gdt::KERNEL_CODE`
- Public API is `los_hal::mem::constants::PAGE_SIZE` (shallow)
- File sizes: constants.rs will be < 100 lines
- Private helper implementation, public const interface
