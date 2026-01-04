# Phase 4 — Integration and Testing

**Feature**: Reference Kernel Improvements  
**Team**: TEAM_043  
**Status**: Blocked (waiting for Phase 3)  
**Parent**: phase-3.md

---

## Integration Testing Strategy

### Test Levels

| Level | What | How |
|-------|------|-----|
| Unit | Individual functions | `cargo test` in levitate-hal |
| Integration | Module interactions | Behavior test suite |
| System | Full boot | QEMU boot test |
| Regression | No breakage | Golden log comparison |

---

## Step 1: Unit Test Coverage

### FDT Module Tests

```rust
#[test]
fn test_parse_valid_fdt() { ... }

#[test]
fn test_find_compatible_gic_v3() { ... }

#[test]
fn test_find_compatible_gic_v2() { ... }

#[test]
fn test_get_reg_address() { ... }

#[test]
fn test_parse_invalid_magic() { ... }
```

### GIC Module Tests

```rust
#[test]
fn test_gic_config_from_fdt_v3() { ... }

#[test]
fn test_gic_config_from_fdt_v2() { ... }

#[test]
fn test_gic_fallback_no_fdt() { ... }
```

### IRQ Module Tests

```rust
#[test]
fn test_handler_registration() { ... }

#[test]
fn test_handler_dispatch() { ... }

#[test]
fn test_unregistered_irq() { ... }
```

---

## Step 2: Behavior Test Updates

### Golden Log Changes

If boot log format changes, update `tests/golden_boot.txt`:

```diff
- Core drivers initialized.
+ GIC: Detected GICv3 from FDT
+ Core drivers initialized.
```

### New Behavior Tests

| Test | Purpose |
|------|---------|
| `test_gicv3_boot` | Verify GICv3 detection and init |
| `test_gicv2_fallback` | Verify GICv2 works without FDT |
| `test_pixel6_profile` | Full Pixel 6 config with cluster topology |

---

## Step 3: System Integration Tests

### QEMU Configurations to Test

| Config | Machine | GIC | SMP | Expected |
|--------|---------|-----|-----|----------|
| Default | virt | v2 | 1 | Pass |
| Pixel6 | virt,gic-version=3 | v3 | 8 clusters | Pass |
| No FDT | virt | v2 | 1 | Pass (fallback) |

### Test Commands

```bash
# Default profile
cargo xtask run --profile default --headless

# Pixel 6 profile with GICv3
cargo xtask run --profile pixel6 --headless

# Full test suite
cargo xtask test all
```

---

## Step 4: Regression Protection

### Behavioral Baselines (Rule 4)

Ensure these baselines still pass:

1. **Boot sequence**: `tests/golden_boot.txt`
2. **Timer interrupts**: Timer fires at expected rate
3. **UART output**: Console output works
4. **Memory mapping**: Kernel text accessible

### Regression Test Checklist

- [ ] Unit tests: All pass
- [ ] Behavior test: Golden log matches
- [ ] Regression tests: All 3 pass
- [ ] Pixel 6 boot: Successful with new GIC detection
- [ ] Default boot: No changes from before

---

## Step 5: Performance Validation

### Measurements

| Metric | Before | After | Acceptable Delta |
|--------|--------|-------|------------------|
| Boot time | TBD | TBD | < 10% increase |
| FDT parse time | N/A | TBD | < 10ms |
| First interrupt | TBD | TBD | No change |

### How to Measure

```rust
// Add timing around FDT parse
let start = timer::read_counter();
let fdt = fdt::parse(fdt_addr)?;
let elapsed = timer::read_counter() - start;
println!("FDT parse: {} cycles", elapsed);
```

---

## Acceptance Criteria

- [ ] All unit tests pass
- [ ] Behavior test passes (golden log)
- [ ] Regression tests pass
- [ ] GICv3 works on Pixel 6 profile
- [ ] GICv2 still works on default profile
- [ ] No performance regression
- [ ] Code coverage for new modules > 70%

---

## Next Phase

After testing passes, proceed to **Phase 5 — Polish, Docs, and Cleanup**.
