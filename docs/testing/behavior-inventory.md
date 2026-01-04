# Behavior Inventory for Unit Testing

TEAM_030: Behavior-driven test inventory

---

## File Groups

### Group 1: Core Primitives
- `levitate-utils/src/lib.rs` (Spinlock, RingBuffer)

### Group 2: Interrupt & Synchronization
- `levitate-hal/src/interrupts.rs`
- `levitate-hal/src/lib.rs` (IrqSafeLock)
- `levitate-hal/src/gic.rs`

### Group 3: Serial I/O
- `levitate-hal/src/uart_pl011.rs`
- `levitate-hal/src/console.rs`

### Group 4: Memory Management
- `levitate-hal/src/mmu.rs`

### Group 5: Timer
- `levitate-hal/src/timer.rs`

### Group 6: Kernel Drivers (runtime-only, no unit tests)
- `kernel/src/block.rs`
- `kernel/src/gpu.rs`
- `kernel/src/input.rs`
- `kernel/src/virtio.rs`

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

### Group 2 Summary
- **interrupts**: 6/6 behaviors tested ✅
- **IrqSafeLock**: 4/4 behaviors tested ✅
- **GIC**: 7/7 behaviors tested ✅

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

### Group 4 Summary
- **MMU**: 22/22 behaviors tested ✅

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
- `levitate-hal/src/fdt.rs` (FDT parsing)
- `kernel/src/fs/initramfs.rs` (CPIO parser)

### FDT Module (fdt.rs)

| ID | Behavior | Tested? | Test |
|----|----------|---------|------|
| FD1 | Invalid DTB header returns InvalidHeader error | ✅ | `test_fdt_invalid_header` |
| FD2 | Missing initrd properties returns InitrdMissing error | ✅ | `test_fdt_error_types` |
| FD3 | 32-bit initrd-start is parsed correctly | ✅ | `test_fdt_byte_parsing` |
| FD4 | 64-bit initrd-start is parsed correctly | ✅ | `test_fdt_byte_parsing` |
| FD5 | Both start and end properties must exist | ✅ | `test_fdt_error_types` |
| FD6 | Big-endian byte order is handled | ✅ | `test_fdt_byte_parsing` |

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
### Group 6 Summary
- **FDT**: 6/6 behaviors tested ✅
- **CPIO**: 10/10 behaviors tested ✅
- **Total**: 16/16 behaviors tested ✅
- **Note**: TEAM_039 relocated CPIO to `levitate-utils/src/cpio.rs` - tests run via `cargo test --features std`.

---

## Updated Overall Summary (TEAM_039)

| Group | Module | Behaviors | Tested | Gap |
|-------|--------|-----------|--------|-----|
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
| 6 | FDT | 6 | 6 | ✅ |
| 6 | CPIO | 10 | 10 | ✅ |
| **Total** | | **83** | **83** | **0 gaps** ✅ |

