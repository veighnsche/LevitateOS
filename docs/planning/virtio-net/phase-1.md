# Phase 1: Discovery — VirtIO Network Driver

**Feature**: VirtIO Net Driver (Phase 6, Task 6.1)
**Team**: TEAM_057
**Parent**: `docs/planning/virtio-ecosystem-phase6/`

---

## Feature Summary

**Short Description**: Implement a VirtIO network device driver to enable basic Ethernet packet transmission and reception in LevitateOS.

**Problem Statement**: LevitateOS currently cannot communicate over a network. All I/O is limited to serial console (UART), block storage, and GPU display. A network driver is the foundation for any future networking capabilities.

**Who Benefits**:
- Kernel developers (network debugging, remote logging)
- Future userspace (TCP/IP stack, internet connectivity)
- Development workflow (network boot, remote testing)

---

## Success Criteria

- [ ] VirtIO-net device detected during MMIO bus scan
- [ ] MAC address successfully read from device configuration
- [ ] Driver can send a raw Ethernet frame (polling mode)
- [ ] Driver can receive raw Ethernet frames (polling mode)
- [ ] No panics or memory corruption during operation
- [ ] Follows existing driver patterns (like `block.rs`)

---

## Current State Analysis

### How the system works today

- **VirtIO Infrastructure**: Complete. `kernel/src/virtio.rs` scans MMIO bus `0x0a000000`–`0x0a100000` and dispatches to device-specific drivers.
- **Existing Drivers**: GPU, Input, Block all use same pattern with `StaticMmioTransport`.
- **QEMU Config**: `run.sh` already includes `-device virtio-net-device,netdev=net0 -netdev user,id=net0` — the device is present but ignored.

### Current behavior

When kernel boots, the MMIO scan encounters the virtio-net device but the match arm falls through to `_ => {}`, silently ignoring it.

### Workarounds

None. Network functionality is completely unavailable.

---

## Codebase Reconnaissance

### Files to touch

| File | Action | Purpose |
|------|--------|---------|
| `kernel/src/net.rs` | **CREATE** | Network driver module |
| `kernel/src/virtio.rs` | MODIFY | Add `DeviceType::Network` match arm |
| `kernel/src/main.rs` | MODIFY | Add `mod net;` declaration |

### APIs involved

**From `virtio-drivers` v0.12**:
```rust
// High-level driver with buffer management
virtio_drivers::device::net::VirtIONet<H, T, QUEUE_SIZE>
  ::new(transport, buf_len) -> Result<Self>
  ::mac_address() -> [u8; 6]
  ::can_send() -> bool
  ::can_recv() -> bool
  ::send(tx_buf: TxBuffer) -> Result
  ::receive() -> Result<RxBuffer>
  ::recycle_rx_buffer(rx_buf: RxBuffer) -> Result

// Low-level raw API (alternative)
virtio_drivers::device::net::VirtIONetRaw<H, T, QUEUE_SIZE>
```

**From LevitateOS**:
```rust
crate::virtio::{StaticMmioTransport, VirtioHal}
levitate_utils::Spinlock
```

### Existing patterns to follow

`kernel/src/block.rs`:
```rust
static BLOCK_DEVICE: Spinlock<Option<VirtIOBlk<VirtioHal, StaticMmioTransport>>> =
    Spinlock::new(None);

pub fn init(transport: StaticMmioTransport) {
    match VirtIOBlk::<VirtioHal, StaticMmioTransport>::new(transport) {
        Ok(blk) => { *BLOCK_DEVICE.lock() = Some(blk); }
        Err(e) => { crate::println!("Failed: {:?}", e); }
    }
}
```

### Tests/snapshots that may be impacted

- Golden file tests (if `verbose` feature enabled) — new boot messages
- No existing network tests to break

---

## Constraints

- **No async**: Must work with polling, no async runtime
- **No allocator changes**: Use existing global allocator
- **Memory safety**: All DMA handled by `VirtioHal` (already implemented)
- **Queue size**: Reasonable default (16 or 32 entries)
- **Buffer size**: Must accommodate MTU (1500) + headers (~2048 bytes)

---

## Open Questions (Phase 1)

None — discovery phase reveals straightforward implementation path.

---

## Next Phase

Proceed to **Phase 2: Design** to define the API surface and behavioral contracts.
