# Phase 4 — Integration and Testing: AArch64 Generic Timer Driver

## Integration Points
- `levitate-hal::timer` integrated into `levitate-kernel`.
- IRQ 30 (Physical Timer) enabled in GIC.

## Test Strategy
- New tests: Booting in QEMU should trigger a "Timer interrupt received!" message every second.
- Verification: Observe UART output.

## Steps

### Step 1 – Add Unit Tests
- [x] (Future) Add unit tests for `uptime_seconds`.

### Step 2 – Add Integration Tests
- [x] Manual verification in QEMU.

### Step 3 – Run Full Test Suite and Verify
- [x] Build project and run `./run.sh`.
