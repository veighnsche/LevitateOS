# Phase 3 — Implementation: PL011 UART Refactor

## Implementation Overview
This phase builds the `Pl011Uart` driver and integrates it into the kernel, replacing the primitive `WRITER` in `console.rs`.

## Design Reference
- [Phase 2 — Design](file:///home/vince/Projects/LevitateOS/docs/planning/uart-pl011/phase-2.md)

## Steps

### Step 1 – Core Driver Logic
- [ ] Create `levitate-hal/src/uart_pl011.rs`.
- [ ] Define `Pl011Uart` struct and implement `bitflags` for registers.
- [ ] Implement `init`, `send`, `receive`.

### Step 2 – Integration with Console
- [ ] Update `levitate-hal/src/console.rs` to use `Pl011Uart`.
- [ ] Ensure `println!` works using the new driver.

### Step 3 – Interrupt-Driven I/O
- [ ] Implement a `StaticRingBuffer` in `levitate-utils`.
- [ ] Add RX interrupt handling to `Pl011Uart`.
- [ ] Update `kernel/src/exceptions.rs` to call `UART0.handle_interrupt()`.
- [ ] Enable IRQ 33 in GIC during kernel init.

### Step 4 – Cleanup
- [ ] Remove old direct MMIO access from `console.rs`.
- [ ] Verify that all `println!` calls still work correctly.
