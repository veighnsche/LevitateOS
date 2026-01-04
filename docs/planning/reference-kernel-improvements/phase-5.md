# Phase 5 — Polish, Docs, and Cleanup

**Feature**: Reference Kernel Improvements  
**Team**: TEAM_043  
**Status**: Blocked (waiting for Phase 4)  
**Parent**: phase-4.md

---

## Step 1: Code Cleanup

### Remove Dead Code (Rule 6)

After integration, remove:

- [ ] Old `detect_gic_version()` if replaced by FDT
- [ ] Unused GIC constants (GICD_IIDR, etc.) if not needed
- [ ] Old handler registration code after trait migration
- [ ] Any `#[allow(dead_code)]` that's no longer needed

### Code Quality

- [ ] Run `cargo clippy` and fix warnings
- [ ] Run `cargo fmt` for consistent formatting
- [ ] Remove any TODO comments that are now done
- [ ] Ensure all TEAM_043 comments are accurate

---

## Step 2: Documentation Updates

### Update QEMU_PROFILES.md

```markdown
## GICv3 Support

LevitateOS now supports GICv3 via FDT detection:

- **Detection**: Parses DTB for `compatible = "arm,gic-v3"`
- **Fallback**: Uses GICv2 if no FDT or GICv2 compatible found
- **Pixel 6 Profile**: Now uses GICv3 with cluster topology
```

### Update README.md

Add section on hardware abstraction:

```markdown
## Hardware Abstraction

LevitateOS uses Device Tree for hardware discovery:

- FDT parsing at boot
- Automatic GIC version detection
- Timer mode selection based on VHE
```

### Create API Documentation

Document new public APIs:

- `fdt::parse()` - Usage and error handling
- `irq::register_handler()` - How to add new handlers
- `gic::init_from_fdt()` - GIC initialization flow

---

## Step 3: Update Auxiliary Files

### run-pixel6.sh

```bash
# Update to reflect GICv3 usage
QEMU_MACHINE="virt,gic-version=3"
QEMU_SMP="sockets=1,clusters=2,cores=4,threads=1"
echo "GIC:    v3 (auto-detected from FDT)"
```

### qemu/pixel6.conf

```ini
# GICv3 now properly detected via FDT
QEMU_MACHINE="virt,gic-version=3"
```

### docs/ARCHITECTURE.md (if exists)

Add section on device discovery and HAL structure.

---

## Step 4: Team Handoff

### Update Team File

Add completion summary to `.teams/TEAM_043_feature_reference_kernel_improvements.md`:

```markdown
## Completion Summary

### Implemented
1. FDT parsing for device discovery
2. GICv3 detection via FDT
3. InterruptHandler trait for IRQ handlers
4. bitflags! for register manipulation
5. VHE detection for timer optimization

### Test Results
- Unit tests: X passed
- Behavior tests: Passed
- Regression tests: 3/3 passed
- GICv3 boot: Working

### Files Added
- levitate-hal/src/fdt.rs
- levitate-hal/src/irq.rs

### Files Modified
- levitate-hal/src/gic.rs
- levitate-hal/src/timer.rs
- kernel/src/main.rs
- kernel/src/exceptions.rs
```

### Create Handoff Notes

Document anything the next team should know:

- Known limitations
- Future improvement ideas
- Gotchas discovered during implementation

---

## Step 5: Final Verification

### Checklist

- [ ] All tests pass
- [ ] No compiler warnings
- [ ] No clippy warnings
- [ ] Documentation complete
- [ ] Team file updated
- [ ] Auxiliary files updated
- [ ] No dead code remaining
- [ ] All TODO items resolved or documented

### Final Test Run

```bash
# Complete test suite
cargo xtask test all

# Verify Pixel 6 profile
cargo xtask run --profile pixel6 --headless

# Check for warnings
cargo build --release 2>&1 | grep -i warning
```

---

## Feature Complete Criteria

The feature is complete when:

1. ✅ All 5 improvements implemented
2. ✅ All tests pass
3. ✅ Documentation updated
4. ✅ Code cleaned up
5. ✅ Team handoff complete
6. ✅ No regressions from baseline

---

## Post-Completion

After this feature is complete:

1. Archive planning files (keep for reference)
2. Update main project overview
3. Consider follow-up features:
   - More device drivers via FDT
   - Multi-core secondary CPU init
   - Power management via PSCI
