# Refactor Phase 1 — Discovery and Telemetry Hooks

## Refactor Summary
The current GPU/Console system is a "black box" that frequently enters an inactive state in QEMU without providing feedback to the developer. This phase focuses on mapping the existing dependencies and creating the "hooks" necessary for host-side observability.

## Success Criteria
- [ ] A complete dependency map of `gpu.rs`, `terminal.rs`, and `console_gpu.rs`.
- [ ] A defined "GPU Status Protocol" that the kernel can use to report its internal VirtIO state.
- [ ] A verified QEMU tracing setup that captures `virtio_gpu` events.

## Behavioral Contracts
- **Serial Console:** Must remain the primary reliable diagnostic path.
- **GPU Display:** Must stay active (10Hz flush baseline).

## Current Architecture Notes
The `console_gpu.rs` currently manages two global locks: `GPU` and `GPU_TERMINAL`. This coupling is high-risk.

## Steps

### Step 1 — Map Current Architecture and Boundaries
- Analyze the locking strategy between `GPU` and `GPU_TERMINAL`.

### Step 2 — Implement Kernel "Heartbeat" Telemetry
- Update `GpuState` to track total flushes, failed flushes, and state.
- Periodically dump this state to the serial console.

### Step 3 — Build `xtask` QEMU Monitor Wrapper
- Add a tool to `xtask` to interact with QEMU Machine Protocol (QMP).
