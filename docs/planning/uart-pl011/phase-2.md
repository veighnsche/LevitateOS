# Phase 2 — Design: PL011 UART Refactor

## Proposed Solution
Refactor `levitate-hal/src/console.rs` to include a full bit-aware PL011 driver. The driver will use `volatile` access to specific registers and check flags before performing I/O. Eventually, it will use a `RingBuffer` for interrupt-driven I/O.

## API Design
```rust
pub struct Pl011Uart {
    base: usize,
}

impl Pl011Uart {
    pub const fn new(base: usize) -> Self { ... }
    pub fn init(&mut self) { ... }
    pub fn send(&mut self, byte: u8) { ... }
    pub fn receive(&mut self) -> Option<u8> { ... }
    pub fn enable_rx_interrupts(&mut self) { ... }
    pub fn handle_interrupt(&mut self) { ... }
}
```

## Register Map (Offsets)
- `DR`: 0x00 (Data)
- `FR`: 0x18 (Flag)
- `IBRD`: 0x24 (Integer Baud Rate)
- `FBRD`: 0x28 (Fractional Baud Rate)
- `LCR_H`: 0x2C (Line Control)
- `CR`: 0x30 (Control)
- `IMSC`: 0x38 (Interrupt Mask)
- `ICR`: 0x44 (Interrupt Clear)

## Behavioral Decisions
1. **Blocking vs non-blocking `send`?**
   - *Recommendation*: Initially blocking (spin wait for `TXFF` to clear). Later, use a TX ring buffer.
2. **Interrupt handling?**
   - *Recommendation*: Use RX interrupts to fill a global `RingBuffer`. The kernel's `input` module can then consume from this buffer.
3. **Baud Rate?**
   - *Recommendation*: Use a standard default (e.g., 115200). QEMU usually ignores this, but it's good practice.

## Open Questions
- **Q1**: Do we need a concurrent `RingBuffer`?
  - *Hypothesis*: Yes, since the IRQ handler writes and the application (kmain loop) reads. A simple lock-free ring buffer or a spinlock-protected one is needed.
- **Q2**: Should we rename `console.rs` to `uart_pl011.rs`?
  - *Hypothesis*: Yes, to be more descriptive, and `console.rs` can be a high-level wrapper if needed.

## Steps
1. **Step 1 – Draft Initial Design**
   - [ ] Define `Pl011Uart` struct and register offsets.
2. **Step 2 – Define Behavioral Contracts**
   - [ ] Document register bit definitions (bitflags).
3. **Step 3 – Review Design Against Architecture**
   - [ ] Ensure it fits the `levitate-hal` pattern (like `Timer` and `Gic`).
