# Phase 2: Structural Extraction

**TEAM_332** | VirtIO Driver Reorganization  
**TEAM_333** | Reviewed: 2026-01-09 — Added device trait crates (Theseus pattern)

## Target Design

### New Crate Structure

```
crates/
├── traits/                         # NEW: Device trait crates (Theseus pattern)
│   ├── storage-device/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs              # StorageDevice trait
│   │
│   ├── input-device/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs              # InputDevice trait
│   │
│   └── network-device/
│       ├── Cargo.toml
│       └── src/lib.rs              # NetworkDevice trait
│
├── drivers/
│   ├── virtio-blk/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # Public API: impl StorageDevice
│   │       └── device.rs           # VirtIO block device logic
│   │
│   ├── virtio-input/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # Public API: impl InputDevice
│   │       ├── device.rs           # VirtIO input device logic
│   │       └── keymap.rs           # Linux keycode mapping
│   │
│   ├── virtio-net/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # Public API: impl NetworkDevice
│   │       └── device.rs           # VirtIO net device logic
│   │
│   └── virtio-gpu/                 # Rename from crates/gpu/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs              # Public API: Gpu, Display
│           ├── device.rs           # VirtIO GPU logic
│           └── framebuffer.rs      # Limine framebuffer fallback
│
├── virtio-transport/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Transport enum, VirtioDriver trait
│       ├── mmio.rs                 # MMIO transport wrapper
│       └── pci.rs                  # PCI transport wrapper
```

### Device Trait Crates (NEW - Theseus Pattern)

**Reference:** `.external-kernels/theseus/kernel/storage_device/src/lib.rs`

Separating traits from implementations enables:
- Non-VirtIO drivers to implement the same interfaces (PS2, AHCI, NVMe)
- Clean dependency graph (traits have no implementation dependencies)
- Runtime polymorphism via `Arc<Mutex<dyn StorageDevice + Send>>`

```rust
// crates/traits/storage-device/src/lib.rs
#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use spin::Mutex;

/// Trait for block storage devices (HDDs, SSDs, VirtIO Block, etc.)
/// 
/// Reference: Theseus storage_device crate
pub trait StorageDevice: Send {
    /// Block size in bytes (typically 512)
    fn block_size(&self) -> usize;
    
    /// Total size in blocks
    fn size_in_blocks(&self) -> usize;
    
    /// Read blocks starting at `block_id` into `buf`
    fn read_blocks(&mut self, block_id: usize, buf: &mut [u8]) -> Result<(), StorageError>;
    
    /// Write blocks starting at `block_id` from `buf`
    fn write_blocks(&mut self, block_id: usize, buf: &[u8]) -> Result<(), StorageError>;
}

/// Thread-safe reference to a storage device
pub type StorageDeviceRef = Arc<Mutex<dyn StorageDevice + Send>>;

#[derive(Debug, Clone, Copy)]
pub enum StorageError {
    NotReady,
    InvalidBlock,
    IoError,
}
```

```rust
// crates/traits/input-device/src/lib.rs
#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use spin::Mutex;

/// Trait for input devices (keyboards, mice, touchpads)
pub trait InputDevice: Send {
    /// Poll for pending input events
    fn poll(&mut self) -> bool;
    
    /// Read next character (keyboard)
    fn read_char(&mut self) -> Option<char>;
    
    /// Check if Ctrl+C was pressed
    fn ctrl_c_pressed(&self) -> bool;
}

pub type InputDeviceRef = Arc<Mutex<dyn InputDevice + Send>>;
```

```rust
// crates/traits/network-device/src/lib.rs
#![no_std]

extern crate alloc;

use alloc::{sync::Arc, vec::Vec};
use spin::Mutex;

/// Trait for network devices (NICs)
pub trait NetworkDevice: Send {
    /// Get MAC address
    fn mac_address(&self) -> [u8; 6];
    
    /// Send packet
    fn send(&mut self, packet: &[u8]) -> Result<(), NetworkError>;
    
    /// Receive packet (returns None if no packet available)
    fn receive(&mut self) -> Option<Vec<u8>>;
    
    /// Check if TX queue has space
    fn can_send(&self) -> bool;
}

pub type NetworkDeviceRef = Arc<Mutex<dyn NetworkDevice + Send>>;

#[derive(Debug, Clone, Copy)]
pub enum NetworkError {
    NotInitialized,
    QueueFull,
    TransmitFailed,
}
```

### Transport Abstraction

**Decision:** WRAP `virtio-drivers` transports (see plan.md Q1)

```rust
// crates/virtio-transport/src/lib.rs
#![no_std]

use virtio_drivers::transport::{
    mmio::MmioTransport,
    pci::PciTransport,
    DeviceType,
};

/// Unified transport for VirtIO devices
/// Wraps virtio-drivers transports (not replaces)
pub enum Transport {
    Mmio(MmioTransport<'static>),
    Pci(PciTransport),
}

impl Transport {
    pub fn device_type(&self) -> DeviceType {
        match self {
            Transport::Mmio(t) => t.device_type(),
            Transport::Pci(t) => t.device_type(),
        }
    }
}

// Delegate Transport trait implementation to inner type
// This allows using Transport with virtio-drivers device constructors
```

### Driver Trait

```rust
// crates/virtio-transport/src/lib.rs

/// Common interface for VirtIO device drivers
pub trait VirtioDriver: Send + Sync {
    /// Device type this driver handles
    const DEVICE_TYPE: DeviceType;
    
    /// Create driver from transport
    fn new(transport: Transport) -> Result<Self, DriverError>
    where Self: Sized;
    
    /// Handle device interrupt
    fn handle_interrupt(&mut self);
}

#[derive(Debug)]
pub enum DriverError {
    TransportMismatch,
    InitFailed,
    NoDevice,
}
```

## Extraction Strategy

### Order of Extraction

1. **Device trait crates** - No dependencies, create first (NEW)
2. **virtio-transport** - Foundation for driver crates
3. **virtio-input** - Already has PCI support, good test case
4. **virtio-blk** - Simple, well-understood
5. **virtio-net** - Similar to block
6. **virtio-gpu** - Most complex, rename + extend existing crate

### Coexistence Strategy

During extraction:
- New crates created alongside old code
- Kernel imports from new crates
- Old kernel/*.rs files become thin wrappers
- Once all call sites migrated, delete old code

## Modular Refactoring Rules

Per Rule 7:

1. **Each module owns its state** - Driver state in driver crate, not kernel
2. **Private fields** - Only expose intentional APIs
3. **No deep imports** - `use virtio_blk::VirtIOBlk`, not `use virtio_blk::internal::queue::*`
4. **File sizes** - Target <500 lines per file

## Feature Flags

**Decision:** Support `std` for testing (see plan.md Q2)

All driver crates support:

```toml
[features]
default = []
std = []  # Enables host-side unit testing with mock transports
```

---

## Phase 2 Steps

### Step 1: Create Device Trait Crates (NEW)

**Goal:** Create abstract trait crates for device categories

Tasks:
1. Create `crates/traits/storage-device/` with `StorageDevice` trait
2. Create `crates/traits/input-device/` with `InputDevice` trait  
3. Create `crates/traits/network-device/` with `NetworkDevice` trait
4. Add all three to workspace Cargo.toml
5. Keep traits minimal — add methods as needed during migration

**Exit Criteria:**
- All trait crates compile
- Traits are `no_std` compatible
- READMEs reference Theseus pattern

### Step 2: Create virtio-transport Crate

**Goal:** Create unified transport abstraction

Tasks:
1. Create `crates/virtio-transport/` directory structure
2. Define `Transport` enum wrapping MMIO and PCI
3. Implement delegation to inner transports
4. Define `VirtioDriver` trait
5. Add to workspace Cargo.toml

**Exit Criteria:**
- Crate compiles
- Can create Transport from either MMIO or PCI
- Transport can be used with `virtio-drivers` device constructors

### Step 3: Extract virtio-input Crate

**Goal:** Move input driver to dedicated crate

Tasks:
1. Create `crates/drivers/virtio-input/` structure
2. Add dependency on `input-device` trait crate
3. Move `linux_code_to_ascii()` and keymap logic
4. Create `VirtIOInput` struct implementing `InputDevice` trait
5. Implement `poll()` and `read_char()` in crate
6. Export clean public API

**Exit Criteria:**
- Crate compiles
- Implements `InputDevice` trait
- Can poll input events from both MMIO and PCI transports
- Kernel can use new crate (temporarily alongside old code)

### Step 4: Extract virtio-blk Crate

**Goal:** Move block driver to dedicated crate

Tasks:
1. Create `crates/drivers/virtio-blk/` structure
2. Add dependency on `storage-device` trait crate
3. Move block device logic from `kernel/src/block.rs`
4. Create `VirtIOBlk` struct implementing `StorageDevice` trait
5. Implement `read_blocks()`, `write_blocks()` in crate
6. Note: PCI support deferred (see plan.md Q3)

**Exit Criteria:**
- Crate compiles
- Implements `StorageDevice` trait
- Block operations work with MMIO transport
- FS layer can use new crate

### Step 5: Extract virtio-net Crate

**Goal:** Move network driver to dedicated crate

Tasks:
1. Create `crates/drivers/virtio-net/` structure
2. Add dependency on `network-device` trait crate
3. Move net device logic from `kernel/src/net.rs`
4. Create `VirtIONet` struct implementing `NetworkDevice` trait
5. Note: PCI support deferred (see plan.md Q3)

**Exit Criteria:**
- Crate compiles
- Implements `NetworkDevice` trait
- Network operations work with MMIO transport

### Step 6: Reorganize virtio-gpu Crate

**Goal:** Rename and extend existing GPU crate

Tasks:
1. Rename `crates/gpu/` to `crates/drivers/virtio-gpu/`
2. Move Limine framebuffer fallback from kernel to crate
3. Create unified `GpuBackend` enum in crate
4. Update all imports
5. Note: GPU does not implement a trait crate (display is special)

**Exit Criteria:**
- Crate compiles at new location
- GPU works with VirtIO and Limine framebuffer
- Screenshot tests pass
