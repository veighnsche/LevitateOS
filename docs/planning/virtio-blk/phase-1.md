# Phase 1: Discovery - VirtIO Block Driver

## Feature Summary
- **Feature**: VirtIO Block Device (`virtio-blk`) Driver.
- **Problem Statement**: LevitateOS currently lacks persistent storage support. A block driver is required to read from and write to disk images, which is a prerequisite for implementing a file system.
- **Benefits**: Enables booting with a root filesystem, loading user programs, and persistent data storage.

## Success Criteria
- [ ] `virtio-blk` device is successfully detected during MMIO scan.
- [ ] Driver can read a specific block from the disk.
- [ ] Driver can write a specific block to the disk.
- [ ] Basic unit or integration tests verify block I/O correctness.

## Current State Analysis
- **Infrastructure**: `kernel/src/virtio.rs` already exists and uses the `virtio_drivers` crate. It handles MMIO scanning (address `0x0a000000` to `0x0a100000`).
- **Existing Devices**: GPU and Input devices are already supported.
- **HAL**: `VirtioHal` is implemented in `virtio.rs`, providing DMA allocation and physical-to-virtual address mapping.
- **Paging**: MMU is initialized and maps the VirtIO MMIO range (`0x0a000000` to `0x0a100000`).

## Codebase Reconnaissance
- **Files to touch**:
    - `kernel/src/virtio.rs`: Add `virtio-blk` to the device scan match arm.
    - `kernel/src/block.rs` (NEW): Implement the block driver wrapper.
    - `docs/ROADMAP.md`: Update status.
- **APIs involved**:
    - `virtio_drivers::device::blk::VirtIOBlk`: The core driver from the crate.
    - `levitate_hal::mmu`: For address translation.
- **Constraints**:
    - Block size is typically 512 bytes.
    - DMA must be used for transfers.
    - Interrupt handling for block I/O completion needs to be considered (though polling might be used for initial implementation).

## Steps
1. **Step 1 - Capture Feature Intent**: (Completed in this file)
2. **Step 2 - Analyze Current State**: (Completed in this file)
3. **Step 3 - Source Code Reconnaissance**: (Completed in this file)
4. **Step 4 - Verify Build and Environment**: Ensure `virtio-drivers` supports `virtio-blk` (it does).
