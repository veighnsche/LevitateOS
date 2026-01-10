# Phase 4: Integration & Testing — x86_64 MMU Completion

## Overview

This phase verifies that all Phase 3 implementations work together and don't regress existing functionality.

---

## Step 1: Verify HAL Tests

### Tasks
1. Run full HAL test suite:
   ```bash
   cargo test --package los_hal --features std
   ```
2. Confirm all 25 tests pass (including previously failing `test_irq_safe_lock_*`)
3. Document any new warnings

### Exit Criteria
- [ ] All HAL unit tests pass
- [ ] No new warnings introduced

---

## Step 2: Verify x86_64 Kernel Build

### Tasks
1. Build x86_64 kernel:
   ```bash
   cargo xtask build --arch x86_64
   ```
2. Verify no linker errors about missing symbols
3. Verify kernel binary is produced

### Exit Criteria
- [ ] x86_64 kernel builds successfully
- [ ] No linker symbol errors
- [ ] Binary produced in target directory

---

## Step 3: Verify aarch64 Not Regressed

### Tasks
1. Build aarch64 kernel:
   ```bash
   cargo xtask build --arch aarch64
   ```
2. Run behavior tests:
   ```bash
   cargo xtask test behavior
   ```
3. Run golden boot test:
   ```bash
   cargo xtask test regress
   ```

### Exit Criteria
- [ ] aarch64 kernel builds successfully
- [ ] Behavior tests pass
- [ ] Golden boot test passes

---

## Step 4: Manual Boot Verification (x86_64)

### Tasks
1. Run x86_64 kernel in QEMU:
   ```bash
   cargo xtask run --arch x86_64
   ```
2. Verify boot output shows:
   - "OK" on VGA (proves early boot works)
   - Multiboot2 info parsed message
   - PMO mapping message
   - Memory allocator initialization message
3. Verify no crashes during boot

### Exit Criteria
- [ ] x86_64 kernel boots in QEMU
- [ ] Expected boot messages appear
- [ ] No crashes or hangs

---

## Regression Protection

### Behavioral Baselines to Protect
- aarch64 golden boot sequence (`tests/golden_boot.txt`)
- aarch64 behavior tests

### New Verification Added
- HAL unit tests now include interrupt mock testing
- x86_64 build is verified as part of CI (future)

---

## Phase 4 Exit Criteria

- [ ] All HAL tests pass
- [ ] x86_64 kernel builds and boots
- [ ] aarch64 not regressed
- [ ] Documentation updated
- [ ] → Feature complete
