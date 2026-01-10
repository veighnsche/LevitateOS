# Test Suite Reorganization Plan

**TEAM_327** | **Date:** 2026-01-09

## Problem Statement

The `xtask/src/tests/` directory is messy:
- **Overlap:** `serial_input.rs`, `keyboard_input.rs`, `shutdown.rs` all duplicate QEMU setup
- **Gaps:** Screenshot tests don't run userspace tests or capture results
- **Inconsistency:** Mixed patterns (some use QemuBuilder, some don't)

## Current Structure (10 files)

| File | Purpose | Issues |
|------|---------|--------|
| `behavior.rs` | Boot log golden test | OK |
| `debug_tools.rs` | QMP debug commands | OK (uses bare QEMU) |
| `keyboard_input.rs` | Keyboard echo test | Duplicates QEMU setup |
| `serial_input.rs` | Serial echo test | Duplicates QEMU setup |
| `shutdown.rs` | Shutdown golden test | Duplicates QEMU setup |
| `screenshot_alpine.rs` | Alpine reference test | OK |
| `screenshot_levitate.rs` | Display test | Only waits 15s, no interaction |
| `regression.rs` | Static code analysis | OK |
| `unit.rs` | Cargo unit tests | OK |

## New Structure (7 files)

```
tests/
├── mod.rs              # Module exports + dispatcher
├── README.md           # Updated docs
├── common.rs           # NEW: Shared QemuSession, send_keys, etc.
├── golden.rs           # Boot + shutdown golden tests
├── input.rs            # Keyboard + serial input tests (consolidated)
├── screenshot.rs       # All visual tests (alpine + levitate + userspace)
├── regression.rs       # Static analysis (unchanged)
├── unit.rs             # Cargo tests (unchanged)
└── debug.rs            # QMP debug tools (renamed from debug_tools.rs)
```

## Key Changes

### 1. `common.rs` - Shared Infrastructure
- `QemuSession` struct wrapping child process + stdin/stdout
- `send_keys()`, `send_key()`, `wait_for_prompt()` helpers
- Non-blocking read utilities
- Screenshot via QMP helpers

### 2. `screenshot.rs` - Unified Visual Tests
Consolidates alpine + levitate + NEW userspace test execution:
- `cargo xtask test screenshot alpine` → Alpine reference tests
- `cargo xtask test screenshot levitate` → Basic display test  
- `cargo xtask test screenshot userspace` → **NEW: Run tests, capture results**

### 3. `input.rs` - Consolidated Input Tests
Merges `serial_input.rs` + `keyboard_input.rs`:
- Uses shared `QemuSession` from `common.rs`
- Single QEMU boot, multiple input tests

### 4. `golden.rs` - Golden File Tests
Merges `behavior.rs` + `shutdown.rs`:
- Boot log comparison
- Shutdown sequence comparison
- Shared infrastructure

## Userspace Screenshot Test Flow

```
1. Build LevitateOS (aarch64)
2. Start QEMU with VNC + QMP
3. Wait for shell prompt
4. Run test commands:
   - echo "=== TEST RESULTS ==="
   - ls /
   - help
   - (future: run actual test binaries)
5. Take screenshot showing results
6. Save to tests/screenshots/levitate_userspace.png
```

## Implementation Order

1. Create `common.rs` with shared utilities
2. Create new `screenshot.rs` with userspace test support
3. Consolidate `input.rs` from serial + keyboard
4. Consolidate `golden.rs` from behavior + shutdown
5. Update `mod.rs` and test dispatcher
6. Delete old files
7. Update README.md

## Commands After Reorganization

| Command | Description |
|---------|-------------|
| `cargo xtask test` | Run all tests |
| `cargo xtask test unit` | Cargo unit tests |
| `cargo xtask test golden` | Boot + shutdown golden tests |
| `cargo xtask test input` | Keyboard/serial input tests |
| `cargo xtask test screenshot` | All screenshot tests |
| `cargo xtask test screenshot userspace` | Userspace tests + screenshot |
| `cargo xtask test debug` | QMP debug tools |
| `cargo xtask test regress` | Static analysis |
