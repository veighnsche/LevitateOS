# Phase 2: Design — VirtIO Network Driver

**Feature**: VirtIO Net Driver (Phase 6, Task 6.1)
**Team**: TEAM_057
**Depends on**: `phase-1.md`

---

## Proposed Solution

### High-Level Description

Create a `net.rs` module that wraps `virtio_drivers::device::net::VirtIONet` following the exact same pattern as `block.rs`. The driver will:

1. Initialize when VirtIO MMIO scan discovers a Network device
2. Store the driver instance in a global `Spinlock<Option<...>>`
3. Provide public functions for MAC address query and packet TX/RX

### User-Facing Behavior

On boot (with `verbose` feature):
```
Initializing Network device...
VirtIO Net: MAC=52:54:00:12:34:56
```

### System Behavior

```
MMIO Scan → DeviceType::Network detected
         → net::init(transport) called
         → VirtIONet::new() initializes virtqueues
         → Pre-allocates RX buffers (QUEUE_SIZE count)
         → Device ready for polling
```

---

## API Design

### Module: `kernel/src/net.rs`

```rust
/// Initialize network device from VirtIO transport
pub fn init(transport: StaticMmioTransport)

/// Get MAC address (returns None if device not initialized)
pub fn mac_address() -> Option<[u8; 6]>

/// Check if device can accept a packet for transmission
pub fn can_send() -> bool

/// Check if there's a received packet waiting
pub fn can_recv() -> bool

/// Send raw Ethernet frame (blocking until complete)
pub fn send(data: &[u8]) -> Result<(), NetError>

/// Receive raw Ethernet frame (non-blocking, returns None if no packet)
pub fn receive() -> Option<Vec<u8>>
```

### Error Type

```rust
#[derive(Debug)]
pub enum NetError {
    NotInitialized,
    DeviceBusy,
    BufferTooSmall,
    TransmitFailed,
}
```

### Constants

```rust
const QUEUE_SIZE: usize = 16;      // Number of virtqueue entries
const RX_BUFFER_LEN: usize = 2048; // MTU (1500) + headers + margin
```

---

## Behavioral Decisions

### Q1: What happens if `send()` is called when device not initialized?
**Decision**: Return `Err(NetError::NotInitialized)`.

### Q2: What happens if `send()` is called when queue is full?
**Decision**: Return `Err(NetError::DeviceBusy)`. Caller should check `can_send()` first.

### Q3: What happens if `receive()` is called with no packet available?
**Decision**: Return `None` (non-blocking). Caller should poll `can_recv()`.

### Q4: Should we handle interrupts or use polling?
**Decision**: Polling only for Phase 6. Interrupt support is future work.

### Q5: What queue size to use?
**Decision**: 16 entries. Matches typical embedded use, balances memory vs throughput.

### Q6: How to handle RX buffer lifecycle?
**Decision**: Use `VirtIONet` (not `VirtIONetRaw`) which manages buffers internally. The `receive()` function will copy data out and recycle the buffer automatically.

---

## Data Flow

### Transmit Path
```
User calls send(data)
  → Acquire NET_DEVICE lock
  → Check can_send()
  → Create TxBuffer from data
  → Call net.send(tx_buf)
  → Release lock
  → Return Ok(())
```

### Receive Path
```
User calls receive()
  → Acquire NET_DEVICE lock
  → Check can_recv()
  → Call net.receive() → RxBuffer
  → Copy packet data to Vec<u8>
  → Call net.recycle_rx_buffer()
  → Release lock
  → Return Some(data)
```

---

## Design Alternatives Considered

### Alternative 1: Use `VirtIONetRaw` instead of `VirtIONet`
- **Pro**: More control over buffer management
- **Con**: Must manage RX buffer pool manually
- **Decision**: Rejected. `VirtIONet` handles this correctly and reduces code.

### Alternative 2: Async/interrupt-based receive
- **Pro**: More efficient, no polling overhead
- **Con**: Requires interrupt handler registration, more complex
- **Decision**: Deferred to future phase. Polling sufficient for Phase 6.

### Alternative 3: Zero-copy API (return reference to buffer)
- **Pro**: Better performance
- **Con**: Complex lifetime management, lock held during use
- **Decision**: Rejected. Copy-out is simpler and safer for initial implementation.

---

## Open Questions

**None** — All behavioral decisions resolved above.

---

## Next Phase

Proceed to **Phase 3: Implementation** with concrete UoW tasks.
