# TEAM_029: VirtIO Block Driver Design Questions

- **Feature**: VirtIO Block Driver
- **Status**: Open

## Q1: Polling vs. Interrupts for Block I/O
The initial implementation will use synchronous polling for block I/O. This is simple and avoids complex interrupt state management for the first disk access. Is this acceptable for Phase 4, or is interrupt-driven I/O required immediately?

- **Hypothesis**: Polling is fine for early development (booting, loading small files).
- **Recommendation**: Proceed with polling, document the need for interrupt-driven I/O in Phase 5.

## Q2: Multiple Block Devices
Should we support multiple VirtIO block devices, or just assume the first one found is the boot/root disk?

- **Recommendation**: Assume the first one for now. Later we can implement a proper device discovery and mounting system.

## Q3: Buffer Alignment and Safety
`virtio-drivers` requires buffers to be DMA-capable. Our current `VirtioHal` uses the global allocator which returns 4KB-aligned pages. For arbitrary buffers passed to `read_block`, we might need to copy into a temporary DMA-aligned buffer if the user-provided buffer isn't suitable. Should we implement this bounce-buffer logic now?

- **Recommendation**: For the initial version, require that callers provide a buffer that meets the HAL's requirements, or just use the heap-allocated buffers for everything.
