# Phase 2: Protocol and Resource Infrastructure

## Target Design
Implement the foundational data structures and resource management for the VirtIO GPU driver.

- **`protocol.rs`**: Strictly repr(C) structs representing the VirtIO GPU control plane.
- **`resource.rs`**: RAII handles for host-side resources.
- **Async foundation**: Command traits designed for async-first per user requirement (Q4).

> **Note:** This is for QEMU VirtIO development. Pixel 6 production target will use Mali GPU driver.

## Perfection Criteria
- Every struct in `protocol.rs` must correctly represent the standard [VirtIO GPU Spec](https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html#x1-3310007).
- `GpuResource` must guarantee that resources are unreferenced on drop.
- Zero usage of "magic constants" outside of the `protocol` module.

## Steps
1. **Step 1 – Define Protocol Constants & Structs**
   - Commands, response types, and format enums.
2. **Step 2 – Implement RAII Resource Manager**
   - `GpuResource` with internal ID tracking.
   - `ResourceId` newtype for type safety.
   - Drop impl that sends RESOURCE_UNREF command.
3. **Step 3 – Unit Test Protocol Serialization**
   - Ensure `zerocopy` alignment and size are as expected.
4. **Step 4 – Define Async Command Trait**
   - `async fn send_command<R: VirtIOGPUResp>(&mut self, cmd: impl VirtIOGPUReq) -> Result<R, GpuError>`
   - Waker-based completion for VirtQueue responses.
