# Refactor Phase 3 — xtask "Peeking" Tools

## Goal
Build host-side tools that can bypass the QEMU display window to verify the kernel's state.

## Proposed Tools

### `cargo xtask gpu-dump`
- **Method:** Connect to QEMU via QMP (Unix Socket).
- **Execution:** 
    1. Query VirtIO GPU scanout address via QMP.
    2. Use `pmemload` or a memory dump command to read the raw pixels from guest RAM.
    3. Convert raw pixels (BGRA8888) to PNG.
- **Benefit:** If the QEMU window says "Display output is not active", we can still see if the kernel *actually* drew anything.

### `cargo xtask qmp-shell`
- A thin wrapper around the QEMU Machine Protocol to allow manual inspection of device states, interrupts, and registers.

## Steps
1. **Step 1:** Update `run.sh` to expose a QMP socket (`-qmp unix:/tmp/qemu-qmp.sock,server,nowait`).
2. **Step 2:** Add a `qmp` module to `xtask` implementing the JSON-RPC-like protocol.
3. **Step 3:** Implement the framebuffer capture logic.
