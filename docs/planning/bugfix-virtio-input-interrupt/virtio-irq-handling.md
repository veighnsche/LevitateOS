# VirtIO MMIO Interrupt Handling

**Created**: 2026-01-07
**Context**: Bugfix `virtio-input-interrupt`

## Key Patterns

### 1. IRQ Number Discovery
QEMU `virt` machine maps VirtIO MMIO IRQs sequentially:
- Base IRQ: **48**
- Formula: `IRQ = 48 + slot_index`

When scanning `VIRTIO_MMIO` slots (0..31), you must propagate the slot index to the device driver initialization to correctly register the interrupt.

### 2. GIC Handler Registry Limits
`los_hal::gic::MAX_HANDLERS` must be large enough to accommodate all potential VirtIO slots.
- Fixed Handlers: Timer (0), UART (1)
- VirtIO Handlers: Indices 2..33 (corresponding to slots 0..31)
- **Minimum Required**: 34

> **Gotcha**: If `MAX_HANDLERS` is too small, `register_handler` may silently fail or `dispatch` will treat valid IRQs as unhandled.

### 3. Immediate Signaling Pattern
For devices that need to interrupt blocked processes (like keyboards):
1. **Poll** inside the ISR.
2. **Detect** the critical event (e.g., Ctrl+C).
3. **Signal** immediately (`signal_foreground_process`), do not wait for a reader.

This ensures responsiveness even when the foreground process is blocked in `sys_pause()` or similar.
