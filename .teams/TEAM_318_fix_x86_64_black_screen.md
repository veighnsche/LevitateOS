# TEAM_318: Fix x86_64 Black Screen

## Bug Report

**Symptom:** x86_64 boots successfully (serial output works) but display shows black screen.

**Environment:**
- Platform: x86_64 (QEMU with Limine)
- Working: Serial console shows full boot to shell
- Failing: VNC/GTK display remains black

## Root Cause

The timer IRQ handler was never registered on x86_64. In `kernel/src/init.rs`, the x86_64 path (lines 254-263) only initialized PIT but skipped registering the `TIMER_HANDLER`.

The `TIMER_HANDLER` is responsible for periodic GPU flushes (every 5th interrupt at ~100Hz = 20Hz display refresh). Without it, the framebuffer was never pushed to the display.

```rust
// Before fix - x86_64 path:
#[cfg(target_arch = "x86_64")]
{
    los_hal::pit::Pit::init(100);
    // NO HANDLER REGISTERED - GPU never flushed!
}
```

## Fix

Added timer handler registration for x86_64 using existing `apic::register_handler()`:

```rust
#[cfg(target_arch = "x86_64")]
{
    los_hal::pit::Pit::init(100);
    
    // TEAM_318: Register timer handler for GPU flush on x86_64
    los_hal::x86_64::interrupts::apic::register_handler(32, &TIMER_HANDLER);
}
```

This uses the existing IRQ dispatch mechanism that doesn't require APIC MMIO access (vector 32 = PIT timer via legacy PIC).

## Files Modified

- `kernel/src/init.rs` - Added timer handler registration for x86_64
- `xtask/src/run.rs` - Fixed VNC mode to use PCI device types for x86_64

## Verification

- Build passes: `cargo build --release -p levitate-kernel --target x86_64-unknown-none`
- Boot test: Serial output shows full boot sequence to shell
- Display: GPU flush now occurs on timer interrupts

## Status: FIXED
