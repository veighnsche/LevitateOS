# TEAM_114: GPU Driver Choice for PCI Migration

**Created:** 2026-01-05
**Status:** Open - Needs USER decision
**Blocking:** VirtIO PCI plan implementation

---

## Context

The VirtIO PCI migration plan requires a GPU driver that works with `PciTransport`.
There are two GPU driver options available.

---

## Architecture

```
┌─────────────────────────────────────┐
│  GPU Driver (protocol/commands)     │  ← DECISION NEEDED
├─────────────────────────────────────┤
│  Transport (how to talk to device)  │  ← PciTransport (from plan)
├─────────────────────────────────────┤
│  Hardware (ECAM/MMIO registers)     │  ← kernel/src/pci.rs (from plan)
└─────────────────────────────────────┘
```

---

## Options

### Option A: Fix and Extend levitate-drivers-gpu

**What:**
1. Debug and fix the timeout issues documented in TEAM_107
2. Make `VirtioGpu<H>` generic over transport: `VirtioGpu<H, T: Transport>`
3. Use with `PciTransport`

**Pros:**
- Keeps custom GPU driver (more control, learning opportunity)
- Can add LevitateOS-specific features later

**Cons:**
- Must fix unknown timeout bug first (TEAM_107)
- More work before PCI migration can be tested
- Custom code = more maintenance

**Risk:** Higher - blocked by undiagnosed bug

---

### Option B: Use virtio-drivers VirtIOGpu

**What:**
1. Use `virtio_drivers::device::gpu::VirtIOGpu<HalImpl, PciTransport>`
2. Keep `levitate-drivers-gpu` for future custom needs (or deprecate)

**Pros:**
- Already tested and working
- Immediate PCI migration testing possible
- Less code to maintain

**Cons:**
- Less control over GPU internals
- Dependency on external crate for core functionality

**Risk:** Lower - proven code

---

## Decision

**Option B selected by USER (2026-01-05)**

- [x] **Option B:** Use virtio-drivers VirtIOGpu, defer custom driver work

### Actions Taken

1. Archived `levitate-drivers-gpu` to `.archive/levitate-drivers-gpu/`
2. Removed from workspace `Cargo.toml`
3. Removed dependency from `kernel/Cargo.toml`
4. Enabled `pci` feature for `virtio-drivers`
5. Stubbed `kernel/src/gpu.rs` until PCI implementation complete

---

## Related Files

- `.questions/TEAM_107_gpu_driver_issues.md` - Documents the timeout bug
- `levitate-drivers-gpu/src/device.rs` - Custom GPU driver
- `kernel/Cargo.toml` - Already depends on `virtio-drivers = "0.12"`
