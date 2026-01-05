# levitate-virtio-gpu

VirtIO GPU driver for LevitateOS with full protocol visibility.

## Overview

This crate provides a complete VirtIO GPU 2D driver implementation with:
- Explicit protocol structs matching VirtIO 1.1 Section 5.7
- State machine driver with full observability
- Async-first command handling
- RAII resource management

## Architecture

```
levitate-virtio-gpu/
├── protocol/           # VirtIO GPU protocol structs
│   ├── mod.rs         # Traits and common types
│   ├── commands.rs    # Command structs (requests)
│   └── formats.rs     # Pixel formats
├── driver.rs          # State machine driver
└── display.rs         # DrawTarget implementation
```

## Key Features

- **Full Visibility**: Every command/response is logged and traceable
- **Async-First**: Designed for async operation per user requirement
- **RAII Resources**: Resources are automatically unreferenced on drop
- **No Black Boxes**: Unlike `virtio-drivers`, all state is visible

## References

- [VirtIO 1.1 GPU Specification](https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html#x1-3310007)
- `docs/planning/virtio-gpu-scanout/VIRTIO_GPU_SPEC.md`
- TEAM_098: Initial implementation
