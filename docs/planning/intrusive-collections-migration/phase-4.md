# Phase 4 — Implementation and Tests

**TEAM_134** | Migrate Allocators to intrusive-collections

## Implementation Steps

### Step 1: Update Page Struct (Low complexity)

**File:** `levitate-hal/src/allocator/page.rs`

**Tasks:**
1. Replace `next: Option<NonNull<Page>>` and `prev: Option<NonNull<Page>>` with `link: LinkedListLink`
2. Remove `Clone, Copy` derives (LinkedListLink is not Copy)
3. Update `new()` to initialize `link: LinkedListLink::new()`
4. Update `reset()` — LinkedListLink auto-resets when removed from list
5. Run `cargo check`

**Code Changes:**
```rust
// Before
pub struct Page {
    pub flags: PhysPageFlags,
    pub order: u8,
    pub refcount: u16,
    pub next: Option<NonNull<Page>>,
    pub prev: Option<NonNull<Page>>,
}

// After
use intrusive_collections::LinkedListLink;

pub struct Page {
    pub flags: PhysPageFlags,
    pub order: u8,
    pub refcount: u16,
    pub link: LinkedListLink,
}
```

**UoW Size:** ~20 lines, 1 session

---

### Step 2: Update BuddyAllocator Structure (Medium complexity)

**File:** `levitate-hal/src/allocator/buddy.rs`

**Tasks:**
1. Add imports for intrusive-collections
2. Define `PageAdapter` using `intrusive_adapter!` macro
3. Change `free_lists` type to `[Option<LinkedList<PageAdapter>>; MAX_ORDER]`
4. Update `new()` to use `[const { None }; MAX_ORDER]`
5. Run `cargo check`

**Code Changes:**
```rust
use intrusive_collections::{intrusive_adapter, LinkedList, LinkedListLink, UnsafeRef};

intrusive_adapter!(pub PageAdapter = UnsafeRef<Page>: Page { link => LinkedListLink });

pub struct BuddyAllocator {
    free_lists: [Option<LinkedList<PageAdapter>>; MAX_ORDER],
    mem_map: Option<&'static mut [Page]>,
    phys_base: usize,
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            free_lists: [const { None }; MAX_ORDER],
            mem_map: None,
            phys_base: 0,
        }
    }
}
```

**UoW Size:** ~30 lines, 1 session

---

### Step 3: Update init() to Create LinkedLists (Low complexity)

**File:** `levitate-hal/src/allocator/buddy.rs`

**Tasks:**
1. In `init()`, initialize all 21 LinkedLists
2. Use `LinkedList::new(PageAdapter::new())` for each order

**Code Changes:**
```rust
pub unsafe fn init(&mut self, mem_map: &'static mut [Page], phys_base: usize) {
    // TEAM_134: Initialize all linked lists (lazy init pattern)
    for i in 0..MAX_ORDER {
        self.free_lists[i] = Some(LinkedList::new(PageAdapter::new()));
    }
    self.mem_map = Some(mem_map);
    self.phys_base = phys_base;
}
```

**UoW Size:** ~10 lines, 1 session

---

### Step 4: Rewrite add_to_list() (Medium complexity)

**File:** `levitate-hal/src/allocator/buddy.rs`

**Tasks:**
1. Replace manual pointer manipulation with LinkedList::push_front()
2. Use UnsafeRef::from_raw() to create the pointer wrapper
3. Remove all unsafe blocks from this function

**Code Changes:**
```rust
fn add_to_list(&mut self, order: usize, page: &'static mut Page) {
    let list = self.free_lists[order]
        .as_mut()
        .expect("TEAM_134: Allocator must be initialized");
    
    // SAFETY: page is a valid 'static reference from mem_map
    let page_ref = unsafe { UnsafeRef::from_raw(page as *mut Page) };
    list.push_front(page_ref);
}
```

**UoW Size:** ~15 lines, 1 session

---

### Step 5: Rewrite remove_from_list() (Medium complexity)

**File:** `levitate-hal/src/allocator/buddy.rs`

**Tasks:**
1. Use LinkedList cursor API to find and remove specific page
2. Replace manual prev/next manipulation with cursor.remove()

**Code Changes:**
```rust
fn remove_from_list(&mut self, order: usize, page: &mut Page) {
    let list = self.free_lists[order]
        .as_mut()
        .expect("TEAM_134: Allocator must be initialized");
    
    // Use cursor to find and remove the page
    let mut cursor = list.front_mut();
    while !cursor.is_null() {
        // SAFETY: cursor points to valid Page from mem_map
        let current = unsafe { cursor.get().unwrap() };
        if core::ptr::eq(current as *const Page, page as *const Page) {
            cursor.remove();
            return;
        }
        cursor.move_next();
    }
    // Page not found - this should not happen if invariants are maintained
    panic!("TEAM_134: Page not found in free list - corrupted allocator state");
}
```

**UoW Size:** ~20 lines, 1 session

---

### Step 6: Update alloc() to use LinkedList API (Medium complexity)

**File:** `levitate-hal/src/allocator/buddy.rs`

**Tasks:**
1. Replace `if let Some(mut page_ptr) = self.free_lists[i]` with LinkedList::is_empty() check
2. Use pop_front() instead of manual head removal
3. Update to work with UnsafeRef

**Code Changes:**
```rust
pub fn alloc(&mut self, order: usize) -> Option<usize> {
    if order >= MAX_ORDER {
        return None;
    }

    for i in order..MAX_ORDER {
        let list = self.free_lists[i].as_mut()?;
        if !list.is_empty() {
            // Pop the first free page
            let page_ref = list.pop_front().unwrap();
            // SAFETY: UnsafeRef from our mem_map
            let page: &'static mut Page = unsafe { 
                &mut *(UnsafeRef::into_raw(page_ref) as *mut Page)
            };
            
            // Split if larger than needed
            for j in (order..i).rev() {
                let buddy_pa = self.page_to_pa(page) + (1 << j) * PAGE_SIZE;
                let buddy_page = self.pa_to_page_mut(buddy_pa)
                    .expect("TEAM_134: Buddy page must exist");
                buddy_page.reset();
                buddy_page.order = j as u8;
                buddy_page.mark_free();
                self.add_to_list(j, buddy_page);
            }

            page.mark_allocated();
            page.order = order as u8;
            return Some(self.page_to_pa(page));
        }
    }
    None
}
```

**UoW Size:** ~40 lines, 1 session

---

### Step 7: Run Tests and Verify (Low complexity)

**Tasks:**
1. Run `cargo xtask test unit` — all allocator tests must pass
2. Run unsafe audit: `grep -rn "unsafe {" levitate-hal/src/allocator/buddy.rs | wc -l`
3. Verify reduction from ~5 unsafe blocks to ~2 (UnsafeRef creation only)

**Pass Criteria:**
- All 5 buddy allocator tests pass
- Unsafe count reduced
- No new panics or undefined behavior

---

## Step Dependencies

```
Step 1 (Page) ──> Step 2 (struct) ──> Step 3 (init) ──> Step 4-6 (list ops) ──> Step 7 (tests)
```

Steps 4, 5, 6 can be done in parallel after Steps 1-3.

---

## Test Plan

### Required Tests (Must Pass)

| Test | Description |
|------|-------------|
| `test_alloc_order_0` | Single page allocation |
| `test_alloc_large` | Multi-page allocation |
| `test_splitting` | Block splitting on alloc |
| `test_coalescing` | Block merging on free |
| `test_alloc_unaligned_range` | Non-power-of-two ranges |

### Unsafe Audit

After each step, run:
```bash
grep -rn "unsafe {" levitate-hal/src/allocator/buddy.rs --include="*.rs" | wc -l
```

**Target:** Reduce from 5 to 2 (UnsafeRef::from_raw only)
