# levitate-kernel

The main kernel binary for LevitateOS — an AArch64 operating system kernel targeting the QEMU `virt` machine and Pixel 6 hardware.

## Purpose

This crate is the **entry point** for the entire operating system. It orchestrates hardware initialization, sets up memory management, initializes device drivers, and runs the main kernel loop.

## Architecture

```
kernel/src/
├── main.rs         # Entry point, boot sequence, kmain()
├── exceptions.rs   # Exception vector table, IRQ handling
├── block.rs        # VirtIO block device driver
├── cursor.rs       # Mouse cursor state management
├── gpu.rs          # VirtIO GPU driver (embedded-graphics integration)
├── input.rs        # VirtIO input device driver (keyboard/tablet)
├── virtio.rs       # VirtIO MMIO transport and HAL implementation
├── fs/
│   ├── mod.rs      # Virtual filesystem layer
│   ├── fat.rs      # FAT32 filesystem (boot partition)
│   ├── ext4.rs     # ext4 filesystem (root partition, read-only)
│   └── initramfs.rs # CPIO initramfs support (re-exports from levitate-utils)
└── memory/
    └── mod.rs      # Physical memory management, frame allocator integration
```

## Boot Sequence

1. **Assembly Entry (`_start`)**: Disables interrupts, enables FP/SIMD, zeroes BSS, saves boot registers (DTB address in x0), sets up early page tables, enables MMU, jumps to `kmain`.

2. **Heap Initialization**: Uses `linked_list_allocator` with heap bounds from linker script.

3. **MMU Re-initialization**: Sets up higher-half kernel mappings with 2MB block optimization.

4. **Core Drivers**: Exception vectors, UART console, GIC (v2/v3 auto-detected via FDT).

5. **Physical Memory**: Buddy allocator initialized from DTB memory map.

6. **Timer**: AArch64 generic timer configured for periodic interrupts.

7. **VirtIO Devices**: Scans MMIO bus, initializes GPU, Input, and Block devices.

8. **Filesystem**: Mounts FAT32 boot partition, parses initramfs from DTB.

9. **Main Loop**: Polls input, echoes UART, draws cursor.

## Key Features

- **Higher-Half Kernel**: Virtual addresses start at `0xFFFF_8000_0000_0000`
- **GICv2/GICv3 Support**: Auto-detected via FDT for Pixel 6 compatibility
- **VirtIO 1.0**: GPU, Input, Block devices via MMIO transport
- **Verbose Mode**: `--features verbose` enables boot messages for golden file testing (Rule 4: Silence is Golden)

## Dependencies

| Crate | Purpose |
|-------|---------|
| `levitate-hal` | Hardware abstraction (GIC, Timer, MMU, UART, Console) |
| `levitate-utils` | Core utilities (Spinlock, RingBuffer, CPIO parser, hex formatting) |
| `virtio-drivers` | VirtIO device drivers (v0.12.0, MMIO transport) |
| `embedded-graphics` | 2D graphics primitives |
| `embedded-sdmmc` | FAT32 filesystem |
| `ext4-view` | Read-only ext4 filesystem |
| `linked_list_allocator` | Kernel heap allocator |

## Building

```bash
# Build for AArch64 (requires nightly)
cargo build -Z build-std=core,alloc --release --target aarch64-unknown-none -p levitate-kernel

# Build with verbose boot messages
cargo build -Z build-std=core,alloc --release --target aarch64-unknown-none -p levitate-kernel --features verbose
```

## Running

```bash
# Via xtask (recommended)
cargo xtask run

# Direct QEMU
qemu-system-aarch64 -M virt -cpu cortex-a53 -m 512M \
  -kernel target/aarch64-unknown-none/release/levitate-kernel \
  -device virtio-gpu-device -serial stdio
```

## IRQ Handling

IRQs are dispatched via `levitate_hal::gic::dispatch()` which calls registered `InterruptHandler` trait implementations:

| IRQ | Source | Handler |
|-----|--------|---------|
| 27 | Virtual Timer | Reloads timer for next tick |
| 33 | PL011 UART | Buffers received bytes |

## DTB Detection

The kernel discovers hardware configuration via Device Tree Blob (DTB):

1. **x0 register**: Bootloader-provided DTB address (standard Linux boot protocol)
2. **Memory scan**: Fallback for QEMU ELF boot — scans `0x4000_0000..0x4900_0000` for DTB magic (`0xD00DFEED`)

## Memory Layout

See `linker.ld` for details. Key regions:

| Region | Physical Address | Virtual Address |
|--------|------------------|-----------------|
| Kernel Start | `0x4008_0000` | `0xFFFF_8000_4008_0000` |
| Heap | After kernel | Higher-half mapped |
| Device MMIO | `0x0000_0000..0x4000_0000` | Identity mapped |
