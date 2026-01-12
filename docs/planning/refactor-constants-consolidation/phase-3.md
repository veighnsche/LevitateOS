# Phase 3: Migration

## Migration Order

Migrate in dependency order (leaves first):

### Wave 1: HAL Internal (no external dependencies)
1. `hal/src/allocator/slab/page.rs` - uses PAGE_SIZE
2. `hal/src/allocator/buddy.rs` - uses PAGE_SIZE
3. `hal/src/x86_64/mem/mmu.rs` - defines PAGE_SIZE, uses alignment
4. `hal/src/x86_64/mem/frame_alloc.rs` - uses 4096 literal
5. `hal/src/aarch64/mmu/constants.rs` - defines PAGE_SIZE

### Wave 2: MM Crate (depends on HAL)
1. `mm/src/heap.rs` - defines USER_HEAP_MAX_SIZE
2. `mm/src/user/layout.rs` - defines STACK_SIZE, TLS_SIZE
3. `mm/src/user/mapping.rs` - uses 0xFFF masks
4. `mm/src/vma.rs` - uses 0xFFF for alignment checks

### Wave 3: Sched Crate (depends on MM)
1. `sched/src/user.rs` - defines USER_STACK_SIZE, USER_HEAP_MAX_SIZE
2. `sched/src/fd_table.rs` - defines MAX_FDS (already done by TEAM_459)

### Wave 4: Arch Crates (syscall numbers)
1. `arch/aarch64/src/lib.rs` - custom syscall enum
2. `arch/x86_64/src/lib.rs` - custom syscall enum, GDT re-exports

### Wave 5: Levitate Binary (top of dependency tree)
1. `levitate/src/memory.rs` - uses 0xFFF, 0x1000 mixed
2. `levitate/src/loader/elf.rs` - uses page masks
3. `levitate/src/process.rs` - uses 0xFFF
4. `levitate/src/config.rs` - references limits

### Wave 6: Syscall Crate
1. `syscall/src/mm.rs` - uses PAGE_SIZE
2. `syscall/src/sync.rs` - MAX_POLL_LOOPS (document or move)
3. `syscall/src/fs/statx.rs` - AT_EMPTY_PATH

## Call Site Inventory

### PAGE_SIZE Definitions (to be removed)
| File | Line | Action |
|------|------|--------|
| `hal/src/allocator/slab/page.rs` | 12 | Import from constants |
| `hal/src/allocator/buddy.rs` | 13 | Import from constants |
| `hal/src/x86_64/mem/mmu.rs` | 54 | Import from constants |
| `hal/src/aarch64/mmu/constants.rs` | ~10 | Import from constants |

### Magic Number 0xFFF/0x1000 (to be replaced)
| File | Lines | Pattern | Replace With |
|------|-------|---------|--------------|
| `hal/src/allocator/slab/cache.rs` | 147 | `& !0xFFF` | `page_align_down()` |
| `hal/src/aarch64/mmu/mapping.rs` | 307-363 | `& !0xFFF`, `+ 0xFFF` | helpers |
| `levitate/src/memory.rs` | 74-106 | mixed | helpers |
| `levitate/src/loader/elf.rs` | 390-523 | `& !0xFFF` | helpers |
| `levitate/src/process.rs` | 134-236 | `& 0xFFF` | `is_page_aligned()` |
| `mm/src/user/mapping.rs` | 111-290 | mixed | helpers |
| `mm/src/vma.rs` | 43-44 | `& 0xFFF == 0` | `is_page_aligned()` |
| `hal/src/x86_64/mem/frame_alloc.rs` | 34 | `+ 4096` | `+ PAGE_SIZE` |

### Stack/Heap Size Duplicates (to be unified)
| File | Constant | Action |
|------|----------|--------|
| `mm/src/user/layout.rs` | STACK_SIZE | Keep as source |
| `sched/src/user.rs` | USER_STACK_SIZE | Import from mm |
| `mm/src/heap.rs` | USER_HEAP_MAX_SIZE | Keep as source |
| `sched/src/user.rs` | USER_HEAP_MAX_SIZE | Import from mm |

### GDT Selectors (to be unified)
| File | Constants | Action |
|------|-----------|--------|
| `hal/src/x86_64/cpu/gdt.rs` | KERNEL_CODE, etc. | Keep as source |
| `hal/src/x86_64/cpu/idt.rs` | 0x08 hardcoded | Use constant |
| `arch/x86_64/src/lib.rs` | GDT_* constants | Remove, import from hal |
| `arch/x86_64/src/syscall.rs` | 0x08, 0x10 | Use constants |

### Custom Syscall Numbers (to be unified)
| File | Syscalls | Action |
|------|----------|--------|
| `arch/aarch64/src/lib.rs` | Spawn=1000, etc. | Import from shared |
| `arch/x86_64/src/lib.rs` | Spawn=1000, etc. | Import from shared |

## Rollback Plan

### If Migration Fails

1. **Revert to pre-migration commit**
   ```bash
   git revert HEAD~N..HEAD  # N = number of migration commits
   ```

2. **Keep new modules but don't remove old ones**
   - Both old and new definitions can coexist
   - Gradually migrate call sites
   - Compiler will catch conflicts

3. **Partial rollback**
   - Each wave is independent
   - Can rollback one wave without affecting others
   - Re-exports maintain compatibility

### Testing Between Waves

After each wave:
```bash
cargo xtask build kernel            # x86_64
cargo xtask --arch aarch64 build kernel  # aarch64
cargo xtask test behavior           # Full behavior test
```

If any fail, rollback that wave before proceeding.

## No Compatibility Shims (Rule 5)

**DO NOT** create wrapper functions like:
```rust
// BAD - compatibility shim
#[deprecated]
pub fn old_page_size() -> usize { PAGE_SIZE }
```

**DO** fix call sites directly:
```rust
// BEFORE
let aligned = addr & !0xFFF;

// AFTER
use los_hal::mem::constants::page_align_down;
let aligned = page_align_down(addr);
```

Let the compiler fail. Fix each call site. No backward compatibility for internal APIs.
