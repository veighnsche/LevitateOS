# Phase 4: Implementation and Tests

**Bug**: VirtQueue DMA Memory Allocation
**TEAM_105**: Following /make-a-bugfix-plan workflow

---

## 4.1 Implementation Steps

### Step 1: Add Alignment to Descriptor (1 UoW)

**File**: `levitate-virtio/src/queue.rs`
**Lines**: 38-46

**Change**:
```rust
// FROM:
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Descriptor {

// TO:
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Default)]
pub struct Descriptor {
```

**Verification**:
```bash
cargo build -p levitate-virtio
```

---

### Step 2: Add Alignment to VirtQueue (1 UoW)

**File**: `levitate-virtio/src/queue.rs`
**Lines**: 65-74

**Change**:
```rust
// FROM:
pub struct VirtQueue<const SIZE: usize> {

// TO:
#[repr(C, align(16))]
pub struct VirtQueue<const SIZE: usize> {
```

**Verification**:
```bash
cargo build -p levitate-virtio
```

---

### Step 3: Add Alignment Assertion (1 UoW)

**File**: `levitate-virtio/src/queue.rs`
**Location**: In `VirtQueue::new()` function

**Add** (at start of function):
```rust
// Compile-time alignment check
const _: () = assert!(core::mem::align_of::<Descriptor>() >= 16);
```

**Verification**:
```bash
cargo build -p levitate-virtio
```

---

### Step 4: Update VirtioGpu Struct for DMA (2 UoW)

**File**: `levitate-drivers-gpu/src/device.rs`

**Change struct definition** (around line 25):
```rust
// FROM:
pub struct VirtioGpu<H: VirtioHal> {
    transport: MmioTransport,
    control_queue: Box<VirtQueue<QUEUE_SIZE>>,
    ...
}

// TO:
pub struct VirtioGpu<H: VirtioHal> {
    transport: MmioTransport,
    control_queue_ptr: NonNull<VirtQueue<QUEUE_SIZE>>,
    control_queue_paddr: u64,
    control_queue_pages: usize,
    ...
}
```

**Add import**:
```rust
use core::ptr::NonNull;
```

---

### Step 5: Update VirtioGpu::new() for DMA Allocation (3 UoW)

**File**: `levitate-drivers-gpu/src/device.rs`
**Location**: `new()` function, around line 130

**Replace Box allocation with DMA allocation**:
```rust
// FROM:
let mut control_queue = Box::new(VirtQueue::new());
let (desc_addr, avail_addr, used_addr) = control_queue.addresses();

// TO:
// Allocate queue via DMA
let queue_size = core::mem::size_of::<VirtQueue<QUEUE_SIZE>>();
let queue_pages = (queue_size + 4095) / 4096;
let (queue_paddr, queue_vptr) = H::dma_alloc(queue_pages, BufferDirection::Both);

// Initialize queue in DMA memory
let control_queue_ptr = unsafe {
    let ptr = queue_vptr.as_ptr() as *mut VirtQueue<QUEUE_SIZE>;
    ptr.write(VirtQueue::new());
    NonNull::new_unchecked(ptr)
};

// Get addresses (these are virtual, will be translated)
let control_queue = unsafe { control_queue_ptr.as_mut() };
let (desc_addr, avail_addr, used_addr) = control_queue.addresses();
```

**Update struct initialization**:
```rust
// Store DMA info for cleanup
control_queue_ptr,
control_queue_paddr: queue_paddr,
control_queue_pages: queue_pages,
```

---

### Step 6: Update control_queue accessor (1 UoW)

**File**: `levitate-drivers-gpu/src/device.rs`

Update all `self.control_queue` usages to dereference the pointer:
```rust
// FROM:
self.control_queue.add_buffer(...)
self.control_queue.has_used()
self.control_queue.pop_used()

// TO:
unsafe { self.control_queue_ptr.as_mut() }.add_buffer(...)
unsafe { self.control_queue_ptr.as_ref() }.has_used()
unsafe { self.control_queue_ptr.as_mut() }.pop_used()
```

Or add a helper method:
```rust
fn control_queue(&mut self) -> &mut VirtQueue<QUEUE_SIZE> {
    unsafe { self.control_queue_ptr.as_mut() }
}
```

---

### Step 7: Implement Drop for DMA Cleanup (1 UoW)

**File**: `levitate-drivers-gpu/src/device.rs`

**Add or update Drop impl**:
```rust
impl<H: VirtioHal> Drop for VirtioGpu<H> {
    fn drop(&mut self) {
        // Reset device first
        self.transport.reset();
        
        // Deallocate DMA queue memory
        unsafe {
            H::dma_dealloc(
                self.control_queue_paddr,
                self.control_queue_ptr.cast(),
                self.control_queue_pages
            );
        }
        
        // Deallocate framebuffer (existing code)
        if self.fb_pages > 0 {
            unsafe {
                H::dma_dealloc(self.fb_paddr, self.fb_ptr, self.fb_pages);
            }
        }
    }
}
```

---

## 4.2 Testing

### Run Full Test Suite
```bash
cargo xtask test
```

**Expected**: All 22 tests pass

### Manual Verification
```bash
./run.sh
```

**Expected**: 
- GPU initializes without timeout
- Terminal displays correctly
- "READY" indicator visible

---

## 4.3 Implementation Order

1. Steps 1-3: Alignment fixes (low risk, can be done first)
2. Step 4-7: DMA allocation changes (higher risk, do together)

**Recommend**: Complete alignment fixes first, verify build, then do DMA changes.

---

## Phase 4 Complete

Implementation steps documented. Proceed to Phase 5 for cleanup and handoff.
