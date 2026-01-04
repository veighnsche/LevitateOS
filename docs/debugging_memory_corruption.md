# Deep Dive: Heap Overlap Memory Corruption

**Team:** TEAM_074
**Date:** 2026-01-04
**Status:** UNRESOLVED - CRITICAL BLOCKER
**Severity:** HIGH (Kernel Panic / Data Abort)

---

## 1. Executive Summary

The kernel is currently crashing with a `Synchronous Exception (Data Abort)` during Userspace Boot. This is **NOT** a userspace issue. It is a fundamental memory management corruption caused by an overlap between the **Kernel Heap** and the **Physical Frame Allocator's Metadata (`mem_map`)**.

**The Bug:** The Frame Allocator places its critical metadata structures (`mem_map`) at physical address `0x40200000`. This address falls directly inside the region reserved for the Kernel Heap (`0x400c0000` to `0x41f00000`).

**The Effect:** When the kernel initializes the heap, it zeroes or writes to this memory effectively destroying the Frame Allocator's free list. The next time `alloc_page()` is called, the allocator follows a corrupted pointer and crashes the kernel.

---

## 2. Detailed System Trace

### 2.1. The Crash
The crash occurs immediately when `spawn_from_elf` calls `create_user_page_table()`, which requests a new page from the Frame Allocator.

```text
[BOOT] TEAM_073: Hijacking boot for Userspace Demo...
[SPAWN] Looking for 'hello' in initramfs...
[SPAWN] Found 'hello' (66576 bytes)
...
[MMU] Creating user page table...
[BUDDY] alloc order 0
[BUDDY] Found block at order 0 ptr 0xffff8000402c0198
*** KERNEL EXCEPTION: Synchronous ***
ESR: 0x0000000096000044  (Data Abort, Translation Fault)
ELR: 0xffff80004009676c  (Instruction causing fault)
FAR: 0xffff8000402c0198  (Faulting Address - same as ptr above)
```

### 2.2. Memory Layout Analysis

We added debug prints to `kernel/src/memory/mod.rs` and the output confirmed the overlap:

**1. Reserved Kernel Range (as reported by `_kernel_start` / `_kernel_end`):**
```text
[MEMORY] Reserved Kernel: 0x400801e8 - 0x400c6b20
```
> **CRITICAL:** This range is only ~280KB. It covers the `.text`, `.data`, and `.bss` sections **BUT NOT THE HEAP**.

**2. Physical RAM Discovery:**
The `memory::init()` function discovers available RAM from the DTB. It then tries to find a "safe" place to put the `mem_map` (an array of `Page` structs, one for every 4KB of RAM).

**3. `mem_map` Placement:**
```text
[MEMORY] Allocated mem_map at PA 0x40200000 (Size 3MB)
```
> The allocator logic looked for a hole. It saw `0x400c6b20` as the end of the used kernel memory. It aligned up to the next 2MB boundary (`0x40200000`) and placed the `mem_map` there.

**4. The Hidden Heap:**
The Linker Script (`linker.ld`) defines the heap as starting after `.bss` and extending to `0x41F00000`.

```text
Heap Start: ~0x400c6b20
Heap End:    0x41F00000
```

**5. The Collision:**
- **Heap Range:** `0x400c6b20` -> `0x41F00000`
- **mem_map Range:** `0x40200000` -> `0x40500000`

The `mem_map` is completely swallowed by the Heap. As soon as the Heap Allocator (Buddy/Slab) initializes or performs allocations, it overwrites the `mem_map`.

---

## 3. Code Analysis & The "Gotcha"

Why did `_kernel_end` not include the heap?

### 3.1. The Linker Script (`linker.ld`)

**Original Code:**
```ld
    . = ALIGN(16);
    __heap_start = .;
    
    /* Defines a symbol with a value, DOES NOT ADVANCE LOCATION COUNTER (.) */
    __heap_end = _kernel_virt_base + 0x41F00000; 
    
    /* _kernel_end gets the value of ., which is still at __heap_start! */
    _kernel_end = .; 
```

Technically, `__heap_end = val;` is just a variable assignment in LD scripts. It implies nothing about memory reservation unless `.` is moved.

### 3.2. The Failed Fix
We attempted to fix this by explicitly moving the location counter:

```ld
    . = ALIGN(16);
    __heap_start = .;
    
    /* Move location counter to the end of the heap */
    . = _kernel_virt_base + 0x41F00000;
    
    /* These now point to the new location */
    __heap_end = .;
    _kernel_end = .;
```

**Why it likely failed:**
Even after applying this change and running `cargo clean`, the `Reserved Kernel` print still showed the old address (`0x400c...`).
This implies:
1.  **Zombie Build:** Identify where `linker.ld` is being picked up. Is `build.rs` triggering a rebuild correctly?
2.  **Symbol Confusion:** Are we reading `_kernel_end` correctly in Rust?
    ```rust
    unsafe extern "C" {
        static _kernel_end: u8;
    }
    let kernel_end = unsafe { &_kernel_end as *const _ as usize };
    ```
    If the symbol value changed in the ELF, this should reflect it.

---

## 4. Remediation Plan (For Next Team)

You have two paths to fix this. Path A is cleaner, Path B is more explicit/robust.

### Path A: Fix the Linker Script (Preferred if possible)

1.  **Modify `linker.ld`** to correctly advance `.` so `_kernel_end` includes the heap.
2.  **Force Rebuild:** Ensure the linker script change actually takes effect.
3.  **Verify:** Check the `[MEMORY] Reserved Kernel` log line. It MUST be `~0x41F00000`.

### Path B: Explicit Reservation in Rust (Recommended as Failsafe)

Modify `kernel/src/memory/mod.rs` to ignore `_kernel_end` for the purposes of heap reservation and explicitly use `__heap_end`.

**Instructions:**

1.  Import `__heap_end` in `kernel/src/memory/mod.rs`:
    ```rust
    unsafe extern "C" {
        static _kernel_virt_start: u8;
        static _kernel_end: u8;
        static __heap_end: u8; // Import this
    }
    ```

2.  Update the reservation logic in `init()`:
    ```rust
    let kernel_start = mmu::virt_to_phys(unsafe { &_kernel_virt_start as *const _ as usize });
    
    // Instead of trusting _kernel_end, calculate the MAX of _kernel_end and __heap_end
    let kernel_end_symbol = mmu::virt_to_phys(unsafe { &_kernel_end as *const _ as usize });
    let heap_end_symbol = mmu::virt_to_phys(unsafe { &__heap_end as *const _ as usize });
    
    let reserved_end = core::cmp::max(kernel_end_symbol, heap_end_symbol);
    
    add_reserved(
        &mut reserved_regions,
        &mut res_count,
        kernel_start,
        reserved_end, // Use the extended range
    );
    ```

3.  **Why this is better:** It decouples safety from the vagaries of linker script "current location" logic. It explicitly says "Reserve everything up to the defined Heap End".

---

## 5. Artifacts Cleanup

Once fixed, please remove the debugging clutter we left behind to help you:

1.  **`levitate-hal/src/allocator/buddy.rs`**: Remove `println!` in `alloc()` and the loop.
2.  **`kernel/src/memory/mod.rs`**: Remove the `[MEMORY]` debug prints.
3.  **`kernel/src/main.rs`**: Revert the `verbose!` macro logic if desired (though keeping it enabled might be good for a while).

## 6. Verification Checklist

- [ ] `cargo xtask run` boots without Data Abort.
- [ ] Log output shows `[MEMORY] Reserved Kernel: ... - 0x41F00000` (or similar high address).
- [ ] Userspace "Hello World" prints successfully.

---
**END OF REPORT**
