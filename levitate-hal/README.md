# levitate-hal

Hardware Abstraction Layer (HAL) for LevitateOS — provides low-level hardware interfaces for the AArch64 platform.

## Purpose

This crate abstracts **hardware-specific details** so the kernel can interact with devices through clean, safe Rust APIs. It handles CPU features, interrupt controllers, memory management, timers, and UART.

## Architecture

```
levitate-hal/src/
├── lib.rs          # Crate root, IrqSafeLock implementation
├── allocator/
│   ├── mod.rs      # Allocator module exports
│   ├── buddy.rs    # Buddy allocator for physical frames
│   └── page.rs     # Page descriptor (PhysPageFlags)
├── console.rs      # UART console (print!/println! macros, RX buffer)
├── fdt.rs          # Flattened Device Tree (FDT) parsing
├── gic.rs          # Generic Interrupt Controller (GICv2/GICv3)
├── interrupts.rs   # CPU interrupt enable/disable/restore
├── mmu.rs          # Memory Management Unit (page tables, mappings)
├── timer.rs        # AArch64 Generic Timer
└── uart_pl011.rs   # PL011 UART driver
```

## Key Components

### IrqSafeLock (`lib.rs`)

An interrupt-safe spinlock that **disables interrupts** before acquiring the lock to prevent deadlocks in IRQ handlers.

```rust
let lock = IrqSafeLock::new(data);
{
    let guard = lock.lock();  // Interrupts disabled
    // Critical section
}  // Interrupts restored on drop
```

### GIC (`gic.rs`)

Supports both **GICv2** (memory-mapped CPU interface) and **GICv3** (system register interface):

- Auto-detects version via FDT or PIDR2 register
- Handles interrupt acknowledge/EOI
- Typed `IrqId` enum with handler registration via `InterruptHandler` trait
- Spurious interrupt detection (IRQ 1020-1023)

```rust
// Register a handler
gic::register_handler(IrqId::VirtualTimer, &TIMER_HANDLER);

// In IRQ handler
let irq = gic::active_api().acknowledge();
gic::dispatch(irq);  // Calls registered handler
gic::active_api().end_interrupt(irq);
```

### MMU (`mmu.rs`)

AArch64 page table management with 4KB granule:

- **PageTable**: 512-entry table structure (4KB aligned)
- **PageFlags**: Bitflags for VALID, TABLE, AF, AP, PXN, UXN, etc.
- **2MB Block Mappings**: Optimized large mappings via `map_block_2mb()`
- **Address Translation**: `virt_to_phys()` / `phys_to_virt()` for higher-half kernel

Key constants:
| Constant | Value | Description |
|----------|-------|-------------|
| `KERNEL_VIRT_START` | `0xFFFF_8000_0000_0000` | Higher-half base |
| `KERNEL_PHYS_START` | `0x4008_0000` | Kernel load address |
| `KERNEL_PHYS_END` | `0x41F0_0000` | Heap end (synced with linker.ld) |

### Timer (`timer.rs`)

AArch64 Generic Timer with VHE detection:

- Automatically selects physical (`CNT*P*`) or virtual (`CNT*V*`) timer
- `Timer` trait for abstraction
- `uptime_seconds()` and `delay_cycles()` helpers

### Buddy Allocator (`allocator/`)

Physical frame allocator with coalescing:

- Orders 0-20 (4KB to 8GB blocks)
- `Page` descriptor tracks flags, order, and free-list pointers
- Integrates with MMU via `PageAllocator` trait

### FDT (`fdt.rs`)

Device Tree parsing utilities:

- `get_initrd_range()`: Extracts initramfs location from `/chosen` node
- `for_each_memory_region()`: Enumerates RAM regions
- `for_each_reserved_region()`: Enumerates reserved memory
- `find_node_by_compatible()`: Searches for device nodes

### UART (`uart_pl011.rs`, `console.rs`)

PL011 UART driver with interrupt-driven RX:

- Bitflag types for register access (`FlagFlags`, `ControlFlags`, etc.)
- `print!` / `println!` macros via `WRITER` static
- RX ring buffer for interrupt-based input

## Features

| Feature | Description |
|---------|-------------|
| `std` | Enables host-side unit tests (mocks for `no_std` hardware ops) |

## Building & Testing

```bash
# Build (no_std, for kernel)
cargo build -p levitate-hal

# Run unit tests (requires std feature)
cargo test -p levitate-hal --features std --target x86_64-unknown-linux-gnu
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `levitate-utils` | Spinlock, RingBuffer |
| `bitflags` | Type-safe hardware register flags |
| `fdt` | Device Tree parsing |

## Behavior IDs

This crate uses behavior IDs for traceability (see `docs/testing/behavior-inventory.md`):

- **L1-L4**: IrqSafeLock behaviors
- **I1-I6**: Interrupt control behaviors
- **M1-M22**: MMU behaviors (page flags, address translation)
- **G1-G9**: GIC behaviors (IRQ mapping, dispatch, spurious detection)
- **T1-T2**: Timer behaviors
- **U1-U8**: UART bitflag behaviors
- **FD1-FD10**: FDT parsing behaviors
