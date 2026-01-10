# Phase 3 — Step 5: MMU & Higher-Half Kernel

## Parent
[Phase 3: Implementation](phase-3.md)

## Goal
Implement PML4 page tables and transition the kernel to higher-half virtual addresses.

## Prerequisites
- Step 2 (Early Boot) complete — basic paging enabled
- Step 4 (HAL) partially complete — at least serial output works

---

## UoW 5.1: Define Page Table Entry Structures

**Goal**: Create Rust types for x86_64 4-level page tables.

**File**: `crates/hal/src/x86_64/paging.rs` (new)

**Tasks**:
1. Create `paging.rs`
2. Define `PageTableEntry` as a u64 wrapper with bitflags:
   - Bit 0: Present
   - Bit 1: Writable
   - Bit 2: User accessible
   - Bit 3: Write-through
   - Bit 4: Cache disable
   - Bit 5: Accessed
   - Bit 6: Dirty
   - Bit 7: Huge page (PS)
   - Bit 8: Global
   - Bits 12-51: Physical address
   - Bit 63: No execute
3. Define `PageTable` as `[PageTableEntry; 512]`
4. Implement `PageTableEntry::new(phys_addr: u64, flags: u64)`
5. Implement `PageTableEntry::address() -> u64`
6. Implement `PageTableEntry::flags() -> u64`

**Exit Criteria**:
- `PageTable` is exactly 4096 bytes
- Entry manipulation is correct

**Verification**:
```rust
assert_eq!(core::mem::size_of::<PageTable>(), 4096);
```

---

## UoW 5.2: Implement Page Table Walker

**Goal**: Traverse 4-level page tables to resolve virtual addresses.

**File**: `crates/hal/src/x86_64/paging.rs` (continued)

**Tasks**:
1. Implement `translate_addr(pml4: &PageTable, virt: u64) -> Option<u64>`:
   - Extract PML4 index (bits 39-47)
   - Extract PDPT index (bits 30-38)
   - Extract PD index (bits 21-29)
   - Extract PT index (bits 12-20)
   - Walk each level, checking Present bit
   - Handle 2MB huge pages (check PS bit at PD level)
   - Handle 1GB huge pages (check PS bit at PDPT level)
2. Return physical address + page offset

**Exit Criteria**:
- Can translate known mapped addresses
- Returns None for unmapped addresses

**Verification**:
- Translate kernel's own address, verify physical match

---

## UoW 5.3: Implement 4KB Page Mapper

**Goal**: Map a single 4KB page at a given virtual address.

**File**: `crates/hal/src/x86_64/paging.rs` (continued)

**Tasks**:
1. Implement `map_page(pml4: &mut PageTable, virt: u64, phys: u64, flags: u64)`:
   - Create PDPT entry if not present (allocate frame)
   - Create PD entry if not present
   - Create PT entry if not present
   - Set final PT entry to point to physical frame
2. Implement helper `allocate_page_table() -> &mut PageTable`:
   - Use frame allocator (from kernel or HAL)
   - Zero-initialize new table
3. Handle errors: return Result with mapping failures

**Exit Criteria**:
- Can map arbitrary virtual → physical
- Missing intermediate tables are created

**Verification**:
- Map a page, then translate_addr to verify

---

## UoW 5.4: Implement Page Unmapper

**Goal**: Unmap a virtual address and optionally free frames.

**File**: `crates/hal/src/x86_64/paging.rs` (continued)

**Tasks**:
1. Implement `unmap_page(pml4: &mut PageTable, virt: u64) -> Result<u64, MmuError>`:
   - Walk to PT entry
   - Check Present bit
   - Clear entry (set to 0)
   - Return old physical address
2. Implement TLB invalidation: `invlpg` instruction after unmap

**Exit Criteria**:
- Unmapped page causes page fault on access
- TLB is properly invalidated

**Verification**:
- Map, access, unmap, access → page fault

---

## UoW 5.5: Implement Frame Allocator Interface

**Goal**: Create a simple frame allocator for page table allocation.

**File**: `crates/hal/src/x86_64/frame_alloc.rs` (new)

**Tasks**:
1. Create `frame_alloc.rs`
2. Define `FrameAllocator` trait (or use existing from HAL)
3. Implement simple `BumpAllocator`:
   - Start at end of kernel image
   - Allocate 4KB frames sequentially
   - Track next free frame address
4. Later: integrate with proper physical memory manager

**Exit Criteria**:
- Can allocate frames for page tables
- Works for early boot

**Verification**:
- Allocate 10 frames, verify addresses are sequential

---

## UoW 5.6: Create Higher-Half Kernel Mappings

**Goal**: Set up mappings for kernel in higher-half address space.

**File**: `crates/hal/src/x86_64/paging.rs` (continued)

**Tasks**:
1. Define kernel virtual base: `0xFFFFFFFF80000000`
2. Implement `setup_kernel_mappings(pml4: &mut PageTable)`:
   - Map kernel code section (read + execute)
   - Map kernel rodata section (read only)
   - Map kernel data/bss section (read + write)
   - Map kernel stack
3. Use linker symbols for section boundaries
4. Map first 4GB identity (for MMIO access)

**Exit Criteria**:
- Kernel accessible at higher-half address
- MMIO still works via identity map

**Verification**:
- Access kernel symbol via higher-half pointer

---

## UoW 5.7: Implement CR3 Switching

**Goal**: Switch to new page tables by updating CR3.

**File**: `crates/hal/src/x86_64/paging.rs` (continued)

**Tasks**:
1. Implement `switch_to(pml4_phys: u64)`:
   - Write to CR3 register
   - Inline assembly: `mov cr3, rax`
2. Implement `current_pml4() -> u64`:
   - Read CR3 register
3. Ensure TLB is flushed on CR3 write

**Exit Criteria**:
- Can switch between different page table sets
- No crash after switch

**Verification**:
- Switch to new PML4, verify old mappings work

---

## UoW 5.8: Implement MmuInterface Trait for PML4

**Goal**: Make x86_64 paging conform to the `MmuInterface` trait.

**File**: `crates/hal/src/x86_64/paging.rs` (continued)

**Tasks**:
1. Create `Pml4Mmu` struct holding PML4 base address
2. Implement `MmuInterface` trait:
   - `map_page(va, pa, flags)`: convert generic flags to x86_64 bitflags
   - `unmap_page(va)`: call internal unmap
   - `switch_to()`: write CR3
3. Define `PageFlags` → x86_64 flags translation

**Exit Criteria**:
- Kernel can use generic `MmuInterface`
- AArch64 and x86_64 MMU share same interface

**Verification**:
- Use trait methods in generic kernel code

---

## UoW 5.9: Transition to Higher-Half at Boot

**Goal**: Perform the jump from identity-mapped to higher-half kernel.

**File**: `kernel/src/arch/x86_64/boot.S` (modify)

**Tasks**:
1. After entering long mode:
   - Set up full page tables with higher-half mappings
   - Reload CR3 with new PML4
   - Jump to higher-half kernel_main address
   - Update RSP to higher-half stack
2. Remove identity mapping after jump (optional, for cleanliness)
3. Update Rust `kernel_main` address in linker script

**Exit Criteria**:
- Kernel runs entirely in higher-half
- Identity map can be removed

**Verification**:
- Print kernel symbol addresses, verify they start with 0xFFFFFFFF8...

---

## Progress Tracking
- [x] UoW 5.1: Page Table Structures
- [x] UoW 5.2: Page Walker
- [x] UoW 5.3: Page Mapper
- [x] UoW 5.4: Page Unmapper
- [x] UoW 5.5: Frame Allocator
- [x] UoW 5.6: Higher-Half Mappings
- [x] UoW 5.7: CR3 Switching
- [x] UoW 5.8: MmuInterface Trait
- [x] UoW 5.9: Boot Transition

## Dependencies Graph
```
UoW 5.1 ──→ UoW 5.2 ──→ UoW 5.3 ──→ UoW 5.4
                              ↓
UoW 5.5 ─────────────────────→ UoW 5.6 ──→ UoW 5.7 ──→ UoW 5.9
                                     ↓
                               UoW 5.8
```
