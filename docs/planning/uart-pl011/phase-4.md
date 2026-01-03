# Phase 4 — Integration and Testing: PL011 UART Refactor

## Integration Points
- `Pl011Uart` in `levitate-hal`.
- `GIC` IRQ 33 (UART0) enabled.
- `kernel/src/exceptions.rs` calling the UART handler.

## Test Strategy
- **Unit Testing**:
  - Test `Pl011Uart` registers (mocking MMIO if possible).
  - Test `RingBuffer` in `levitate-utils`.
- **Integration Testing**:
  - Boot in QEMU.
  - Verify that `println!` output is still readable.
  - Verify that typing in the QEMU console triggers UART interrupts and "Input Event" logs (once integrated into the input system).
- **Manual Verification**:
  - Verify character echo or simple shell behavior if implemented.

## Steps
1. **Step 1 – Baseline Verification**
   - [ ] Ensure `println!` works before making changes.
2. **Step 2 – Integration Verification**
   - [ ] Verify that new driver produces identical output to the legacy one.
3. **Step 3 – Interrupt Verification**
   - [ ] Verify IRQ 33 is triggered in GIC.
   - [ ] Verify characters are received in the buffer.
