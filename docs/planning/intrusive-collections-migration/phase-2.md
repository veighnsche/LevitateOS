# Phase 2 — Root Cause Analysis

**TEAM_134** | Migrate Allocators to intrusive-collections

## Hypotheses List

### H1: LinkedList::new() is not const (CONFIRMED)

**Evidence:** TEAM_133 encountered this error:
```
cannot call non-const associated function `LinkedList::new` in constants
```

**Confidence:** HIGH — This is the primary blocker.

**Root Cause:** The `intrusive_adapter!` macro generates an adapter struct with a `new()` function. When using `UnsafeRef<T>`, the adapter's `new()` is not marked `const`, so `LinkedList::new(adapter)` cannot be called in const context.

### H2: UnsafeRef requires special handling

**Evidence:** `UnsafeRef<T>` is designed for nodes in external storage (like our static `mem_map`). However, it may have different const requirements than `Box<T>` or `Arc<T>`.

**Confidence:** MEDIUM — Need to verify with crate docs.

### H3: Lazy initialization can work around const limitation

**Evidence:** Other kernel allocators (e.g., Linux, Redox) use lazy initialization patterns for intrusive data structures.

**Confidence:** HIGH — This is a known pattern.

## Key Code Areas

### BuddyAllocator::new() — The Const Constraint

```rust
// levitate-hal/src/allocator/buddy.rs
impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            free_lists: [None; MAX_ORDER],  // ← Must remain const
            mem_map: None,
            phys_base: 0,
        }
    }
}
```

### Static Allocation Site

```rust
// levitate-hal/src/allocator/memory.rs (or similar)
static FRAME_ALLOCATOR: IrqSafeLock<FrameAllocator> = IrqSafeLock::new(FrameAllocator::new());
```

### Linked List Operations (Current)

```rust
fn add_to_list(&mut self, order: usize, page: &'static mut Page) {
    page.next = self.free_lists[order];
    page.prev = None;
    if let Some(mut next_ptr) = self.free_lists[order] {
        unsafe { next_ptr.as_mut().prev = Some(NonNull::from(&mut *page)) };
    }
    self.free_lists[order] = Some(NonNull::from(&mut *page));
}
```

## Investigation Strategy

1. **Check intrusive-collections for const support**
   - Look for `const fn new()` on adapters
   - Check if `LinkedList` has const constructors
   - Review UnsafeRef documentation

2. **Research lazy initialization patterns**
   - `MaybeUninit` + manual init
   - `OnceCell` / `OnceLock` patterns
   - Initialize in `init()` instead of `new()`

3. **Consider alternative designs**
   - Keep `Option<NonNull<Page>>` but add safe wrapper
   - Use `LinkedList` only after init, store as `Option<LinkedList>`

## Root Cause Summary

The migration is blocked because:

1. **`intrusive_adapter!` with `UnsafeRef` does not generate a const `new()`**
2. **`BuddyAllocator` requires const initialization for static storage**
3. **The 21-element `free_lists` array needs const initialization**

The fix requires either:
- A way to const-initialize `LinkedList<PageAdapter>`
- A lazy initialization pattern that defers list creation to `init()`
