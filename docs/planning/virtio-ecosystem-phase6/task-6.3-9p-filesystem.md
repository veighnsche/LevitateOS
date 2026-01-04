# Task 6.3: VirtIO 9P Filesystem

## Phase 1: Discovery

### Feature Summary
- **Feature**: Mount host directories via VirtIO 9P (Plan 9 filesystem protocol)
- **Problem Statement**: LevitateOS cannot access host files directly. Currently requires building files into initramfs or disk image. 9P enables live access to host directories.
- **Benefits**: Rapid development iteration (edit on host, run in guest), larger file access without rebuilding images, shared development workflow

### Success Criteria
- [ ] Detect `virtio-9p` device during MMIO scan
- [ ] Complete 9P version handshake (`Tversion`/`Rversion`)
- [ ] Attach to filesystem root (`Tattach`/`Rattach`)
- [ ] Walk directory tree (`Twalk`/`Rwalk`)
- [ ] Read file contents (`Topen`, `Tread`)

### Current State Analysis

**QEMU Configuration** (NOT yet in `run.sh`):
```bash
# Need to add:
-fsdev local,security_model=mapped,id=fsdev0,path=/tmp/levitate_share \
-device virtio-9p-device,fsdev=fsdev0,mount_tag=hostshare
```

**virtio-drivers Support**:
```rust
// Device type IS recognized:
pub enum DeviceType {
    _9P = 9,  // ✅ Detected
    ...
}

// BUT: No 9P driver implementation exists in the crate!
// Only transport layer is available, protocol must be implemented manually.
```

**No Existing `no_std` Rust 9P Implementation**:
- `rs9p` — Tokio-based, requires async runtime
- `rust-9p` — Also Tokio-based
- Must implement 9P2000.L protocol from scratch

---

## Phase 2: 9P Protocol Overview

### Message Format

All 9P messages follow this structure:
```
┌─────────┬──────┬─────┬─────────────┐
│ size[4] │ type │ tag │ payload...  │
│ (u32le) │ (u8) │(u16)│ (variable)  │
└─────────┴──────┴─────┴─────────────┘
```

### Required Messages (Minimal Read-Only)

| T-Message | R-Message | Purpose |
|-----------|-----------|---------|
| Tversion | Rversion | Protocol version negotiation |
| Tattach | Rattach | Connect to filesystem root |
| Twalk | Rwalk | Navigate directory tree |
| Topen | Ropen | Open file for reading |
| Tread | Rread | Read file data |
| Tclunk | Rclunk | Release file handle (fid) |

### Message Type IDs

```rust
const TVERSION: u8 = 100;
const RVERSION: u8 = 101;
const TATTACH: u8 = 104;
const RATTACH: u8 = 105;
const TWALK: u8 = 110;
const RWALK: u8 = 111;
const TOPEN: u8 = 112;
const ROPEN: u8 = 113;
const TREAD: u8 = 116;
const RREAD: u8 = 117;
const TCLUNK: u8 = 120;
const RCLUNK: u8 = 121;
const RERROR: u8 = 107;  // Error response
```

### 9P2000.L Extensions

Linux uses 9P2000.L which adds:
- `Tlopen` (12) / `Rlopen` (13) — Linux-style open
- `Tlcreate` (14) / `Rlcreate` (15) — Linux-style create
- `Tgetattr` (24) / `Rgetattr` (25) — Get file attributes
- `Treaddir` (40) / `Rreaddir` (41) — Read directory

---

## Phase 3: Implementation Plan

### Module Structure

```
kernel/src/
├── p9/
│   ├── mod.rs        # Module exports
│   ├── protocol.rs   # Message types and serialization
│   ├── client.rs     # 9P client state machine
│   └── transport.rs  # VirtIO transport wrapper
└── virtio.rs         # Add DeviceType::_9P handling
```

### Step 1: Create Protocol Types (`kernel/src/p9/protocol.rs`)

```rust
//! 9P2000.L Protocol Message Types

use alloc::string::String;
use alloc::vec::Vec;

pub const P9_VERSION: &str = "9P2000.L";
pub const MAX_MSG_SIZE: u32 = 8192;

#[repr(u8)]
pub enum MsgType {
    Tversion = 100,
    Rversion = 101,
    Tattach = 104,
    Rattach = 105,
    Rerror = 107,
    Twalk = 110,
    Rwalk = 111,
    Topen = 112,
    Ropen = 113,
    Tread = 116,
    Rread = 117,
    Tclunk = 120,
    Rclunk = 121,
}

/// File identifier (handle)
pub type Fid = u32;

/// Unique QID identifying a file on the server
#[derive(Clone, Copy, Debug)]
pub struct Qid {
    pub qtype: u8,
    pub version: u32,
    pub path: u64,
}

impl Qid {
    pub fn parse(data: &[u8]) -> Option<(Self, &[u8])> {
        if data.len() < 13 { return None; }
        let qid = Qid {
            qtype: data[0],
            version: u32::from_le_bytes([data[1], data[2], data[3], data[4]]),
            path: u64::from_le_bytes([
                data[5], data[6], data[7], data[8],
                data[9], data[10], data[11], data[12]
            ]),
        };
        Some((qid, &data[13..]))
    }
}

pub struct MsgBuilder {
    buf: Vec<u8>,
}

impl MsgBuilder {
    pub fn new(msg_type: MsgType, tag: u16) -> Self {
        let mut buf = Vec::with_capacity(128);
        buf.extend_from_slice(&[0u8; 4]); // size placeholder
        buf.push(msg_type as u8);
        buf.extend_from_slice(&tag.to_le_bytes());
        Self { buf }
    }
    
    pub fn put_u32(&mut self, val: u32) {
        self.buf.extend_from_slice(&val.to_le_bytes());
    }
    
    pub fn put_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.buf.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        self.buf.extend_from_slice(bytes);
    }
    
    pub fn finish(mut self) -> Vec<u8> {
        let len = self.buf.len() as u32;
        self.buf[0..4].copy_from_slice(&len.to_le_bytes());
        self.buf
    }
}
```

### Step 2: Create VirtIO Transport (`kernel/src/p9/transport.rs`)

```rust
//! VirtIO 9P Transport Layer

use crate::virtio::{StaticMmioTransport, VirtioHal};
use alloc::vec::Vec;
use virtio_drivers::transport::Transport;

pub struct P9Transport {
    // VirtIO queues for 9P communication
    // Queue 0: hipri (high priority) - optional
    // Queue 1: request queue
}

impl P9Transport {
    pub fn new(transport: StaticMmioTransport) -> Result<Self, &'static str> {
        // Initialize VirtIO queues
        // Note: virtio-9p uses a simple request/response model
        todo!("Implement VirtIO 9P transport")
    }
    
    pub fn send(&mut self, msg: &[u8]) -> Result<(), &'static str> {
        todo!()
    }
    
    pub fn recv(&mut self) -> Result<Vec<u8>, &'static str> {
        todo!()
    }
}
```

### Step 3: Create 9P Client (`kernel/src/p9/client.rs`)

```rust
//! 9P Client State Machine

use super::protocol::*;
use super::transport::P9Transport;
use alloc::string::String;
use alloc::vec::Vec;

pub struct P9Client {
    transport: P9Transport,
    msize: u32,
    root_fid: Fid,
    next_fid: Fid,
    next_tag: u16,
}

impl P9Client {
    pub fn connect(transport: P9Transport) -> Result<Self, &'static str> {
        let mut client = P9Client {
            transport,
            msize: MAX_MSG_SIZE,
            root_fid: 0,
            next_fid: 1,
            next_tag: 0,
        };
        
        client.version()?;
        client.attach()?;
        
        Ok(client)
    }
    
    fn next_tag(&mut self) -> u16 {
        let tag = self.next_tag;
        self.next_tag = self.next_tag.wrapping_add(1);
        tag
    }
    
    fn version(&mut self) -> Result<(), &'static str> {
        let tag = self.next_tag();
        let mut msg = MsgBuilder::new(MsgType::Tversion, tag);
        msg.put_u32(MAX_MSG_SIZE);
        msg.put_string(P9_VERSION);
        
        self.transport.send(&msg.finish())?;
        let resp = self.transport.recv()?;
        
        // Parse Rversion response
        // ...
        
        Ok(())
    }
    
    fn attach(&mut self) -> Result<(), &'static str> {
        let tag = self.next_tag();
        let mut msg = MsgBuilder::new(MsgType::Tattach, tag);
        msg.put_u32(self.root_fid);  // fid
        msg.put_u32(u32::MAX);       // afid (no auth)
        msg.put_string("");          // uname
        msg.put_string("");          // aname
        
        self.transport.send(&msg.finish())?;
        let _resp = self.transport.recv()?;
        
        Ok(())
    }
    
    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>, &'static str> {
        // 1. Twalk to path
        // 2. Topen file
        // 3. Tread contents
        // 4. Tclunk to release
        todo!()
    }
}
```

### Step 4: Update `kernel/src/virtio.rs`

```rust
virtio_drivers::transport::DeviceType::_9P => {
    crate::p9::init(transport);
}
```

---

## Phase 4: QEMU Configuration

### Update `run.sh`

```bash
# Add before -serial:
-fsdev local,security_model=mapped-xattr,id=fsdev0,path=/tmp/levitate_share \
-device virtio-9p-device,fsdev=fsdev0,mount_tag=hostshare
```

### Create Host Share Directory

```bash
mkdir -p /tmp/levitate_share
echo "Hello from host!" > /tmp/levitate_share/hello.txt
```

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| No existing crate | High | Manual protocol implementation |
| Complex protocol | Medium | Implement minimal subset first |
| VirtIO transport complexity | Medium | Study virtio-blk for patterns |
| Debugging difficulty | Medium | Add verbose protocol logging |

---

## Estimated Effort

| Component | Time |
|-----------|------|
| Protocol types | 2-3 hours |
| VirtIO transport | 4-6 hours |
| Client state machine | 4-6 hours |
| Testing & debugging | 4-8 hours |
| **Total** | **14-23 hours (2-3 days)** |

---

## Alternative: Defer to Later Phase

Given complexity, consider:
1. Complete VirtIO Net and GPU Text first
2. Defer 9P to Phase 7+ when more infrastructure exists
3. Use initramfs for file loading in the meantime

---

## References

- [9P2000 Protocol Spec](http://ericvh.github.io/9p-rfc/)
- [9P2000.L Extensions](https://github.com/chaos/diod/blob/master/protocol.md)
- [VirtIO 9P Spec](https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html#x1-3900009)
- [Linux kernel 9P](https://www.kernel.org/doc/Documentation/filesystems/9p.txt)

---

## DEFERRED — Notes for Future Team

**Status**: This task is deferred. VirtIO Net and GPU Text have higher priority.

### Recommended Approach for Future Implementation

1. **Check crates.io for new `no_std` 9P crates** before starting manual implementation:
   ```
   https://crates.io/search?q=9p+no_std
   https://crates.io/search?q=plan9+embedded
   ```

2. **Consider forking/adapting existing crates**:
   - `rs9p` (https://github.com/pfpacket/rust-9p) — Tokio-based but protocol types are reusable
   - `nine` (https://crates.io/crates/nine) — Check if it supports `no_std`
   - Extract protocol parsing code, remove async/Tokio dependencies

3. **Minimal viable subset** (read-only is sufficient for dev workflow):
   - `Tversion/Rversion` — Version handshake
   - `Tattach/Rattach` — Connect to root
   - `Twalk/Rwalk` — Navigate paths
   - `Tlopen/Rlopen` — Open file (9P2000.L)
   - `Tread/Rread` — Read data
   - `Tclunk/Rclunk` — Release handle

4. **VirtIO transport pattern** — Follow `block.rs`:
   - Device type `_9P = 9` is already recognized
   - Use same `StaticMmioTransport` + `VirtioHal` pattern
   - 9P uses 2 virtqueues: hipri (optional) and requests

5. **Simplification options**:
   - Start with single-threaded, blocking reads only
   - No write support initially (read-only mount)
   - No directory listing initially (just `walk` + `read`)
   - Fixed max message size (8KB typical)

### Quick Win: Use Initramfs Instead

If 9P complexity is blocking, expand initramfs capabilities:
- Build larger initramfs with needed files
- Use `embedded-sdmmc` for FAT32 disk images
- Defer 9P until a `no_std` crate emerges

### Contact Points

- VirtIO 9P in QEMU: https://wiki.qemu.org/Documentation/9psetup
- Linux 9P client source: `fs/9p/` in kernel tree (reference implementation)
