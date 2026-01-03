# Phase 2: Design - VirtIO Block Driver

## Proposed Solution
- **Driver Wrapper**: Create `kernel/src/block.rs` to house the `BlockDevice` abstraction and the `VirtIOBlk` wrapper.
- **Initialization**:
    - Update `virtio::init` in `kernel/src/virtio.rs` to recognize `DeviceType::Block`.
    - Pass the `MmioTransport` to `crate::block::init(transport)`.
- **Concurrency**: Use a `Spinlock` (or `IrqSafeLock`) to protect the block device instance, as disk I/O should be thread-safe.
- **DMA Management**: Use the existing `VirtioHal` from `virtio.rs`.

## API Design
The `block` module will expose a simple interface for now:
```rust
pub fn read_block(block_id: usize, buf: &mut [u8]);
pub fn write_block(block_id: usize, buf: &[u8]);
```
Internally, these will call `virtio_drivers::device::blk::VirtIOBlk::read_block` and `write_block`.

### Data Model
- `BLOCK_SIZE`: 512 bytes (standard for VirtIO block).
- `static BLOCK_DEVICE`: `Spinlock<Option<VirtIOBlk<VirtioHal, MmioTransport>>>`.

## Behavioral Decisions
- **Polling vs. Interrupts**: For the initial implementation, we will use **polling** (synchronous I/O). VirtIO-Blk supports polling via `virtio-drivers`. This matches the simplicity goal (Rule 8 - Simplicity > Perfection).
- **Error Handling**: If I/O fails, the kernel will currently `panic!` or log a critical error, as we don't have a robust error recovery path for the disk yet.
- **Buffer Alignment**: `virtio-drivers` requires buffers to be DMA-capable. Our `VirtioHal` handles this by allocating from the heap and mapping to physical addresses.

## Design Alternatives Considered
- **Async I/O**: Considered using `async/await` for disk I/O, but LevitateOS doesn't have an async runtime yet. Synchronous polling is easier to implement and sufficient for early stages.

## Open Questions
- **Q1**: Should we support multiple block devices?
    - *Recommendation*: For now, only the first detected block device will be used as the root disk.
- **Q2**: How should we handle the interrupt if we switch away from polling later?
    - *Response*: We will need to register a handler in the GIC for the VirtIO IRQ associated with the block device. Each VirtIO slot might have its own IRQ or they might share one.
- **Q3**: What happens if the buffer size passed to `read_block` is not exactly 512 bytes?
    - *Decision*: The API will enforce 512-byte slices or return an error/panic.
