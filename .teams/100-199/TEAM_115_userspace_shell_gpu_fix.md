# TEAM_115: Userspace Shell GPU Display Fix

## Objective
Fix userspace shell output not appearing on GPU display while still appearing on serial console.

## Root Cause Analysis

### Problem 1: sys_write bypassed dual console
The `sys_write` syscall in `kernel/src/syscall.rs` used:
```rust
let _ = levitate_hal::console::WRITER.lock().write_str(s);
```

This wrote directly to UART, bypassing the `_print()` function which handles the secondary output callback (GPU terminal).

### Problem 2: try_lock() silently dropped output
The `terminal::write_str` function used `try_lock()` for both TERMINAL and GPU locks:
```rust
if let Some(mut term_guard) = TERMINAL.try_lock() {
    if let Some(term) = term_guard.as_mut() {
        if let Some(mut gpu_guard) = crate::gpu::GPU.try_lock() {
```

If either lock was already held (e.g., by timer interrupt), the output was silently discarded.

## Fixes Applied

### 1. kernel/src/syscall.rs
Changed sys_write to use `print!()` macro instead of direct UART write:
```rust
// TEAM_115: Use print! macro to go through dual console path (UART + GPU)
print!("{}", s);
```

### 2. kernel/src/terminal.rs
Changed `write_str` to use blocking `lock()` and added GPU flush:
```rust
pub fn write_str(s: &str) {
    let mut term_guard = TERMINAL.lock();
    if let Some(term) = term_guard.as_mut() {
        let mut gpu_guard = crate::gpu::GPU.lock();
        if let Some(gpu_state) = gpu_guard.as_mut() {
            let mut display = crate::gpu::Display::new(gpu_state);
            term.write_str(&mut display, s);
            // TEAM_115: Flush immediately so output is visible
            let _ = gpu_state.flush();
        }
    }
}
```

### 3. xtask/src/main.rs
Increased GPU resolution from 1280x800 to 1920x1080.

### 4. xtask/src/tests/behavior.rs
- Added regex normalization for ELF stack addresses (vary between builds)
- Added regex dependency to xtask/Cargo.toml

### 5. xtask/src/tests/regression.rs
Updated GPU display test to check `levitate-gpu` crate (replaced deleted `levitate-drivers-gpu`).

## Verification
- [x] `cargo xtask test all` - All 79 unit tests pass
- [x] `cargo xtask test behavior` - Golden file matches
- [x] `cargo xtask run-vnc` - Shell visible on GPU via browser VNC
- [x] VNC interactive test - `echo hello` works

## Behavior Inventory Updates
Added Group 12: Userspace Shell with 24 behaviors:
- 9 syscall behaviors (SYS1-SYS9)
- 4 terminal GPU behaviors (GPU1-GPU4)
- 4 user process behaviors (PROC1-PROC4)
- 7 shell behaviors (SH1-SH7)

## ROADMAP Updates
Marked Phase 8b as COMPLETED.

## Files Modified
- `kernel/src/syscall.rs` - Use print! for dual console
- `kernel/src/terminal.rs` - Blocking locks + GPU flush
- `xtask/src/main.rs` - 1920x1080 resolution
- `xtask/src/tests/behavior.rs` - ELF address normalization
- `xtask/src/tests/regression.rs` - Updated GPU test
- `xtask/Cargo.toml` - Added regex dependency
- `docs/testing/behavior-inventory.md` - Group 12
- `docs/ROADMAP.md` - Phase 8b complete

## Handoff
Phase 8b (Interactive Shell) is now complete. The remaining task is Spawn Syscall for executing external programs, which can be tackled in Phase 8c or Phase 9.
