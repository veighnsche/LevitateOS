# Phase 4: Integration & Testing - x86_64 Behavior Test Completion

## Prerequisites

- [ ] Phase 3 implementation complete
- [ ] Kernel boots on x86_64 via Limine without page faults
- [ ] HHDM working for MMIO access

## Integration Steps

### Step 1: Run Full Test Suite

**Goal**: Verify no regressions on any architecture.

**Commands**:
```bash
# Unit tests
cargo test --workspace

# aarch64 behavior test
cargo xtask test --behavior --arch aarch64

# x86_64 behavior test
cargo xtask test --behavior --arch x86_64
```

**Success Criteria**:
- All unit tests pass
- aarch64 behavior test unchanged
- x86_64 behavior test passes

---

### Step 2: Verify Golden File Accuracy

**Goal**: Ensure x86_64 golden file captures correct boot sequence.

**Tasks**:
1. Run behavior test multiple times
2. Verify output is deterministic
3. Confirm masking covers all variable content
4. Review golden file for completeness

**Verification**:
```bash
# Run 3 times, compare outputs
for i in 1 2 3; do
    cargo xtask test --behavior --arch x86_64 2>&1 | tee run_$i.txt
done
diff run_1.txt run_2.txt
diff run_2.txt run_3.txt
```

---

### Step 3: Cross-Architecture Validation

**Goal**: Ensure changes don't affect aarch64.

**Tasks**:
1. Review all modified files for arch-specific guards
2. Run aarch64 build and boot
3. Compare aarch64 behavior test output with baseline

---

### Step 4: Edge Case Testing

**Goal**: Verify error handling and fallbacks.

**Scenarios**:
| Scenario | Expected Result |
|----------|-----------------|
| HHDM unavailable | Clear panic message |
| Invalid ECAM access | Page fault with diagnostic |
| Missing golden file | Test fails with helpful error |

---

## Regression Protection

### Behavioral Baselines

| Test | Baseline File |
|------|---------------|
| aarch64 boot | `tests/golden_boot.txt` |
| x86_64 boot | `tests/golden_boot_x86_64.txt` |

### Before Merging

1. Run all unit tests
2. Run both architecture behavior tests
3. Compare outputs with baselines
4. Document any intentional differences
