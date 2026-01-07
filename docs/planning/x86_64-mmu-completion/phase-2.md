# Phase 2: Design — x86_64 MMU Completion

## Proposed Solutions

### TODO 1: x86_64 Linker Script with Segment Symbols

**Solution:** Create a dedicated x86_64 linker script that defines all required segment symbols.

**Design:**
```ld
/* x86_64 kernel linker script */
OUTPUT_FORMAT("elf64-x86-64")
ENTRY(kernel_main)

KERNEL_VIRT_BASE = 0xFFFFFFFF80000000;

SECTIONS {
    . = 1M;  /* Multiboot2 loads at 1MB */
    
    __kernel_phys_start = .;
    
    .text : {
        __text_start = .;
        *(.text .text.*)
        __text_end = .;
    }
    
    .rodata : {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    }
    
    .data : {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }
    
    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }
    
    __heap_start = .;
    . += 4M;  /* 4MB heap */
    __heap_end = .;
    
    _kernel_end = .;
}
```

**File Location:** `kernel/src/arch/x86_64/linker.ld`

**Build Integration:** Update `xtask/src/build.rs` to use arch-specific linker script.

---

### TODO 2: Heap Initialization Order in kernel_main

**Solution:** Initialize the heap early in `kernel_main`, before any code that requires allocations.

**Design:**
```rust
pub extern "C" fn kernel_main(multiboot_magic: usize, multiboot_info: usize) -> ! {
    // 1. Initialize serial first (no heap needed)
    los_hal::x86_64::serial::init();
    
    // 2. Initialize heap BEFORE anything that allocates
    init_heap();
    
    // 3. Now safe to use println! and other allocating code
    los_hal::x86_64::init();
    
    // 4. Parse multiboot2
    if multiboot_magic == MULTIBOOT2_BOOTLOADER_MAGIC {
        unsafe { multiboot2::init(multiboot_info); }
    }
    
    // 5. Expand PMO and init buddy allocator
    // ...
}

fn init_heap() {
    unsafe extern "C" {
        static __heap_start: u8;
        static __heap_end: u8;
    }
    
    let heap_start = unsafe { &__heap_start as *const _ as usize };
    let heap_end = unsafe { &__heap_end as *const _ as usize };
    let heap_size = heap_end - heap_start;
    
    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
}
```

**Key Insight:** Serial output doesn't need heap, so initialize that first for early debugging.

---

### TODO 3: Fix test_irq_safe_lock_behavior Crash

**Root Cause Analysis:**

The test calls `interrupts::disable()` and `interrupts::restore()`. Looking at the mock implementation:

```rust
// In crates/hal/src/interrupts.rs
#[cfg(feature = "std")]
pub fn disable() -> u64 {
    // Mock implementation for tests
    INTERRUPT_STATE.fetch_and(!1, Ordering::SeqCst)
}
```

**Hypothesis:** The `INTERRUPT_STATE` static or related code may have undefined behavior in the test harness.

**Solution Options:**

1. **Option A:** Fix the mock implementation to be fully safe
2. **Option B:** Skip the test on `std` feature if it's hardware-dependent
3. **Option C:** Refactor to use a trait-based approach for testability

**Recommended:** Option A — Fix the mock to be safe.

**Design:**
```rust
#[cfg(all(test, feature = "std"))]
mod mock_interrupts {
    use std::sync::atomic::{AtomicU64, Ordering};
    
    static INTERRUPT_STATE: AtomicU64 = AtomicU64::new(1); // 1 = enabled
    
    pub fn disable() -> u64 {
        INTERRUPT_STATE.swap(0, Ordering::SeqCst)
    }
    
    pub fn restore(state: u64) {
        INTERRUPT_STATE.store(state, Ordering::SeqCst);
    }
    
    pub fn is_enabled() -> bool {
        INTERRUPT_STATE.load(Ordering::SeqCst) != 0
    }
}
```

---

## Behavioral Decisions

### Q1: Should x86_64 use the same linker script structure as aarch64?
**Decision:** Yes, use parallel structure for maintainability.

### Q2: What heap size should x86_64 use?
**Decision:** 4MB initial heap (same as aarch64), can be adjusted later.

### Q3: Should `init_kernel_mappings_refined` be called if linker symbols are missing?
**Decision:** Fall back to `init_kernel_mappings` (coarse permissions) if symbols not found.
This prevents hard crashes on misconfigured builds.

### Q4: Should the test fix be x86_64-specific or general?
**Decision:** General fix in `crates/hal/src/interrupts.rs` — benefits all architectures.

---

## Open Questions

None — all behavioral decisions made above.

---

## Design Alternatives Considered

### Linker Script
- **Alternative:** Modify root `linker.ld` with conditional sections
- **Rejected:** Too complex, arch-specific scripts are cleaner

### Heap Init
- **Alternative:** Use a static heap array instead of linker symbols
- **Rejected:** Wastes memory and doesn't integrate with memory map

### Test Fix
- **Alternative:** `#[ignore]` the failing test
- **Rejected:** Hides the bug rather than fixing it

---

## Phase 2 Exit Criteria

- [x] Solution designed for each TODO
- [x] Behavioral decisions documented
- [x] No open questions
- [ ] → Proceed to Phase 3: Implementation
