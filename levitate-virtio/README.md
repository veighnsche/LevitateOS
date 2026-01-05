# levitate-virtio

General-purpose VirtIO transport layer for LevitateOS.

## Overview

This crate provides the foundational VirtIO abstractions used by device-specific drivers:
- `VirtQueue` - Split virtqueue implementation
- `Transport` trait - Abstraction over MMIO/PCI transports
- Buffer management for DMA transfers

## Architecture

```
levitate-virtio           <- This crate (transport layer)
    │
    ├── levitate-virtio-gpu   <- GPU driver
    ├── levitate-virtio-blk   <- Block driver (future)
    └── levitate-virtio-net   <- Network driver (future)
```

## Usage

Device drivers implement the `Transport` trait and use `VirtQueue` for command/response handling.

## References

- [VirtIO 1.1 Specification](https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html)
- TEAM_098: Initial implementation
