# levitate-gpu

VirtIO GPU library for LevitateOS — provides hardware abstraction and `embedded-graphics` integration for the VirtIO GPU device.

## Purpose

This crate encapsulates the **low-level VirtIO GPU driver** logic, separating it from the kernel's platform-specific code. It provides a `Display` wrapper that implements the `DrawTarget` trait, allowing the kernel and other libraries to use standard drawing primitives.

## Architecture

```
levitate-gpu/src/
├── lib.rs          # Crate root, error types, and core exports
└── gpu.rs          # VirtIO GPU driver implementation and Display wrapper
```

## Key Components

### GpuState

Manages the life-cycle of the VirtIO GPU device:
- **Initialization**: Maps the VirtIO MMIO transport to a `VirtIOGpu` instance.
- **Framebuffer**: Manages access to the mapped framebuffer memory.
- **Flushing**: High-level `flush()` command to commit changes to the host display.
- **Telemetry**: Tracks flush statistics and errors for debugging.

### Display

A wrapper around `&mut GpuState` that enables graphical output:
- **DrawTarget**: Implements the `embedded-graphics` trait for 2D drawing.
- **OriginDimensions**: Provides screen resolution for layout calculations.
- **Safe Rendering**: Ensures all drawing operations stay within the bounds of the framebuffer.

## Usage

```rust
use levitate_gpu::{GpuState, Display};
use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle};

// Initialized via VirtIO transport
let mut gpu_state = GpuState::new(transport).expect("GpuInit");
let mut display = Display::new(&mut gpu_state);

// Draw something
Rectangle::new(Point::new(10, 10), Size::new(50, 50))
    .into_styled(PrimitiveStyle::with_fill(Rgb888::RED))
    .draw(&mut display)
    .ok();

// Commit to hardware
gpu_state.flush();
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `levitate-hal` | Provides `VirtioHal` and `StaticMmioTransport` |
| `virtio-drivers` | Underlying VirtIO protocol implementation (v0.12) |
| `embedded-graphics` | Standard 2D graphics traits |

## Integration with LevitateOS

The kernel uses this crate in `kernel/src/gpu.rs` as a thin wrapper around a global `GpuState`. The `levitate-terminal` crate also accepts a `Display` as its `DrawTarget` for rendering text.
