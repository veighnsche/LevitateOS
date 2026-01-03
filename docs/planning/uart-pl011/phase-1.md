# Phase 1 — Discovery: PL011 UART Refactor

## Feature Summary
Refactor the current `console.rs` into a robust PrimeCell UART (PL011) driver. This will add support for check-before-write, input (read) capabilities, and eventually interrupt-driven RX/TX using ring buffers.

## Success Criteria
- [ ] PL011 Register map defined with `bitflags`.
- [ ] `write_byte` checks `FR`.
- [ ] `read_byte` checks `FR`.
- [ ] Interrupts enabled in `CR` and `IMSC`.
- [ ] UART integrated with GIC (IRQ 33).

## Current State Analysis
- Current `console.rs` writes directly to `UART0_BASE` without any flow control or status checking.
- It is write-only.
- It is a global `static WRITER` protected by a `Spinlock`.

## Codebase Reconnaissance
- `levitate-hal/src/console.rs`: Core implementation.
- `kernel/src/main.rs`: Calls `println!`.
- `kernel/src/exceptions.rs`: Will need to handle UART interrupts.

## Constraints
- Must remain compatible with current `print!` and `println!` macros.
- Must be `no_std`.
- Should handle concurrent access (already uses `Spinlock`).
