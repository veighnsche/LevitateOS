# Phase 2: Root Cause Analysis

**Team:** TEAM_337  
**Purpose:** Isolate where and why the bug occurs.

## Root Cause Identified

### Finding

The GPU on AArch64 QEMU is attached via **PCI**, not MMIO!

```rust
// xtask/src/qemu/builder.rs:261-265
_ => {
    // aarch64: always use virtio-gpu-pci  <-- PCI, not MMIO!
    let gpu_spec = format!(
        "virtio-gpu-pci,xres={},yres={}",
        self.gpu_resolution.width, self.gpu_resolution.height
    );
    cmd.args(["-device", &gpu_spec]);
}
```

### Why TEAM_336's Fix Didn't Work

TEAM_336 added MMIO scanning for GPU on AArch64:

```rust
// kernel/src/virtio.rs - detect_gpu_transport()
#[cfg(target_arch = "aarch64")]
fn detect_gpu_transport() -> Option<MmioTransport> {
    // Scans MMIO slots for GPU
    for i in 0..VIRTIO_MMIO_COUNT {
        // ... looks for DeviceType::GPU in MMIO
    }
}
```

But the GPU isn't on MMIO - it's on **PCI bus** (same as x86_64).

### Device Configuration on AArch64 QEMU

| Device | Bus | Config |
|--------|-----|--------|
| GPU | **PCI** | `virtio-gpu-pci` |
| Keyboard | MMIO | `virtio-keyboard-device` |
| Block | MMIO | `virtio-blk-device` |
| Network | MMIO | `virtio-net-device` |

The other devices use MMIO (`-device` suffix), but GPU uses PCI.

## Fix Strategy

On AArch64, use **PCI scanning** for GPU (same as x86_64), not MMIO scanning.

### Option A: Use PCI for GPU on both architectures

Update `detect_gpu_transport()` to use PCI on both x86_64 and aarch64.

### Option B: Use arch-unified transport

Keep the `GpuTransport` type alias but make it `PciTransport` on both architectures.

## Recommendation

**Option A** - Simply use `los_pci::find_virtio_gpu()` for both architectures since QEMU attaches GPU via PCI on both.
