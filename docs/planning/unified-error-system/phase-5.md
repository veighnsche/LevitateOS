# Phase 5: Cleanup, Regression Protection, and Handoff

**Bug:** Inconsistent error handling across LevitateOS  
**Author:** TEAM_151  
**Status:** Ready after Phase 4

---

## Overview

This phase ensures the fix is complete, tested, and documented for future teams.

---

## Step 1: Verification

### 1.1 Build Verification
```bash
cargo build --release
cargo test -p levitate-hal -p levitate-utils -p xtask
```

### 1.2 Panic Audit
```bash
# Verify no new panics in converted files
grep -n "panic!\|unwrap()\|expect(" kernel/src/loader/elf.rs
grep -n "panic!\|unwrap()\|expect(" kernel/src/task/process.rs
grep -n "panic!\|unwrap()\|expect(" kernel/src/task/user_mm.rs
grep -n "panic!\|unwrap()\|expect(" kernel/src/fs/*.rs
grep -n "panic!\|unwrap()\|expect(" kernel/src/net.rs
grep -n "panic!\|unwrap()\|expect(" levitate-hal/src/mmu.rs
grep -n "panic!\|unwrap()\|expect(" levitate-hal/src/fdt.rs
```

### 1.3 Error Code Uniqueness
```bash
# Extract all error codes and verify uniqueness
grep -roh "0x0[0-9A-Fa-f]\{3\}" kernel/src/ levitate-hal/src/ | sort | uniq -d
# Should output nothing (no duplicates)
```

---

## Step 2: Regression Test Updates

### 2.1 Golden Test Check

Run the boot regression test:
```bash
cargo xtask test
```

If error message format changed in boot output, update `tests/golden_boot.txt` with USER approval.

### 2.2 Unit Test Addition

Add tests for each new error type (if not done in UoWs):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes_unique() {
        // Collect all codes, verify no duplicates
    }

    #[test]
    fn test_display_format() {
        let err = MmuError::NotMapped;
        let s = format!("{}", err);
        assert!(s.starts_with("E0102:"));
    }
}
```

---

## Step 3: Documentation

### 3.1 Update Architecture Docs

Add to `docs/ARCHITECTURE.md`:

```markdown
## Error Handling

LevitateOS uses typed error enums with numeric codes for debugging:

### Error Code Format
```
0xSSCC where:
  SS = Subsystem (00-FF)
  CC = Error code within subsystem (00-FF)
```

### Subsystem Allocation
| Range | Subsystem |
|-------|-----------|
| 0x01xx | MMU |
| 0x02xx | ELF |
| 0x03xx | Process |
| 0x05xx | Filesystem |
| 0x06xx | Block |
| 0x07xx | Network |
| 0x09xx | FDT |

### Pattern for New Error Types
Every error type must implement:
- `code() -> u16`
- `name() -> &'static str`
- `Display` (format: "E{code:04X}: {name}")
- `Error` trait
```

### 3.2 Remove Breadcrumbs

Remove any investigation breadcrumbs left in code during Phase 2.

---

## Step 4: Handoff

### 4.1 Update Team File

Mark all phases complete in `.teams/TEAM_XXX_*.md`

### 4.2 Update Success Criteria

In `docs/planning/unified-error-system/plan.md`:

```markdown
## Success Criteria

- [x] All `&'static str` errors replaced with typed enums
- [x] All errors have unique numeric codes
- [x] `block.rs` panics replaced with Result
- [x] All tests pass
- [x] Error output includes codes: `E0102: Page not mapped`
```

### 4.3 Handoff Checklist

- [ ] Project builds cleanly
- [ ] All tests pass
- [ ] Golden boot test passes
- [ ] No duplicate error codes
- [ ] Documentation updated
- [ ] Team file complete

---

## Exit Criteria for Phase 5

- [ ] All verification checks pass
- [ ] Tests added/updated as needed
- [ ] Documentation updated
- [ ] Handoff complete
- [ ] Plan marked as DONE
