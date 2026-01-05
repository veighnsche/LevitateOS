# Kernel Development Best Practices

This document captures architectural patterns and technical gotchas discovered during the development of LevitateOS.

## 1. Console & Deadlock Prevention

### The Dual-Console Trap
LevitateOS supports dual-console output (UART + GPU). The `println!` macro targets both.
- **Problem**: If a low-level component (like the GPU driver or an interrupt handler) calls `println!`, it can cause a recursive deadlock if the console system is already locked or if the callback itself triggers another log.
- **Gotcha**: A GPU flush failure that logs an error via `println!` will deadlock because `println!` is already holding the `WRITER` lock and is waiting for the GPU callback to finish.

### Best Practice: `serial_println!`
Always use `levitate_hal::serial_println!` (UART-only) for:
1. **Interrupt Handlers**: Standard `println!` is unsafe in IRQs due to dual-console complexity.
2. **Low-level Drivers**: Components like `gpu.rs` and `terminal.rs` should never use `println!`.
3. **Early Boot**: Before Dual Console is registered.

---

## 2. Safe Interrupt Handling

### IrqSafeLock
Any data structure shared between a thread and an interrupt handler (like `RX_BUFFER` or `GPU_TERMINAL`) **must** use `IrqSafeLock`. Standard `Spinlock` will deadlock if an interrupt occurs while the thread holds the lock.

### Verification Techniques
If the system feels unresponsive, use these "Heartbeat" diagnostics:
1. **IRQ Heartbeat**: Add `serial_println!("T")` in `TimerHandler`. If you don't see `T`s, interrupts are disabled at the CPU level or the GIC is misconfigured.
2. **UART Status**: Read the Flag Register (FR) at `UART_VA + 0x18`.
   - Bit 4 (`RXFE`): If this stays `1` while you type, the hardware is not receiving data.
   - Bit 5 (`TXFF`): If this is `1`, the UART is backed up.

---

## 3. GPU Rendering Performance

### The Flush Penalty
GPU memory flushes are extremely expensive.
- **Rule**: Never flush per character.
- **Pattern**: Perform all drawing logic first, then call `flush()` once at the end of the operation (e.g., at the end of `write_str` in `console_gpu.rs`).

---

## 4. Team Attribution
When documenting fixes or gotchas in code, always include your Team ID:
```rust
// TEAM_083: Use serial_println! to avoid recursive deadlock in dual-console path
```
