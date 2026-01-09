# TEAM_319: Fix x86_64 Black Screen - PIC Initialization

## Bug Report

**Symptom:** x86_64 boots successfully (serial output works) but display shows black screen.

**Root Cause:** The 8259 PIC was never initialized. By default:
1. All IRQs are masked (disabled)
2. IRQ0-7 map to vectors 0x08-0x0F (conflicts with CPU exceptions)

Without PIC initialization:
- IRQ0 (timer) is masked → no timer interrupts
- No timer interrupts → `TIMER_HANDLER` never fires
- No GPU flush → black screen

TEAM_318 added the timer handler registration, but the underlying PIC was never configured.

## Fix

Added `pic.rs` module with 8259 PIC initialization:
1. Remap IRQ0-7 to vectors 32-39, IRQ8-15 to vectors 40-47
2. Unmask IRQ0 (timer), IRQ2 (cascade), IRQ4 (COM1 serial)

## Files Modified

- `crates/hal/src/x86_64/interrupts/pic.rs` - NEW: PIC driver
- `crates/hal/src/x86_64/interrupts/mod.rs` - Added pic module
- `crates/hal/src/x86_64/mod.rs` - Call init_pic() in HAL init

## Verification

- Build passes: `cargo build --release -p levitate-kernel --target x86_64-unknown-none`

## Status: IN PROGRESS - Testing
