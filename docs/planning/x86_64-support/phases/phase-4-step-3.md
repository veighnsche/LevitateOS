# Phase 4 — Step 3: Testing and Regression Protection

## Parent
[Phase 4: Integration and Testing](phase-4.md)

## Goal
Add automated tests for x86_64 and ensure AArch64 regression protection.

## Prerequisites
- x86_64 boots to userspace
- Existing AArch64 test suite passes

---

## UoW 3.1: Add x86_64 Build Verification Test

**Goal**: Ensure x86_64 kernel and userspace build successfully.

**File**: `xtask/src/tests/build.rs` (new or modify)

**Tasks**:
1. Add test function `test_build_x86_64()`
2. Run `cargo xtask build --arch x86_64`
3. Verify kernel ELF exists
4. Verify userspace binaries exist
5. Add to test suite in `xtask/src/main.rs`

**Exit Criteria**:
- `cargo xtask test build` includes x86_64
- Build failures are caught

**Verification**:
```bash
cargo xtask test build
```

---

## UoW 3.2: Add x86_64 Boot Test

**Goal**: Test that x86_64 kernel boots to `kernel_main`.

**File**: `xtask/src/tests/boot.rs` (new)

**Tasks**:
1. Create boot test for x86_64
2. Start QEMU with x86_64 kernel
3. Capture serial output
4. Look for "x86_64 kernel_main reached" or similar
5. Timeout after 10 seconds if not found
6. Return success/failure

**Exit Criteria**:
- Boot test passes when kernel starts
- Fails on triple fault or hang

**Verification**:
```bash
cargo xtask test boot --arch x86_64
```

---

## UoW 3.3: Add x86_64 Userspace Smoke Test

**Goal**: Test that init runs and produces output.

**File**: `xtask/src/tests/userspace.rs` (new)

**Tasks**:
1. Build x86_64 kernel + userspace + initramfs
2. Start QEMU
3. Capture serial output
4. Look for init's expected output (e.g., "LevitateOS init")
5. Timeout and fail if not found

**Exit Criteria**:
- Userspace runs on x86_64
- Test catches userspace boot failures

**Verification**:
```bash
cargo xtask test userspace --arch x86_64
```

---

## UoW 3.4: Create x86_64 Behavior Golden File

**Goal**: Establish baseline output for regression testing.

**File**: `tests/golden/x86_64_boot.txt` (new)

**Tasks**:
1. Run x86_64 QEMU boot
2. Capture serial output up to shell prompt
3. Sanitize timestamps and variable data
4. Save as golden file
5. Add comparison test in xtask

**Exit Criteria**:
- Golden file represents successful boot
- Diff test catches unexpected changes

**Verification**:
```bash
cargo xtask test behavior --arch x86_64
```

---

## UoW 3.5: Verify AArch64 Tests Still Pass

**Goal**: Confirm no AArch64 regressions from x86_64 work.

**File**: (existing test suite)

**Tasks**:
1. Run full AArch64 test suite
2. Compare results to baseline
3. Fix any unexpected failures
4. Document any expected changes

**Exit Criteria**:
- All AArch64 tests pass
- No regressions introduced

**Verification**:
```bash
cargo xtask test --arch aarch64
```

---

## UoW 3.6: Add CI Matrix for Both Architectures

**Goal**: CI runs tests for both AArch64 and x86_64.

**File**: `.github/workflows/ci.yml` (or equivalent)

**Tasks**:
1. Add matrix strategy with `[aarch64, x86_64]`
2. Run build tests for each arch
3. Run boot tests for each arch
4. Mark build as failed if either arch fails
5. Show which arch failed in CI output

**Exit Criteria**:
- CI tests both architectures
- Both must pass for green build

**Verification**:
- Push and check CI results

---

## Progress Tracking
- [ ] UoW 3.1: Build Verification
- [ ] UoW 3.2: Boot Test
- [ ] UoW 3.3: Userspace Test
- [ ] UoW 3.4: Golden File
- [ ] UoW 3.5: AArch64 Verification
- [ ] UoW 3.6: CI Matrix

## Dependencies Graph
```
UoW 3.1 ──→ UoW 3.2 ──→ UoW 3.3 ──→ UoW 3.4
                                        ↓
UoW 3.5 ──→ UoW 3.6 (requires all above)
```
