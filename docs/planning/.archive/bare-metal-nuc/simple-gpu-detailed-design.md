# SimpleGPU Detailed Design

This document outlines the implementation plan for the `simple-gpu` driver, which provides generic UEFI GOP (Graphics Output Protocol) support for LevitateOS on bare metal.

## Architectural Inspiration (Theseus)

The `simple-gpu` driver will follow Theseus's pattern of a "Memory-Mapped Framebuffer" with specific focus on performance:

1.  **PAT/WC Support (Future)**: Like Theseus, we aim to use the **Page Attribute Table (PAT)** to enable **Write-Combining (WC)** on x86_64. This allows the CPU to batch small writes to the framebuffer into larger bursts, significantly increasing frame rates.
2.  **Generic Pixel Abstraction**: While currently focused on 32-bit RGB/BGR (standard for GOP), the driver will be designed to support multiple pixel formats.
3.  **Backbuffering**: To prevent flickering on bare metal (where direct MMIO can be slow), the driver will support a "Virtual Framebuffer" (backbuffer) that is flushed to the physical framebuffer.

## Component: `crates/drivers/simple-gpu`

### 1. GPU Structure
```rust
pub struct SimpleGpu {
    frontbuffer: DeviceRegion, // Real physical memory (MMIO)
    backbuffer: Option<Box<[u8]>>, // Optional virtual backbuffer
    width: u32,
    height: u32,
    pitch: u32,
    format: PixelFormat,
}
```

### 2. Implementation with `embedded-graphics`
The driver implements the `DrawTarget` trait from the `embedded-graphics` crate.
- `draw_iter`: Writes pixels to the backbuffer (if present) or directly to the frontbuffer.
- `flush`: Copies the dirty regions of the backbuffer to the frontbuffer using optimized `memcpy` (or similar).

### 3. Initialization
1.  **Boot**: Limine provides the `limine_framebuffer_response`.
2.  **Mapping**: The x86_64 HAL maps the physical address to a virtual range using the `DeviceMapper`.
3.  **Handover**: The kernel initializes `SimpleGpu` with the mapped region and dimensions.

## Non-Blocking Strategy (Rule 9)
Drawing is inherently synchronous, but **Flushing** can be optimized:
- For simple CLI/Console use: Direct frontbuffer writes (lowest latency).
- For GUI/Complex rendering: Backbuffer with batch flushing.

## Verification Plan
- **Mocking**: Run `simple-gpu` in `systest` with a virtual backbuffer and verify pixel values.
- **QEMU**: Verify the driver works as a fallback when `virtio-gpu` is disabled.
- **Bare Metal**: Deploy to NUC7 and verify "Booting LevitateOS..." text appears on screen via GOP.
