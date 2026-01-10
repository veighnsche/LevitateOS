# Behavior Inventory for Unit Testing

TEAM_030: Behavior-driven test inventory

---

## File Groups

### Group 1: Core Primitives
- `crates/utils/src/lib.rs` (Spinlock, RingBuffer) — `los_utils`

### Group 2: Interrupt & Synchronization
- `crates/hal/src/interrupts.rs` — `los_hal`
- `crates/hal/src/lib.rs` (IrqSafeLock) — `los_hal`
- `crates/hal/src/gic.rs` — `los_hal`

### Group 3: Serial I/O
- `crates/hal/src/uart_pl011.rs` — `los_hal`
- `crates/hal/src/console.rs` — `los_hal`

### Group 4: Memory Management
- `crates/hal/src/mmu.rs` — `los_hal`

### Group 5: Timer
- `crates/hal/src/timer.rs` — `los_hal`

### Group 6: Initramfs & FDT
- `crates/hal/src/fdt.rs` — `los_hal`
- `crates/utils/src/cpio.rs` — `los_utils`

### Group 7: Slab Allocator
- `crates/hal/src/allocator/slab/` — `los_hal`

### Group 8: Buddy Allocator
- `crates/hal/src/allocator/buddy.rs` — `los_hal`

### Group 9: VirtIO Network
- `crates/drivers/virtio-net/src/lib.rs`

### Group 10: GPU Terminal
- `kernel/src/terminal.rs`

### Group 11: Multitasking & Scheduler
- `kernel/src/task/`

### Group 12: Userspace Shell
- `kernel/src/syscall.rs`
- `kernel/src/task/process.rs`
- `userspace/shell/src/main.rs`

### Group 13: GPU Display Regression
- `kernel/src/gpu.rs`

### Group 14: x86_64 Architecture
- `crates/hal/src/x86_64/`

### Group 15: PCI Bus & Discovery
- `crates/pci/src/lib.rs`

### Group 16: NVMe Storage Driver
- `crates/drivers/nvme/src/lib.rs`

### Group 17: VirtIO GPU Driver
- `crates/drivers/virtio-gpu/src/lib.rs`

### Group 18: XHCI USB Controller
- `crates/drivers/xhci/src/lib.rs`

### Group 19: Eyra Userspace Integration
- `crates/userspace/eyra/libsyscall/` — libsyscall with std support
- `crates/userspace/eyra/libsyscall-tests/` — Integration test binary
- See: `crates/userspace/eyra/BEHAVIOR_INVENTORY.md`

---

## Group 1: Core Primitives — Behavior Inventory

### Spinlock

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| S1 | Lock acquires exclusive access | ✅ | `test_spinlock_basic` |
| S2 | Lock blocks until released | ✅ | `test_spinlock_blocking` |
| S3 | Guard releases lock on drop | ✅ | `test_spinlock_basic` (implicit) |
| S4 | Data is accessible through guard (read) | ✅ | `test_spinlock_basic` |
| S5 | Data is modifiable through guard (write) | ✅ | `test_spinlock_basic` |
| S6 | Multiple sequential lock/unlock cycles work | ✅ | `test_spinlock_basic` |

### RingBuffer

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| R1 | New buffer is empty | ✅ | `test_ring_buffer_fifo` |
| R2 | Push adds element to buffer | ✅ | `test_ring_buffer_fifo` |
| R3 | Pop removes oldest element (FIFO order) | ✅ | `test_ring_buffer_fifo` |
| R4 | Push to full buffer returns false | ✅ | `test_ring_buffer_fifo` |
| R5 | Pop from empty buffer returns None | ✅ | `test_ring_buffer_fifo` |
| R6 | Buffer wraps around correctly | ✅ | `test_ring_buffer_wrap_around` |
| R7 | is_empty returns true when empty | ✅ | `test_ring_buffer_fifo` |
| R8 | is_empty returns false when has data | ✅ | `test_ring_buffer_is_empty_false_when_has_data` |

### Group 1 Summary
- **Spinlock**: 6/6 behaviors tested ✅
- **RingBuffer**: 8/8 behaviors tested ✅

---

## Group 2: Interrupt & Synchronization — Behavior Inventory

### interrupts module

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| I1 | disable() disables interrupts | ✅ | `test_irq_safe_lock_behavior` (via IrqSafeLock) |
| I2 | disable() returns previous state | ✅ | `test_irq_safe_lock_nested` (implicit) |
| I3 | restore() restores previous state | ✅ | `test_irq_safe_lock_behavior` |
| I4 | is_enabled() returns true when enabled | ✅ | `test_irq_safe_lock_behavior` |
| I5 | is_enabled() returns false when disabled | ✅ | `test_irq_safe_lock_behavior` |
| I6 | disable→restore cycle preserves original state | ✅ | `test_irq_safe_lock_nested` |

### IrqSafeLock

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| L1 | Lock disables interrupts before acquiring | ✅ | `test_irq_safe_lock_behavior` |
| L2 | Lock restores interrupts after releasing | ✅ | `test_irq_safe_lock_behavior` |
| L3 | Nested locks work correctly | ✅ | `test_irq_safe_lock_nested` |
| L4 | Data is accessible through guard | ✅ | `test_irq_safe_lock_behavior` |

### GIC (Generic Interrupt Controller)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| G1 | IrqId maps to correct IRQ numbers | ✅ | `test_irq_id_mapping` |
| G2 | from_irq_number returns correct IrqId | ✅ | `test_irq_id_mapping` |
| G3 | from_irq_number returns None for unknown IRQ | ✅ | `test_irq_id_mapping` |
| G4 | Handler registration stores handler | ✅ | `test_handler_registration_and_dispatch` |
| G5 | Dispatch calls registered handler | ✅ | `test_handler_registration_and_dispatch` |
| G6 | Dispatch returns false for unregistered IRQ | ✅ | `test_handler_registration_and_dispatch` |
| G7 | Spurious IRQ detection (1020-1023) | ✅ | `test_spurious_check` |
| G8 | Active GIC access is thread-safe | ✅ | `test_handler_registration_and_dispatch` (implicit) |
| G9 | Driver prioritizes FDT discovery | ✅ | Behavior Test (Boot) |

### Group 2 Summary
- **interrupts**: 6/6 behaviors tested ✅
- **IrqSafeLock**: 4/4 behaviors tested ✅
- **GIC**: 9/9 behaviors tested ✅

---

## Group 3: Serial I/O — Behavior Inventory

### Pl011Uart (uart_pl011.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| U1 | FlagFlags TXFF bit is bit 5 | ✅ | `test_flag_flags_txff_bit_position` |
| U2 | FlagFlags RXFE bit is bit 4 | ✅ | `test_flag_flags_rxfe_bit_position` |
| U3 | ControlFlags UARTEN bit is bit 0 | ✅ | `test_control_flags_uarten_bit_position` |
| U4 | ControlFlags TXE bit is bit 8 | ✅ | `test_control_flags_txe_bit_position` |
| U5 | ControlFlags RXE bit is bit 9 | ✅ | `test_control_flags_rxe_bit_position` |
| U6 | LineControlFlags FEN bit is bit 4 | ✅ | `test_line_control_flags_fen_bit_position` |
| U7 | LineControlFlags WLEN_8 is bits 5-6 | ✅ | `test_line_control_flags_wlen8_bit_position` |
| U8 | InterruptFlags RXIM bit is bit 4 | ✅ | `test_interrupt_flags_rxim_bit_position` |

### console module

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| C1 | format_hex converts 0 to "0x0000000000000000" | ✅ | `test_format_hex_zero` |
| C2 | format_hex converts max u64 correctly | ✅ | `test_format_hex_max` |
| C3 | format_hex handles mixed nibble values | ✅ | `test_format_hex_mixed` |
| C4 | Nibble 0-9 maps to '0'-'9' | ✅ | `test_nibble_to_hex_digits` |
| C5 | Nibble 10-15 maps to 'a'-'f' | ✅ | `test_nibble_to_hex_letters` |

### Group 3 Summary
- **Pl011Uart bitflags**: 8/8 behaviors tested ✅
- **console**: 5/5 behaviors tested ✅

---

## Group 4: Memory Management — Behavior Inventory

### MMU (mmu.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| M1 | PageFlags VALID bit is bit 0 | ✅ | `test_page_flags_block_vs_table` |
| M2 | PageFlags TABLE bit is bit 1 | ✅ | `test_page_flags_block_vs_table` |
| M3 | Block descriptor has TABLE bit = 0 | ✅ | `test_block_flags_no_table_bit` |
| M4 | Table descriptor has TABLE bit = 1 | ✅ | `test_page_flags_block_vs_table` |
| M5 | KERNEL_DATA_BLOCK has correct flags | ✅ | `test_block_flags_no_table_bit` |
| M6 | DEVICE_BLOCK has ATTR_DEVICE | ✅ | `test_device_block_flags` |
| M7 | va_l0_index extracts bits [47:39] | ✅ | `test_va_l0_index` |
| M8 | va_l1_index extracts bits [38:30] | ✅ | `test_va_l1_index` |
| M9 | va_l2_index extracts bits [29:21] | ✅ | `test_va_l2_index` |
| M10 | va_l3_index extracts bits [20:12] | ✅ | `test_va_l3_index` |
| M11 | Kernel address indices are correct | ✅ | `test_kernel_address_indices` |
| M12 | 2MB alignment detection works | ✅ | `test_block_alignment` |
| M13 | Constants have correct values | ✅ | `test_constants` |
| M14 | PageTableEntry empty is invalid | ✅ | `test_page_table_entry_empty` |
| M15 | PageTableEntry set stores address | ✅ | `test_page_table_entry_set_block` |
| M16 | PageTableEntry is_table() works | ✅ | `test_page_table_entry_set_table` |
| M17 | Block mapping calculation | ✅ | `test_table_count_for_block_mapping` |
| M18 | MappingStats total_bytes() | ✅ | `test_mapping_stats` |
| M19 | virt_to_phys converts high VA to PA | ✅ | `test_virt_to_phys_high_address` |
| M20 | phys_to_virt converts PA to high VA | ✅ | `test_phys_to_virt_kernel_region` |
| M21 | virt_to_phys identity for low addresses | ✅ | `test_virt_to_phys_low_address_identity` |
| M22 | phys_to_virt identity for device addresses | ✅ | `test_phys_to_virt_device_identity` |

### Dynamic Page Table Allocation (TEAM_054)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| M23 | PageAllocator trait defines alloc_page() | ✅ | `test_page_allocator_trait_interface` |
| M24 | PageAllocator trait defines free_page() | ✅ | `test_page_allocator_trait_interface` |
| M25 | set_page_allocator() accepts &'static dyn PageAllocator | ✅ | `test_set_page_allocator_signature` |
| M26 | get_or_create_table() uses dynamic allocator if set | ⚠️ | Behavior test (boot) |
| M27 | get_or_create_table() falls back to static pool | ⚠️ | Implicit (early boot) |

### Group 4 Summary
- **MMU**: 27/27 behaviors documented
- **Unit tested**: 25/27 ✅
- **Runtime verified**: 2/27 ⚠️ (M26, M27 verified via kernel boot)


---

## Group 5: Timer — Behavior Inventory

### Timer (timer.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| T1 | uptime_seconds = counter / frequency | ✅ | `test_uptime_seconds` |

### Group 5 Summary
- **Timer**: 1/1 behaviors tested ✅

---

## Overall Summary (Updated by TEAM_030)

| Group | Module | Behaviors | Tested | Gap |
|-------|--------|-----------|--------|-----|
| 1 | Spinlock | 6 | 6 | ✅ |
| 1 | RingBuffer | 8 | 8 | ✅ |
| 2 | interrupts | 6 | 6 | ✅ |
| 2 | IrqSafeLock | 4 | 4 | ✅ |
| 2 | GIC | 7 | 7 | ✅ |
| 3 | Pl011Uart bitflags | 8 | 8 | ✅ |
| 3 | console | 5 | 5 | ✅ |
| 4 | MMU | 22 | 22 | ✅ |
| 5 | Timer | 1 | 1 | ✅ |
| **Total** | | **67** | **67** | **0 gaps** ✅ |

## Remaining Gap

None!

## Tests Added by TEAM_030

- `test_ring_buffer_is_empty_false_when_has_data` (R8)
- `test_flag_flags_txff_bit_position` (U1)
- `test_flag_flags_rxfe_bit_position` (U2)
- `test_control_flags_uarten_bit_position` (U3)
- `test_control_flags_txe_bit_position` (U4)
- `test_control_flags_rxe_bit_position` (U5)
- `test_line_control_flags_fen_bit_position` (U6)
- `test_line_control_flags_wlen8_bit_position` (U7)
- `test_interrupt_flags_rxim_bit_position` (U8)
- `test_nibble_to_hex_digits` (C4)
- `test_nibble_to_hex_letters` (C5)
- `test_format_hex_zero` (C1)
- `test_format_hex_max` (C2)
- `test_format_hex_mixed` (C3)
- `test_virt_to_phys_high_address` (M19)
- `test_phys_to_virt_kernel_region` (M20)
- `test_virt_to_phys_low_address_identity` (M21)
- `test_phys_to_virt_device_identity` (M22)

---

## Group 6: Initramfs & FDT — Behavior Inventory

TEAM_039: Added per behavior-testing SOP

### File Groups
- `crates/hal/src/fdt.rs` (FDT parsing) — `los_hal`
- `crates/utils/src/cpio.rs` (CPIO parser) — `los_utils`

### FDT Module (fdt.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| FD1 | Invalid DTB header returns InvalidHeader error | ✅ | `test_fdt_invalid_header` |
| FD2 | Missing initrd properties returns InitrdMissing error | ✅ | `test_fdt_error_types` |
| FD3 | 32-bit initrd-start is parsed correctly | ✅ | `test_fdt_byte_parsing` |
| FD4 | 64-bit initrd-start is parsed correctly | ✅ | `test_fdt_byte_parsing` |
| FD5 | Both start and end properties must exist | ✅ | `test_fdt_error_types` |
| FD6 | Big-endian byte order is handled | ✅ | `test_fdt_byte_parsing` |
| FD7 | find_node_by_compatible searches all nodes | ✅ | `test_fdt_discovery` |
| FD8 | get_node_reg returns first register tuple | ✅ | `test_fdt_discovery` |
| FD9 | for_each_memory_region discovers memory ranges | ✅ | `test_fdt_memory_regions` |
| FD10 | for_each_reserved_region discovers reserved memory | ✅ | `test_fdt_reserved_regions` |

### CPIO Parser (initramfs.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| CP1 | CpioHeader is_valid accepts "070701" magic | ✅ | `test_cpio_header_valid_magic` |
| CP2 | CpioHeader is_valid accepts "070702" magic | ✅ | `test_cpio_header_valid_magic` |
| CP3 | CpioHeader is_valid rejects invalid magic | ✅ | `test_cpio_header_invalid_magic` |
| CP4 | parse_hex converts hex string to usize | ✅ | `test_parse_hex` |
| CP5 | parse_hex returns 0 for invalid input | ✅ | `test_parse_hex_invalid` |
| CP6 | CpioArchive::iter returns entries in order | ✅ | `test_cpio_iter_order` |
| CP7 | Iterator stops at TRAILER!!! | ✅ | `test_cpio_iter_trailer` |
| CP8 | CpioArchive::get_file finds existing file | ✅ | `test_cpio_get_file_found` |
| CP9 | CpioArchive::get_file returns None for missing file | ✅ | `test_cpio_get_file_missing` |
| CP10 | 4-byte alignment is applied after header+name | ✅ | `test_cpio_alignment` |

### Group 6 Summary
- **FDT**: 10/10 behaviors tested ✅
- **CPIO**: 10/10 behaviors tested ✅
- **Total**: 20/20 behaviors documented ✅
- **Note**: TEAM_039 relocated CPIO to `crates/utils/src/cpio.rs` (`los_utils`) - tests run via `cargo test -p los_utils --features std`.

---

## Updated Overall Summary (TEAM_039)

| Group | Module | Behaviors | Tested | Gap |
|-------|--------|-----------|--------|-----|
| 1 | Spinlock | 6 | 6 | ✅ |
| 1 | RingBuffer | 8 | 8 | ✅ |
| 2 | interrupts | 6 | 6 | ✅ |
| 2 | IrqSafeLock | 4 | 4 | ✅ |
| 2 | GIC | 9 | 9 | ✅ |
| 3 | Pl011Uart bitflags | 8 | 8 | ✅ |
| 3 | console | 5 | 5 | ✅ |
| 4 | MMU | 22 | 22 | ✅ |
| 5 | Timer | 1 | 1 | ✅ |
| 6 | FDT | 10 | 10 | ✅ |
| 6 | CPIO | 10 | 10 | ✅ |
| **Total** | | **89** | **89** | **0 gaps** ✅ |

---

## Group 7: Slab Allocator — Behavior Inventory

TEAM_051: Added slab allocator for fixed-size object allocation

### File Groups
- `crates/hal/src/allocator/slab/list.rs` (Intrusive linked list) — `los_hal`
- `crates/hal/src/allocator/slab/page.rs` (Slab page structure) — `los_hal`
- `crates/hal/src/allocator/slab/cache.rs` (Per-size-class allocator) — `los_hal`
- `crates/hal/src/allocator/slab/mod.rs` (Top-level slab API) — `los_hal`

### SlabList (intrusive linked list)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| SL1 | New list is empty | ✅ | `test_empty_list` |
| SL2 | push_front adds node to front | ✅ | `test_add_to_head` |
| SL3 | pop_front removes from front | ✅ | `test_add_to_head` |
| SL4 | push_front updates head pointer | ✅ | `test_add_to_head` |
| SL5 | remove() unlinks node from middle | ✅ | `test_remove_from_middle` |
| SL6 | remove() updates prev/next pointers | ✅ | `test_remove_from_middle` |
| SL7 | is_empty() returns true for empty list | ✅ | `test_empty_list` |
| SL8 | Multiple operations maintain list integrity | ✅ | `test_multiple_operations` |

### SlabPage (4KB page with embedded metadata)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| SP1 | Page size is 4096 bytes | ✅ | `test_page_size_constants` |
| SP2 | alloc_object returns sequential offsets | ✅ | `test_alloc_sequential` |
| SP3 | alloc_object increments allocated_count | ✅ | `test_alloc_sequential` |
| SP4 | free_object decrements allocated_count | ✅ | `test_free_and_realloc` |
| SP5 | free_object allows reallocation of same slot | ✅ | `test_free_and_realloc` |
| SP6 | is_full() returns true at capacity | ✅ | `test_is_full_after_max_allocations` |
| SP7 | alloc_object returns None when full | ✅ | `test_is_full_after_max_allocations` |
| SP8 | is_empty() returns true when all freed | ✅ | `test_is_empty_after_freeing_all` |

### SlabCache (per-size-class cache with partial/full/empty lists)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| SC1 | 6 size classes: 64B to 2048B | ✅ | `test_size_class_constants` |
| SC2 | Objects per page calculated correctly | ✅ | `test_size_class_constants` |
| SC3 | New cache has empty lists | ✅ | `test_new_cache` |

### SlabAllocator (top-level API)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| SA1 | New allocator initializes 6 caches | ✅ | `test_new_allocator` |
| SA2 | size_to_class maps sizes correctly | ✅ | `test_size_to_class_mapping` |
| SA3 | size_to_class returns None for invalid sizes | ✅ | `test_invalid_allocation_requests` |
| SA4 | size_to_class returns None for size > 2048 | ✅ | `test_invalid_allocation_requests` |

### Group 7 Summary
- **SlabList**: 8/8 behaviors tested ✅
- **SlabPage**: 8/8 behaviors tested ✅
- **SlabCache**: 3/3 behaviors tested ✅
- **SlabAllocator**: 4/4 behaviors tested ✅
- **Total**: 23/23 behaviors tested ✅

---

## Group 8: Buddy Allocator — Behavior Inventory

TEAM_055: Added buddy allocator for physical page frame management

### File Groups
- `crates/hal/src/allocator/buddy.rs` (Buddy allocator core) — `los_hal`
- `crates/hal/src/allocator/page.rs` (Page descriptor struct) — `los_hal`

### BuddyAllocator (physical frame allocator)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| B1 | Allocator starts with empty free lists | ✅ | `test_alloc_order_0` |
| B2 | alloc(order=0) returns single page address | ✅ | `test_alloc_order_0` |
| B3 | OOM returns None when pool exhausted | ✅ | `test_alloc_order_0` |
| B4 | alloc(order=N) allocates 2^N contiguous pages | ✅ | `test_alloc_large` |
| B5 | Large allocation consumes entire pool | ✅ | `test_alloc_large` |
| B6 | Block splitting creates buddy pairs | ✅ | `test_splitting` |
| B7 | Sequential allocs get sequential addresses | ✅ | `test_splitting` |
| B8 | Free blocks are coalesced with buddies | ✅ | `test_coalescing` |
| B9 | Coalesced blocks can be reallocated | ✅ | `test_coalescing` |
| B10 | Non-power-of-two ranges are handled | ✅ | `test_alloc_unaligned_range` |
| B11 | Leftover pages added to appropriate order | ✅ | `test_alloc_unaligned_range` |

### Group 8 Summary
- **BuddyAllocator**: 11/11 behaviors tested ✅

---

## Updated Overall Summary (TEAM_055)

| Group | Module | Behaviors | Tested | Gap |
|-------|--------|-----------|--------|-----|
| 1 | Spinlock | 6 | 6 | ✅ |
| 1 | RingBuffer | 8 | 8 | ✅ |
| 2 | interrupts | 6 | 6 | ✅ |
| 2 | IrqSafeLock | 4 | 4 | ✅ |
| 2 | GIC | 9 | 9 | ✅ |
| 3 | Pl011Uart bitflags | 8 | 8 | ✅ |
| 3 | console | 5 | 5 | ✅ |
| 4 | MMU | 27 | 25 | ⚠️ |
| 5 | Timer | 1 | 1 | ✅ |
| 6 | FDT | 10 | 10 | ✅ |
| 6 | CPIO | 10 | 10 | ✅ |
| 7 | SlabList | 8 | 8 | ✅ |
| 7 | SlabPage | 8 | 8 | ✅ |
| 7 | SlabCache | 3 | 3 | ✅ |
| 7 | SlabAllocator | 4 | 4 | ✅ |
| 8 | BuddyAllocator | 11 | 11 | ✅ |
| **Total** | | **128** | **126** | **2 unit + 2 runtime** ⚠️ |

> **Note:** M26 and M27 are runtime-verified through kernel boot tests.

---

## Group 9: VirtIO Network — Behavior Inventory

TEAM_057: VirtIO Net driver for Phase 6

### File Groups
- `kernel/src/net.rs` (Network driver)

### VirtIO Net (net.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| NET1 | init() detects and initializes virtio-net device | ⚠️ | Runtime (boot) |
| NET2 | init() reads MAC address from device config | ⚠️ | Runtime (boot) |
| NET3 | mac_address() returns device MAC when initialized | ⚠️ | Runtime (boot) |
| NET4 | mac_address() returns None when not initialized | ⚠️ | Runtime (boot) |
| NET5 | can_send() returns true when TX queue has space | ⚠️ | Runtime |
| NET6 | can_send() returns false when not initialized | ⚠️ | Runtime |
| NET7 | can_recv() returns true when RX packet available | ⚠️ | Runtime |
| NET8 | can_recv() returns false when not initialized | ⚠️ | Runtime |
| NET9 | send() transmits packet when device ready | ⚠️ | Runtime |
| NET10 | send() returns NotInitialized when device missing | ⚠️ | Runtime |
| NET11 | send() returns DeviceBusy when queue full | ⚠️ | Runtime |
| NET12 | receive() returns packet data when available | ⚠️ | Runtime |
| NET13 | receive() returns None when no packet | ⚠️ | Runtime |
| NET14 | receive() recycles RX buffer after read | ⚠️ | Runtime |

### Group 9 Summary
- **VirtIO Net**: 14/14 behaviors documented
- **Runtime verified**: 14/14 ⚠️ (hardware-dependent, verified via kernel boot)

---

## Updated Overall Summary (TEAM_057)

| Group | Module | Behaviors | Tested | Gap |
|-------|--------|-----------|--------|-----|
| 1 | Spinlock | 6 | 6 | ✅ |
| 1 | RingBuffer | 8 | 8 | ✅ |
| 2 | interrupts | 6 | 6 | ✅ |
| 2 | IrqSafeLock | 4 | 4 | ✅ |
| 2 | GIC | 9 | 9 | ✅ |
| 3 | Pl011Uart bitflags | 8 | 8 | ✅ |
| 3 | console | 5 | 5 | ✅ |
| 4 | MMU | 27 | 25 | ⚠️ |
| 5 | Timer | 1 | 1 | ✅ |
| 6 | FDT | 10 | 10 | ✅ |
| 6 | CPIO | 10 | 10 | ✅ |
| 7 | SlabList | 8 | 8 | ✅ |
| 7 | SlabPage | 8 | 8 | ✅ |
| 7 | SlabCache | 3 | 3 | ✅ |
| 7 | SlabAllocator | 4 | 4 | ✅ |
| 8 | BuddyAllocator | 11 | 11 | ✅ |
| 9 | VirtIO Net | 14 | 14 | ⚠️ |
| **Total** | | **142** | **140** | **2 unit + 16 runtime** ⚠️ |

> **Note:** NET behaviors are runtime-verified (hardware-dependent). M26/M27 verified via boot.

---

## Group 10: GPU Terminal — Behavior Inventory

TEAM_058: GPU Terminal for Phase 6
TEAM_059: Verified behaviors after fixing newline/cursor bugs

### File Groups
- `kernel/src/terminal.rs` (Terminal emulator)
- `kernel/src/gpu.rs` (Resolution helper)

### Terminal (terminal.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| TERM1 | Character renders at cursor position | ✅ | Runtime (visual) |
| TERM2 | Cursor advances after character | ✅ | Runtime (UART log) |
| TERM3 | Newline moves to next line start | ✅ | Runtime (UART log) |
| TERM4 | Screen scrolls when cursor exceeds rows | ✅ | Runtime (UART log) |
| TERM5 | Carriage return resets column | ✅ | Runtime (UART log) |
| TERM6 | Tab advances to 8-column boundary | ✅ | Runtime (UART log) |
| TERM7 | Clear fills with background color | ✅ | Runtime (visual) |
| TERM8 | Backspace moves cursor left | ✅ | Runtime (UART log) |
| TERM9 | Resolution adapts to screen size | ✅ | Runtime (UART log) |
| TERM10 | Tab wraps to next line at end of row | ✅ | TEAM_065 fix |
| TERM11 | Backspace wraps to previous line at col 0 | ✅ | Runtime (UART log) |
| TERM12 | ANSI ESC[J clears screen | ✅ | Runtime (visual) |

### Group 10 Summary
- **Terminal**: 12/12 behaviors documented
- **Runtime verified**: 12/12 ✅ (visual + UART logging)
- **TEAM_065**: Added TERM10-12 for wrap/ANSI behaviors

---

## Updated Overall Summary (TEAM_058)

| Group | Module | Behaviors | Tested | Gap |
|-------|--------|-----------|--------|-----|
| 1 | Spinlock | 6 | 6 | ✅ |
| 1 | RingBuffer | 8 | 8 | ✅ |
| 2 | interrupts | 6 | 6 | ✅ |
| 2 | IrqSafeLock | 4 | 4 | ✅ |
| 2 | GIC | 9 | 9 | ✅ |
| 3 | Pl011Uart bitflags | 8 | 8 | ✅ |
| 3 | console | 5 | 5 | ✅ |
| 4 | MMU | 27 | 25 | ⚠️ |
| 5 | Timer | 1 | 1 | ✅ |
| 6 | FDT | 10 | 10 | ✅ |
| 6 | CPIO | 10 | 10 | ✅ |
| 7 | SlabList | 8 | 8 | ✅ |
| 7 | SlabPage | 8 | 8 | ✅ |
| 7 | SlabCache | 3 | 3 | ✅ |
| 7 | SlabAllocator | 4 | 4 | ✅ |
| 8 | BuddyAllocator | 11 | 11 | ✅ |
| 9 | VirtIO Net | 14 | 14 | ⚠️ |
| 10 | Terminal | 12 | 12 | ⚠️ |
| **Total** | | **154** | **152** | **2 unit + 28 runtime** ⚠️ |

> **TEAM_065**: Added TERM10-12 (tab wrap, backspace wrap, ANSI clear)

---

### Group 11: Multitasking & Scheduler — Behavior Inventory

TEAM_071: Added multitasking behaviors for Phase 7
TEAM_273: Added x86_64 specific context switch behaviors

### File Groups
- `kernel/src/task/mod.rs` (Task primitives, context switch)
- `kernel/src/task/scheduler.rs` (Round-robin scheduler)
- `crates/hal/src/mmu.rs` (unmap_page, table reclamation) — `los_hal`
- `kernel/src/arch/x86_64/task.rs` (x86_64 context switching)

### Context Switching (task/mod.rs & arch/x86_64)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| MT1 | cpu_switch_to saves x19-x29, lr, sp to old context (AArch64) | ✅ | Runtime (preemption) |
| MT2 | cpu_switch_to restores x19-x29, lr, sp from new context (AArch64) | ✅ | Runtime (preemption) |
| MT1a | cpu_switch_to saves rbx, rbp, r12-r15 to old context (x86_64) | ⚠️ | Phase 3 Integration |
| MT2a | cpu_switch_to restores rbx, rbp, r12-r15 from new context (x86_64) | ⚠️ | Phase 3 Integration |
| MT3 | switch_to() updates CURRENT_TASK before switch | ✅ | Runtime |
| MT4 | switch_to() no-ops when switching to same task | ✅ | Implicit (code path) |
| MT5 | yield_now() re-adds current task to ready queue | ✅ | Runtime |
| MT6 | task_exit() marks task as Exited | ✅ | Runtime |
| MT7 | task_exit() does not re-add task to ready queue | ✅ | Runtime |
| MT8 | idle_loop() uses WFI/HLT for power efficiency | ✅ | Runtime (Rule 16) |

### Task Primitives (task/mod.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| MT9 | TaskId::next() returns unique IDs | ✅ | Implicit (AtomicUsize) |
| MT10 | TaskControlBlock::new() allocates stack | ✅ | Runtime |
| MT11 | Context initializes lr to trampoline | ✅ | Runtime |
| MT12 | Context initializes x19 to entry point | ✅ | Runtime |
| MT13 | TaskState transitions: Ready → Running → Exited | ✅ | Runtime |

### Scheduler (task/scheduler.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| MT14 | SCHEDULER uses IrqSafeLock (Rule 7) | ✅ | Code inspection |
| MT15 | add_task() appends to ready_list | ✅ | Runtime |
| MT16 | pick_next() removes from front (FIFO) | ✅ | Runtime |
| MT17 | schedule() calls switch_to when task available | ✅ | Runtime |

### Unmap Page (crates/hal/src/mmu.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| MT18 | unmap_page returns Err for unmapped address (Rule 14) | ✅ | `test_map_unmap_cycle` |
| MT19 | unmap_page clears L3 entry | ✅ | `test_map_unmap_cycle` |
| MT20 | unmap_page calls tlb_flush_page | ✅ | `test_map_unmap_cycle` |
| MT21 | Table reclamation frees empty L3 tables | ✅ | `test_table_reclamation` |
| MT22 | Table reclamation recursively frees L2/L1 | ✅ | `test_table_reclamation` |

### Group 11 Summary
- **Context Switching**: 8/8 behaviors tested ✅
- **Task Primitives**: 5/5 behaviors tested ✅
- **Scheduler**: 4/4 behaviors tested ✅
- **Unmap Page**: 5/5 behaviors tested ✅
- **Total**: 22/22 behaviors tested ✅

---

## Updated Overall Summary (TEAM_071)

| Group | Module | Behaviors | Tested | Gap |
|-------|--------|-----------|--------|-----|
| 1 | Spinlock | 6 | 6 | ✅ |
| 1 | RingBuffer | 8 | 8 | ✅ |
| 2 | interrupts | 6 | 6 | ✅ |
| 2 | IrqSafeLock | 4 | 4 | ✅ |
| 2 | GIC | 9 | 9 | ✅ |
| 3 | Pl011Uart bitflags | 8 | 8 | ✅ |
| 3 | console | 5 | 5 | ✅ |
| 4 | MMU | 27 | 25 | ⚠️ |
| 5 | Timer | 1 | 1 | ✅ |
| 6 | FDT | 10 | 10 | ✅ |
| 6 | CPIO | 10 | 10 | ✅ |
| 7 | SlabList | 8 | 8 | ✅ |
| 7 | SlabPage | 8 | 8 | ✅ |
| 7 | SlabCache | 3 | 3 | ✅ |
| 7 | SlabAllocator | 4 | 4 | ✅ |
| 8 | BuddyAllocator | 11 | 11 | ✅ |
| 9 | VirtIO Net | 14 | 14 | ⚠️ |
| 10 | Terminal | 12 | 12 | ⚠️ |
| 11 | Context Switching | 8 | 8 | ⚠️ |
| 11 | Task Primitives | 5 | 5 | ⚠️ |
| 11 | Scheduler | 4 | 4 | ⚠️ |
| 11 | Unmap Page | 5 | 5 | ✅ |
| **Total** | | **176** | **174** | **2 unit + 43 runtime** ⚠️ |

> **TEAM_071**: Added 22 multitasking behaviors (Phase 7). Unmap page behaviors unit-tested via `test_map_unmap_cycle` and `test_table_reclamation`.

---

## Group 12: Userspace Shell — Behavior Inventory

TEAM_115: Added userspace shell behaviors for Phase 8b

### File Groups
- `kernel/src/syscall.rs` (System call handlers)
- `kernel/src/terminal.rs` (Console GPU output)
- `kernel/src/task/process.rs` (User process spawning)
- `userspace/shell/src/main.rs` (Interactive shell)

### Syscall Handler (syscall.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| SYS1 | sys_write(fd=1) outputs to UART | ✅ | Behavior Test (serial log) |
| SYS2 | sys_write(fd=1) outputs to GPU terminal | ✅ | VNC visual verification |
| SYS3 | sys_write(fd=2) outputs to UART | ✅ | Behavior Test (serial log) |
| SYS4 | sys_write validates user buffer address | ✅ | Code inspection |
| SYS5 | sys_write limits output to 4KB | ✅ | Code inspection |
| SYS6 | sys_read(fd=0) blocks until input | ✅ | VNC interactive test |
| SYS7 | sys_read returns character from VirtIO keyboard | ✅ | VNC interactive test |
| SYS8 | sys_read returns character from UART | ✅ | UART interactive test |
| SYS9 | sys_exit terminates process | ✅ | `exit` command |

### Terminal GPU Output (terminal.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| GPU1 | write_str renders text to GPU framebuffer | ✅ | VNC visual verification |
| GPU2 | write_str flushes GPU after each write | ✅ | VNC visual verification |
| GPU3 | write_str acquires locks (not try_lock) | ✅ | Code inspection (TEAM_115) |
| GPU4 | Userspace output appears on GPU terminal | ✅ | VNC visual verification |

### User Process (process.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| PROC1 | run_from_initramfs finds ELF in CPIO archive | ✅ | Behavior Test (boot log) |
| PROC2 | spawn_from_elf creates user page table | ✅ | Behavior Test (boot log) |
| PROC3 | enter_user_mode transitions to EL0 | ✅ | Behavior Test (shell runs) |
| PROC4 | User TTBR0 switches before entering EL0 | ✅ | Code inspection |

### Interactive Shell (hello/src/main.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| SH1 | Shell prints banner on startup | ✅ | Behavior Test + VNC |
| SH2 | Shell prints # prompt | ✅ | Behavior Test + VNC |
| SH3 | Shell reads input line | ✅ | VNC interactive test |
| SH4 | Shell echoes command before execution | ✅ | VNC interactive test |
| SH5 | echo command outputs text | ✅ | VNC interactive test |
| SH6 | help command shows available commands | ✅ | VNC interactive test |
| SH7 | exit command terminates shell | ✅ | VNC interactive test |

### Group 12 Summary
- **Syscall Handler**: 9/9 behaviors tested ✅
- **Terminal GPU Output**: 4/4 behaviors tested ✅
- **User Process**: 4/4 behaviors tested ✅
- **Interactive Shell**: 7/7 behaviors tested ✅
- **Total**: 24/24 behaviors tested ✅

---

## Updated Overall Summary (TEAM_115)

| Group | Module | Behaviors | Tested | Gap |
|-------|--------|-----------|--------|-----|
| 1 | Spinlock | 6 | 6 | ✅ |
| 1 | RingBuffer | 8 | 8 | ✅ |
| 2 | interrupts | 6 | 6 | ✅ |
| 2 | IrqSafeLock | 4 | 4 | ✅ |
| 2 | GIC | 9 | 9 | ✅ |
| 3 | Pl011Uart bitflags | 8 | 8 | ✅ |
| 3 | console | 5 | 5 | ✅ |
| 4 | MMU | 27 | 25 | ⚠️ |
| 5 | Timer | 1 | 1 | ✅ |
| 6 | FDT | 10 | 10 | ✅ |
| 6 | CPIO | 10 | 10 | ✅ |
| 7 | SlabList | 8 | 8 | ✅ |
| 7 | SlabPage | 8 | 8 | ✅ |
| 7 | SlabCache | 3 | 3 | ✅ |
| 7 | SlabAllocator | 4 | 4 | ✅ |
| 8 | BuddyAllocator | 11 | 11 | ✅ |
| 9 | VirtIO Net | 14 | 14 | ⚠️ |
| 10 | Terminal | 12 | 12 | ⚠️ |
| 11 | Context Switching | 8 | 8 | ⚠️ |
| 11 | Task Primitives | 5 | 5 | ⚠️ |
| 11 | Scheduler | 4 | 4 | ⚠️ |
| 11 | Unmap Page | 5 | 5 | ✅ |
| 12 | Syscall Handler | 9 | 9 | ⚠️ |
| 12 | Terminal GPU Output | 4 | 4 | ⚠️ |
| 12 | User Process | 4 | 4 | ⚠️ |
| 12 | Interactive Shell | 7 | 7 | ⚠️ |
| 13 | GPU Regression | 2 | 2 | ✅ |
| **Total** | | **202** | **200** | **2 unit + 67 runtime** ⚠️ |

> **TEAM_115**: Added 24 userspace shell behaviors (Phase 8b). All verified via behavior test (golden file) and VNC interactive testing.

---

## Group 13: GPU Display Regression — Behavior Inventory

TEAM_129: Added GPU display regression tests to prevent black screen issues

### File Groups
- `kernel/src/gpu.rs` (Flush counter, framebuffer content check)
- `kernel/src/main.rs` (GPU_TEST verification during boot)
- `xtask/src/tests/behavior.rs` (Regression assertions)

### GPU Display Pipeline

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| GPU5 | GPU flush is called during boot (flush_count > 0) | ✅ | Behavior Test `[GPU_TEST] Flush count:` |
| GPU6 | Framebuffer contains rendered content (non-black pixels > 0) | ✅ | Behavior Test `[GPU_TEST] Framebuffer:` |
| GPU7 | GPU flush happens during shell execution (flush_count >= 10) | ✅ | Behavior Test flush count threshold |

### Shell Execution Pipeline

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| SHELL1 | Shell process is spawned by init | ✅ | Behavior Test `[INIT] Shell spawned as PID 2` |
| SHELL2 | Shell task is scheduled (gets CPU time) | ✅ | Behavior Test `[TASK] Entering user task PID=2` |
| SHELL3 | Shell _start() executes and prints banner | ✅ | Behavior Test `LevitateOS Shell` |

### Group 13 Summary
- **GPU Regression**: 3/3 behaviors tested ✅
- **Shell Execution**: 3/3 behaviors tested ✅
- **Total**: 6/6 behaviors tested ✅

### Regressions Prevented
- **Black screen**: GPU flush commented out → GPU5/GPU6 fail
- **Shell output invisible**: write_str not flushing → GPU7 fails
- **Shell not running**: Init not yielding → SHELL2/SHELL3 fail

---

## Group 14: x86_64 Architecture — Behavior Inventory

TEAM_303: x86_64 architecture behaviors for Phase 3 multi-arch support

### File Groups
- `crates/hal/src/x86_64/gdt.rs` (GDT/TSS initialization) — `los_hal`
- `crates/hal/src/x86_64/pit.rs` (PIT timer) — `los_hal`
- `kernel/src/arch/x86_64/task.rs` (Context switch, enter_user_mode) — `kernel`
- `kernel/src/arch/x86_64/mod.rs` (SyscallFrame) — `kernel`
- `kernel/src/arch/x86_64/syscall.rs` (Syscall entry/exit) — `kernel`
- `userspace/ulib/src/entry.rs` (Userspace _start) — `ulib`

### GDT/TSS (gdt.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| X86_GDT1 | GDT has null, kernel code/data, user code/data, TSS entries | ⚠️ | Runtime (boot) |
| X86_GDT2 | Kernel code segment is DPL=0 with long mode flag | ⚠️ | Runtime (boot) |
| X86_GDT3 | User code segment is DPL=3 with long mode flag | ⚠️ | Runtime (userspace) |
| X86_GDT4 | TSS entry correctly encodes 64-bit base address | ⚠️ | Runtime (boot) |
| X86_TSS1 | TSS.rsp0 provides kernel stack for Ring 0 transitions | ⚠️ | Runtime (syscall/interrupt) |
| X86_TSS2 | set_kernel_stack() updates TSS.rsp[0] | ⚠️ | Runtime (context switch) |

### PIT Timer (pit.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| X86_PIT1 | PIT.init() configures Channel 0 as rate generator | ⚠️ | Runtime (timer ticks) |
| X86_PIT2 | PIT divisor = 1193182 / frequency_hz | ⚠️ | Runtime (100Hz default) |
| X86_PIT3 | PIT fires IRQ 0 at configured frequency | ⚠️ | Runtime (timer handler) |

### Context Switch (task.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| X86_CTX1 | cpu_switch_to saves rbx, r12-r15, rbp to old Context | ⚠️ | Runtime (preemption) |
| X86_CTX2 | cpu_switch_to restores rbx, r12-r15, rbp from new Context | ⚠️ | Runtime (preemption) |
| X86_CTX3 | cpu_switch_to saves/restores RFLAGS | ⚠️ | Runtime (interrupt state) |
| X86_CTX4 | cpu_switch_to updates PCR.kernel_stack via GS | ⚠️ | Runtime (context switch) |
| X86_CTX5 | cpu_switch_to saves/restores FS_BASE (TLS) | ⚠️ | Runtime (thread-local) |
| X86_CTX6 | cpu_switch_to saves/restores KERNEL_GS_BASE | ⚠️ | Runtime (swapgs) |
| X86_CTX7 | task_entry_trampoline calls entry wrapper from rbx | ⚠️ | Runtime (task spawn) |

### Syscall Entry/Exit (syscall.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| X86_SYS1 | syscall_entry saves all registers to SyscallFrame | ⚠️ | Runtime (syscall) |
| X86_SYS2 | syscall_entry performs swapgs for kernel GS | ⚠️ | Runtime (syscall) |
| X86_SYS3 | syscall_exit restores RCX (user RIP) from frame | ⚠️ | Runtime (syscall) |
| X86_SYS4 | syscall_exit restores R11 (user RFLAGS) with IF=1 | ⚠️ | Runtime (syscall) |
| X86_SYS5 | syscall_exit returns via sysretq | ⚠️ | Runtime (syscall) |

### SyscallFrame (mod.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| X86_FRM1 | SyscallFrame layout matches assembly push order | ⚠️ | Code inspection |
| X86_FRM2 | syscall_number() returns RAX | ⚠️ | Runtime (syscall) |
| X86_FRM3 | arg0-arg5 return RDI, RSI, RDX, R10, R8, R9 | ⚠️ | Runtime (syscall) |
| X86_FRM4 | set_return() modifies RAX for return value | ⚠️ | Runtime (syscall) |

### enter_user_mode (task.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| X86_USR1 | enter_user_mode sets RSP to user stack | ⚠️ | Runtime (userspace) |
| X86_USR2 | enter_user_mode sets RCX to entry point | ⚠️ | Runtime (userspace) |
| X86_USR3 | enter_user_mode sets R11 to RFLAGS with IF=1 | ⚠️ | Runtime (userspace) |
| X86_USR4 | enter_user_mode executes swapgs before sysretq | ⚠️ | Runtime (userspace) |
| X86_USR5 | enter_user_mode transitions to Ring 3 via sysretq | ⚠️ | Runtime (userspace) |

### Userspace Entry (entry.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| X86_ENT1 | _start clears RBP (frame pointer) | ⚠️ | Runtime (stack trace) |
| X86_ENT2 | _start passes RSP as first argument | ⚠️ | Runtime (args) |
| X86_ENT3 | _start aligns stack to 16 bytes | ⚠️ | Runtime (ABI) |
| X86_ENT4 | _start calls _start_rust | ⚠️ | Runtime (init) |

### Group 14 Summary
- **GDT/TSS**: 6/6 behaviors documented ⚠️ (runtime verified)
- **PIT Timer**: 3/3 behaviors documented ⚠️ (runtime verified)
- **Context Switch**: 7/7 behaviors documented ⚠️ (runtime verified)
- **Syscall Entry/Exit**: 5/5 behaviors documented ⚠️ (runtime verified)
- **SyscallFrame**: 4/4 behaviors documented ⚠️ (code inspection)
- **enter_user_mode**: 5/5 behaviors documented ⚠️ (runtime verified)
- **Userspace Entry**: 4/4 behaviors documented ⚠️ (runtime verified)
- **Total**: 34/34 behaviors documented ⚠️

> **Note:** All x86_64 behaviors are runtime-verified. Unit tests are not practical
> for hardware-specific assembly. Verification is through successful kernel boot,
> shell interaction, and syscall execution on x86_64 QEMU.

---

## Group 15: PCI Bus & Discovery — Behavior Inventory

TEAM_373: Added PCI discovery behaviors

### PCI Bus (`crates/pci/src/lib.rs`)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| P1 | pci_allocate returns aligned address | ❌ | |
| P2 | pci_allocate returns None when pool exhausted | ❌ | |
| P3 | pci_allocate is thread-safe (atomic) | ❌ | |
| P4 | find_virtio_device identifies correct DeviceType | ❌ | |
| P5 | find_virtio_device allocates BARs for found device | ❌ | |

### Group 15 Summary
- **PCI Bus**: 0/5 behaviors tested ❌

---

## Group 16: NVMe Storage Driver — Behavior Inventory

TEAM_373: Added NVMe driver behaviors (stub)

### NVMe Driver (`crates/drivers/nvme/src/lib.rs`)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| N1 | block_size returns 512 | ❌ | |
| N2 | size_in_blocks returns 0 (stub) | ❌ | |
| N3 | read_blocks returns Err(NotReady) (stub) | ❌ | |
| N4 | write_blocks returns Err(NotReady) (stub) | ❌ | |

### Group 16 Summary
- **NVMe Driver**: 0/4 behaviors tested ❌

---

## Group 17: VirtIO GPU Driver — Behavior Inventory

TEAM_373: Added VirtIO GPU wrapper behaviors

### VirtIO GPU (`crates/drivers/virtio-gpu/src/lib.rs`)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| VG1 | new() initializes with correct resolution | ❌ | |
| VG2 | new() sets up and clears framebuffer | ❌ | |
| VG3 | flush() calls inner flush | ❌ | |
| VG4 | Display adapter draws to correct offsets | ❌ | |
| VG5 | FramebufferGpu handles BGR vs RGB (Limine) | ❌ | |

### Group 17 Summary
- **VirtIO GPU**: 0/5 behaviors tested ❌

---

## Group 18: XHCI USB Controller — Behavior Inventory

TEAM_373: Added XHCI driver behaviors (stub)

### XHCI Driver (`crates/drivers/xhci/src/lib.rs`)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| X1 | poll returns false (stub) | ❌ | |
| X2 | read_char returns None (stub) | ❌ | |
| X3 | poll_event returns None (stub) | ❌ | |

### Group 18 Summary
- **XHCI Driver**: 0/3 behaviors tested ❌

---

## Updated Overall Summary (TEAM_373)

| Group | Module | Behaviors | Tested | Gap |
|-------|--------|-----------|--------|-----|
| 1-14 | Previous Groups | 202 | 200 | ⚠️ |
| 15 | PCI Bus | 5 | 0 | ❌ |
| 16 | NVMe Driver | 4 | 0 | ❌ |
| 17 | VirtIO GPU | 5 | 0 | ❌ |
| 18 | XHCI Driver | 3 | 0 | ❌ |
| **Total** | | **219** | **200** | **19 new gaps** ❌ |

---

## Maintenance Log

### TEAM_373: Structural Testing Gap Resolution (2026-01-10)

Added Groups 15-18 to document driver behaviors and identify testing gaps:
- PCI Bus & Discovery (5 behaviors)
- NVMe Storage Driver (4 behaviors)
- VirtIO GPU Driver (5 behaviors)
- XHCI USB Controller (3 behaviors)

Updated file headers for consistency across all groups. 19 new testing gaps identified.


### TEAM_303: x86_64 Behavior Documentation (2026-01-08)

Added Group 14 (x86_64 Architecture) with 34 behaviors covering:
- GDT/TSS initialization and configuration
- PIT timer setup and IRQ generation
- Context switching with full register save/restore
- Syscall entry/exit via syscall/sysretq
- SyscallFrame layout verification
- enter_user_mode Ring 3 transition
- Userspace _start naked function

All behaviors are runtime-verified through successful x86_64 kernel boot and shell execution.

### TEAM_157: Crate Reorganization Update (2026-01-06)

Updated all file paths to reflect new crate structure:
- `levitate-utils/` → `crates/utils/` (`los_utils`)
- `levitate-hal/` → `crates/hal/` (`los_hal`)
- `levitate-terminal/` → `crates/term/` (`los_term`)
- `levitate-virtio/` → `crates/virtio/` (`los_virtio`)
- `levitate-pci/` → `crates/pci/` (`los_pci`)
- `levitate-gpu/` → `crates/gpu/` (`los_gpu`)
- `levitate-error/` → `crates/error/` (`los_error`)

Fixed duplicate table headers in summary sections.

### TEAM_158: Traceability Sweep (2026-01-06)

Added behavior ID traceability comments per Rules 4-5:
- [SL1-SL8] → `crates/hal/src/allocator/intrusive_list.rs`
- [SP1-SP8] → `crates/hal/src/allocator/slab/page.rs`
- [SC1-SC3] → `crates/hal/src/allocator/slab/cache.rs`
- [SA1-SA4] → `crates/hal/src/allocator/slab/mod.rs`
- [B1-B11] → `crates/hal/src/allocator/buddy.rs`

**34 behaviors now have full source+test traceability.**

Runtime-only behaviors (NET, TERM, MT, SYS, GPU, PROC, SH, SHELL) remain
acceptable without source traceability per Rule 7 (Test Abstraction Levels).

