# Phase 4: Cleanup

## Dead Code Removal (Rule 6)

After all call sites are migrated, remove:

### Duplicate PAGE_SIZE Definitions
| File | Line | Remove |
|------|------|--------|
| `hal/src/allocator/slab/page.rs` | 12 | `pub const PAGE_SIZE: usize = 4096;` |
| `hal/src/allocator/buddy.rs` | 13 | `pub const PAGE_SIZE: usize = 4096;` |
| `hal/src/x86_64/mem/mmu.rs` | 54 | `pub const PAGE_SIZE: usize = 4096;` |
| `hal/src/aarch64/mmu/constants.rs` | ~10 | Duplicate PAGE_SIZE |

### Duplicate Process Limits
| File | Line | Remove |
|------|------|--------|
| `sched/src/user.rs` | 51 | `pub const USER_STACK_SIZE` (keep import) |
| `sched/src/user.rs` | 66 | `pub const USER_HEAP_MAX_SIZE` (keep import) |

### Duplicate GDT Constants
| File | Line | Remove |
|------|------|--------|
| `arch/x86_64/src/lib.rs` | 23-26 | GDT_KERNEL_CODE, etc. (import from hal) |

### Temporary Re-exports
After all call sites migrated, remove any temporary re-exports added during Phase 3.

## Remove Temporary Adapters

None planned - per Rule 5, we don't create compatibility shims.

If any were created during development (mistakes happen), remove them here.

## Tighten Encapsulation

### Make Private What Should Be Private

After migration, audit visibility:

| Module | Should Be | Action |
|--------|-----------|--------|
| `hal::mem::constants::PAGE_SIZE` | `pub` | Keep - API |
| `hal::mem::constants::PAGE_MASK` | `pub` | Keep - API |
| Internal alignment helpers | `pub(crate)` | Consider if only used internally |

### Module Boundaries

Each crate should only export its public API:

```rust
// hal/src/lib.rs
pub mod mem {
    pub mod constants;  // Public - used by other crates
}

// Internal modules not re-exported
mod internal_helpers;
```

## File Size Check

Target: <1000 lines preferred, <500 ideal

| File | Expected Size | Status |
|------|---------------|--------|
| `hal/src/mem/constants.rs` | ~50 lines | NEW - OK |
| `mm/src/user/limits.rs` | ~30 lines | NEW - OK |
| `hal/src/allocator/buddy.rs` | Check after | Should shrink slightly |
| `hal/src/x86_64/mem/mmu.rs` | Check after | Should shrink slightly |

### Large Files to Monitor

Files that were already large and should not grow:

| File | Current Size | Watch For |
|------|--------------|-----------|
| `levitate/src/init.rs` | ~400 lines | No growth |
| `syscall/src/lib.rs` | ~500 lines | No growth |
| `mm/src/user/mapping.rs` | ~300 lines | Should shrink with helpers |

## Final Verification

After cleanup:

```bash
# No dead code warnings
cargo build --release 2>&1 | grep -i "dead_code\|never used"

# No unused imports
cargo build --release 2>&1 | grep -i "unused import"

# All tests still pass
cargo xtask test

# Both architectures build
cargo xtask build kernel
cargo xtask --arch aarch64 build kernel
```

## Cleanup Checklist

- [ ] All duplicate PAGE_SIZE definitions removed
- [ ] All duplicate process limits removed
- [ ] All duplicate GDT constants removed
- [ ] All temporary re-exports removed
- [ ] All magic numbers replaced with constants
- [ ] No compatibility shims exist
- [ ] All files under 1000 lines
- [ ] No dead code warnings
- [ ] Both architectures build
- [ ] All tests pass
