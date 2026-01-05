# Phase 1 — Understanding and Scoping

**TEAM_134** | Migrate Allocators to intrusive-collections

## Bug Summary

**Issue:** The buddy allocator and slab allocator use manual `NonNull<T>` linked list operations with ~8-10 unsafe blocks that could be eliminated using the `intrusive-collections` crate.

**Severity:** Medium (code hygiene / maintainability)

**Impact:**
- Manual pointer manipulation is error-prone
- Each unsafe block requires careful SAFETY documentation
- Violates Rule 5: "Wrap unsafe in safe, idiomatic abstractions"

## Reproduction Status

Not a runtime bug — this is a code quality improvement identified during the reduce-unsafe-code refactor (TEAM_133).

**TEAM_133 Findings:**
- Attempted migration failed due to const initialization issues
- `LinkedList::new()` with `UnsafeRef` adapters is not const-compatible
- Pages live in static memory map (`mem_map: &'static mut [Page]`)

## Context

### Affected Code Areas

| File | Unsafe Count | Pattern |
|------|--------------|---------|
| `levitate-hal/src/allocator/buddy.rs` | 5 | `NonNull::as_mut()`, linked list ops |
| `levitate-hal/src/allocator/slab/list.rs` | 4 | `NonNull::as_mut()`, linked list ops |
| `levitate-hal/src/allocator/page.rs` | 0 | Struct definition (next/prev fields) |

### Current Implementation

**Page struct (`page.rs`):**
```rust
pub struct Page {
    pub flags: PhysPageFlags,
    pub order: u8,
    pub refcount: u16,
    pub next: Option<NonNull<Page>>,  // Manual linked list
    pub prev: Option<NonNull<Page>>,
}
```

**BuddyAllocator (`buddy.rs`):**
```rust
pub struct BuddyAllocator {
    free_lists: [Option<NonNull<Page>>; MAX_ORDER],  // 21 free lists
    mem_map: Option<&'static mut [Page]>,
    phys_base: usize,
}
```

### Key Constraint

The allocator uses **const initialization**:
```rust
pub const fn new() -> Self {
    Self {
        free_lists: [None; MAX_ORDER],  // Must be const
        mem_map: None,
        phys_base: 0,
    }
}
```

This is required because `BuddyAllocator` is stored in a static:
```rust
static FRAME_ALLOCATOR: IrqSafeLock<FrameAllocator> = IrqSafeLock::new(FrameAllocator::new());
```

## Constraints

- **Const initialization required** — Allocator is in a static
- **No runtime behavior changes** — Must maintain same allocation semantics
- **Must work in no_std** — Kernel environment
- **Maintain performance** — Linked list ops are on hot path
- **Dependency already added** — `intrusive-collections = "0.10"` in Cargo.toml

## Open Questions

1. Does `intrusive-collections` support const initialization with `UnsafeRef`?
2. Can we use lazy initialization pattern instead?
3. Should we keep the slab allocator's custom `SlabList<T>` or migrate it too?

## Steps

- [ ] Step 1 — Research intrusive-collections const support
- [ ] Step 2 — Identify initialization pattern options
- [ ] Step 3 — Document affected code paths
