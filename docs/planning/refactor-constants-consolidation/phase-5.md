# Phase 5: Hardening

## Final Verification

### Compilation Tests

```bash
# Clean build from scratch
cargo clean
cargo xtask build all

# Both architectures
cargo xtask build kernel
cargo xtask --arch aarch64 build kernel

# All tests
cargo xtask test
cargo xtask test behavior
cargo xtask test unit
```

### Runtime Verification

Boot and verify shell works:

```bash
cargo xtask run --term
# In shell:
echo "test"
ls /
cat /root/hello.txt
```

### Static Analysis

```bash
# No dead code
cargo clippy -- -W dead_code 2>&1 | grep "dead_code" | wc -l
# Should be 0 or only known acceptable warnings

# No magic numbers remain (spot check)
grep -rn "0xFFF\|0x1000" crates/kernel/levitate/src/ | wc -l
# Should be 0 (all replaced with constants)

# PAGE_SIZE only defined once
grep -rn "pub const PAGE_SIZE" crates/kernel/ | wc -l
# Should be 1 (in hal/src/mem/constants.rs)
```

### Consistency Checks

```bash
# Stack size only defined once
grep -rn "STACK_SIZE.*=" crates/kernel/ | grep "pub const" | wc -l
# Should be 1

# Heap size only defined once
grep -rn "HEAP_MAX.*=" crates/kernel/ | grep "pub const" | wc -l
# Should be 1

# Custom syscalls match between architectures
# (After unification, both should import from same source)
diff <(grep "Spawn\|SpawnArgs\|SetForeground" crates/kernel/arch/aarch64/src/lib.rs) \
     <(grep "Spawn\|SpawnArgs\|SetForeground" crates/kernel/arch/x86_64/src/lib.rs)
# Should show only enum vs import differences, same values
```

## Documentation Updates

### Update CLAUDE.md

Add section about constants:

```markdown
### Constants Location

All kernel-wide constants are defined in centralized locations:

| Constant Type | Location | Examples |
|---------------|----------|----------|
| Memory | `los_hal::mem::constants` | PAGE_SIZE, page_align_up() |
| Process limits | `los_mm::user::limits` | USER_STACK_SIZE, MAX_FDS |
| Custom syscalls | `los_syscall::custom` | SYS_SPAWN, SYS_ISATTY |

**Never** define PAGE_SIZE or alignment constants locally. Import from HAL.
```

### Update Error Handling Docs

If any new error types were added, update `docs/planning/error-macro/phase-1.md`.

### Update Architecture Docs

If memory layout changed, update `docs/ARCHITECTURE.md`.

## Handoff Notes

### What Changed

1. **Constants consolidation**: PAGE_SIZE and related constants now in `los_hal::mem::constants`
2. **Alignment helpers**: New `page_align_up()`, `page_align_down()`, `is_page_aligned()`
3. **Process limits**: Unified in `los_mm::user::limits`
4. **Custom syscalls**: Shared definition prevents aarch64/x86_64 desync

### Migration Guide for Future Code

When writing new kernel code:

```rust
// DO: Import from centralized location
use los_hal::mem::constants::{PAGE_SIZE, page_align_up};

// DON'T: Define locally
const PAGE_SIZE: usize = 4096;  // NO!

// DO: Use alignment helpers
let aligned = page_align_up(addr);

// DON'T: Use magic numbers
let aligned = (addr + 0xFFF) & !0xFFF;  // NO!
```

### Known Limitations

1. **Custom syscalls**: Still temporary, should be migrated to clone+execve
2. **Tmpfs limits**: Still hardcoded (16MB/64MB), should be dynamic % of RAM
3. **GDT selectors**: x86_64 specific, not abstracted for other architectures

### Future Work

- Make tmpfs limits dynamic based on available RAM
- Remove custom spawn syscalls after clone+execve works
- Add compile-time assertions that constants match expected values

## Exit Checklist

- [ ] All tests pass
- [ ] Both architectures build
- [ ] No duplicate constant definitions
- [ ] No magic numbers in page alignment code
- [ ] Documentation updated
- [ ] Team file complete with handoff notes
- [ ] CLAUDE.md updated with constants location
- [ ] No compatibility shims exist
