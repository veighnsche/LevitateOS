# TEAM_001: Rust Porting Notes

## VirtIO & Graphics Implementation Findings

### 1. QEMU Device Configuration
For ARM64 `virt` machine, use MMIO-based VirtIO devices.
**Gotcha**: `virtio-keyboard-device` and `virtio-tablet-device` are separate. Use `tablet` for absolute coordinates (essential for GUI cursors).
**Command Line**:
```bash
qemu-system-aarch64 ... \
    -device virtio-gpu-device \
    -device virtio-keyboard-device \
    -device virtio-tablet-device
```

### 2. Debugging in Interrupt Contexts
**Critical**: Calling `println!` (UART) inside an interrupt handler or a loop polled by interrupts (like `input::poll`) can cause deadlocks or crashes (Sync Exception).
**Solution**:
- Use a raw `puts` function that writes directly to the UART MMIO address `0x09000000`.
- Avoid `core::fmt` machinery in critical sections.

### 3. VirtIO Interrupt Handling
**Gotcha**: High-frequency devices (Tablet) can cause an interrupt storm if not acknowledged immediately.
**Solution**: Always call `.ack_interrupt()` on the driver instance or `Gic::eoi()` in the handler loop. We implemented explicit `input.ack_interrupt()` in the polling loop.

### 4. Headless Verification
To verify graphics and input without a display:
1. Use `-display none`.
2. Use `-monitor stdio` to interact with QEMU.
3. Inject input via monitor: `sendkey a`.
4. Verify via serial logs: check for "Input Event" or cursor update logs.
