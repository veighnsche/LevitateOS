# Phase 3: Implementation — VirtIO Network Driver

**Feature**: VirtIO Net Driver (Phase 6, Task 6.1)
**Team**: TEAM_057
**Depends on**: `phase-2.md`

---

## Implementation Overview

Three files to modify/create. Work is small enough for a single UoW.

---

## Step 1: Create `kernel/src/net.rs`

### UoW 1: Implement Network Driver Module

**Goal**: Create complete network driver following `block.rs` pattern.

**File**: `kernel/src/net.rs`

```rust
//! VirtIO Network Device Driver
//!
//! TEAM_057: Initial VirtIO Net implementation for Phase 6

extern crate alloc;

use alloc::vec::Vec;
use crate::virtio::{StaticMmioTransport, VirtioHal};
use levitate_utils::Spinlock;
use virtio_drivers::device::net::VirtIONet;

const QUEUE_SIZE: usize = 16;
const RX_BUFFER_LEN: usize = 2048;

static NET_DEVICE: Spinlock<Option<VirtIONet<VirtioHal, StaticMmioTransport, QUEUE_SIZE>>> =
    Spinlock::new(None);

#[derive(Debug)]
pub enum NetError {
    NotInitialized,
    DeviceBusy,
    SendFailed,
    ReceiveFailed,
}

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

pub fn send(data: &[u8]) -> Result<(), NetError> {
    let mut dev = NET_DEVICE.lock();
    let net = dev.as_mut().ok_or(NetError::NotInitialized)?;
    
    if !net.can_send() {
        return Err(NetError::DeviceBusy);
    }
    
    let mut tx_buf = net.new_tx_buffer(data.len());
    tx_buf.packet_mut().copy_from_slice(data);
    
    net.send(tx_buf).map_err(|_| NetError::SendFailed)
}

pub fn receive() -> Option<Vec<u8>> {
    let mut dev = NET_DEVICE.lock();
    let net = dev.as_mut()?;
    
    if !net.can_recv() {
        return None;
    }
    
    match net.receive() {
        Ok(rx_buf) => {
            let data = rx_buf.packet().to_vec();
            let _ = net.recycle_rx_buffer(rx_buf);
            Some(data)
        }
        Err(_) => None,
    }
}
```

---

## Step 2: Update `kernel/src/virtio.rs`

### UoW 2: Add Network Device Match Arm

**Goal**: Route Network devices to `net::init()`.

**Location**: In `init()` function, inside the `match device_type` block.

**Add after `DeviceType::Block` arm** (~line 97-99):

```rust
virtio_drivers::transport::DeviceType::Network => {
    crate::net::init(transport);
}
```

---

## Step 3: Update `kernel/src/main.rs`

### UoW 3: Declare Network Module

**Goal**: Add module declaration.

**Location**: After `mod input;` (~line 30).

**Add**:
```rust
mod net;
```

---

## Verification Steps

After implementation:

1. **Build**: `cargo build --release`
2. **Run**: `./run.sh`
3. **Expected output** (with verbose):
   ```
   Initializing Network device...
   VirtIO Net: MAC=52:54:00:12:34:56
   ```
4. **Verify no panics** during boot and main loop

---

## Test Commands

```bash
# Build
cargo build --release

# Run with verbose to see init messages
cargo build --release --features verbose
./run.sh

# Check MAC was read (should appear in output)
# QEMU user networking assigns MAC 52:54:00:XX:XX:XX
```

---

## Estimated Time

| Task | Time |
|------|------|
| Create net.rs | 15 min |
| Update virtio.rs | 5 min |
| Update main.rs | 2 min |
| Build & test | 10 min |
| **Total** | **~30 min** |

---

## Checklist

- [ ] `kernel/src/net.rs` created
- [ ] `kernel/src/virtio.rs` updated with Network match arm
- [ ] `kernel/src/main.rs` has `mod net;`
- [ ] Project builds without errors
- [ ] Network device detected on boot
- [ ] MAC address printed correctly
