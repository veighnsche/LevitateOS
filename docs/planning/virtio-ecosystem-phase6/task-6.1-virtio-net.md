# Task 6.1: VirtIO Net Driver

## Phase 1: Discovery

### Feature Summary
- **Feature**: VirtIO Network Device (`virtio-net`) Driver
- **Problem Statement**: LevitateOS cannot communicate over a network. A network driver is required for any future networking capabilities (TCP/IP stack, debugging via network, etc.)
- **Benefits**: Enables network communication, remote debugging, future internet connectivity

### Success Criteria
- [ ] `virtio-net` device detected during MMIO scan
- [ ] MAC address readable from device
- [ ] Can send raw Ethernet frames
- [ ] Can receive raw Ethernet frames (polling mode)

### Current State Analysis

**QEMU already passes virtio-net**:
```bash
# From run.sh
-device virtio-net-device,netdev=net0
-netdev user,id=net0
```

**Crate Support** (`virtio-drivers` v0.12):
```rust
// Available in virtio_drivers::device::net
pub struct VirtIONet<H: Hal, T: Transport, const QUEUE_SIZE: usize>
pub struct VirtIONetRaw<H: Hal, T: Transport, const QUEUE_SIZE: usize>
```

**Key APIs**:
- `VirtIONet::new(transport, buf_len)` — Create driver with internal buffers
- `mac_address()` — Get 6-byte MAC
- `send(tx_buf: TxBuffer)` — Transmit packet
- `receive()` -> `Result<RxBuffer>` — Receive packet (non-blocking)
- `can_send()` / `can_recv()` — Check availability

### Codebase Reconnaissance

**Files to modify**:
| File | Change |
|------|--------|
| `kernel/src/net.rs` | **NEW** — Network driver wrapper |
| `kernel/src/virtio.rs` | Add `DeviceType::Network` match arm |
| `kernel/src/main.rs` | Add `mod net;` |

**Pattern to follow** (from `block.rs`):
```rust
use crate::virtio::{StaticMmioTransport, VirtioHal};
use levitate_utils::Spinlock;
use virtio_drivers::device::net::VirtIONet;

const QUEUE_SIZE: usize = 16;
const RX_BUF_LEN: usize = 2048;  // MTU + headers

static NET_DEVICE: Spinlock<Option<VirtIONet<VirtioHal, StaticMmioTransport, QUEUE_SIZE>>> =
    Spinlock::new(None);

pub fn init(transport: StaticMmioTransport) {
    match VirtIONet::<VirtioHal, StaticMmioTransport, QUEUE_SIZE>::new(transport, RX_BUF_LEN) {
        Ok(net) => {
            crate::verbose!("VirtIO Net: MAC={:02x?}", net.mac_address());
            *NET_DEVICE.lock() = Some(net);
        }
        Err(e) => crate::println!("VirtIO Net init failed: {:?}", e),
    }
}
```

---

## Phase 2: Implementation Plan

### Step 1: Create `kernel/src/net.rs`

```rust
//! VirtIO Network Device Driver
//!
//! TEAM_XXX: Initial VirtIO Net implementation for Phase 6

use crate::virtio::{StaticMmioTransport, VirtioHal};
use levitate_utils::Spinlock;
use virtio_drivers::device::net::VirtIONet;

const QUEUE_SIZE: usize = 16;
const RX_BUFFER_LEN: usize = 2048;

static NET_DEVICE: Spinlock<Option<VirtIONet<VirtioHal, StaticMmioTransport, QUEUE_SIZE>>> =
    Spinlock::new(None);

pub fn init(transport: StaticMmioTransport) {
    crate::verbose!("Initializing Network device...");
    match VirtIONet::<VirtioHal, StaticMmioTransport, QUEUE_SIZE>::new(transport, RX_BUFFER_LEN) {
        Ok(net) => {
            let mac = net.mac_address();
            crate::verbose!(
                "VirtIO Net: MAC={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
            );
            *NET_DEVICE.lock() = Some(net);
        }
        Err(e) => crate::println!("Failed to init VirtIO Net: {:?}", e),
    }
}

pub fn mac_address() -> Option<[u8; 6]> {
    NET_DEVICE.lock().as_ref().map(|net| net.mac_address())
}

pub fn can_send() -> bool {
    NET_DEVICE.lock().as_ref().map_or(false, |net| net.can_send())
}

pub fn can_recv() -> bool {
    NET_DEVICE.lock().as_ref().map_or(false, |net| net.can_recv())
}
```

### Step 2: Update `kernel/src/virtio.rs`

Add match arm in `init()`:
```rust
virtio_drivers::transport::DeviceType::Network => {
    crate::net::init(transport);
}
```

### Step 3: Update `kernel/src/main.rs`

Add module declaration:
```rust
mod net;
```

### Step 4: Verification

Run kernel and check for:
```
VirtIO Net: MAC=52:54:00:12:34:56
```

---

## Phase 3: Testing

### Manual Test
1. Build and run kernel
2. Verify MAC address is printed
3. Check no panics occur

### Future Tests (Post-Implementation)
- Send ARP request, verify it appears in host `tcpdump`
- Receive DHCP offer (if using user networking)

---

## Dependencies

- `virtio-drivers` v0.12 ✅ (already in Cargo.toml)
- No new dependencies required

---

## Estimated Effort

| Task | Time |
|------|------|
| Create net.rs | 30 min |
| Update virtio.rs | 10 min |
| Update main.rs | 5 min |
| Testing | 15 min |
| **Total** | **~1 hour** |
