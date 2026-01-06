# Phase 4: Integration and Testing

**Feature:** `define_kernel_error!` Macro  
**Author:** TEAM_153  
**Status:** Ready after Phase 3

---

## Overview

This phase ensures the macro works correctly and documents the migration path for remaining error types.

---

## Step 1: Unit Tests for Macro

**File:** `levitate-hal/src/error.rs` (add tests at bottom)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    define_kernel_error! {
        /// Test error type
        pub enum TestError(0xFF) {
            /// First error
            First = 0x01 => "First error",
            /// Second error  
            Second = 0x02 => "Second error",
        }
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(TestError::First.code(), 0xFF01);
        assert_eq!(TestError::Second.code(), 0xFF02);
    }

    #[test]
    fn test_error_names() {
        assert_eq!(TestError::First.name(), "First error");
        assert_eq!(TestError::Second.name(), "Second error");
    }

    #[test]
    fn test_display_format() {
        assert_eq!(format!("{}", TestError::First), "E FF01: First error");
        assert_eq!(format!("{}", TestError::Second), "EFF02: Second error");
    }

    #[test]
    fn test_subsystem_constant() {
        assert_eq!(TestError::SUBSYSTEM, 0xFF);
    }

    #[test]
    fn test_derives() {
        // Clone
        let e = TestError::First;
        let e2 = e.clone();
        assert_eq!(e, e2);

        // Copy
        let e3 = e;
        assert_eq!(e, e3);

        // Debug
        let debug_str = format!("{:?}", TestError::First);
        assert!(debug_str.contains("First"));
    }
}
```

---

## Step 2: Migration Verification

After migrating `FdtError`, verify:

```bash
# Build passes
cargo build --release

# Tests pass
cargo test -p levitate-hal

# Behavior identical
# (manual inspection of test output)
```

---

## Step 3: Migration Path for Remaining Types

### Priority Order (simplest first)

| Priority | Error Type | Complexity | Notes |
|----------|------------|------------|-------|
| 1 | `FdtError` | Simple | 2 variants, done in Phase 3 |
| 2 | `NetError` | Simple | 3 variants |
| 3 | `BlockError` | Simple | 4 variants |
| 4 | `MmuError` | Simple | 5 variants |
| 5 | `ElfError` | Medium | 9 variants |
| 6 | `SpawnError` | Complex | Has nested errors - manual |
| 7 | `FsError` | Complex | Has nested errors - manual |

### Nested Error Types (Keep Manual)

`SpawnError` and `FsError` have variants with inner errors:

```rust
pub enum SpawnError {
    Elf(ElfError),      // Nested
    PageTable(MmuError), // Nested
    Stack(MmuError),     // Nested
}
```

These require custom `Display` impls to show inner error. Keep manual implementation for now.

**Future consideration:** Extend macro syntax for nested errors if pattern becomes common.

---

## Step 4: Documentation Update

Update `docs/ARCHITECTURE.md` with error handling section:

```markdown
## Error Handling

LevitateOS uses typed error enums with numeric codes for debugging.

### Defining New Error Types

Use the `define_kernel_error!` macro for simple error types:

\`\`\`rust
use levitate_hal::define_kernel_error;

define_kernel_error! {
    /// My subsystem errors (0x10xx)
    pub enum MyError(0x10) {
        /// Something went wrong
        SomethingWrong = 0x01 => "Something went wrong",
    }
}
\`\`\`

For error types with nested errors, implement manually following the pattern
in `kernel/src/task/process.rs`.

### Error Code Format

\`\`\`
0xSSCC where:
  SS = Subsystem (from phase-3.md canonical list)
  CC = Error code within subsystem (00-FF)
\`\`\`

### Subsystem Allocation

See `docs/planning/unified-error-system/phase-3.md` for canonical list.
```

---

## Exit Criteria for Phase 4

- [ ] Unit tests added and passing
- [ ] FdtError migration verified
- [ ] Migration path documented
- [ ] ARCHITECTURE.md updated
- [ ] All existing tests still pass

---

## Handoff Checklist

- [ ] Project builds cleanly
- [ ] All tests pass
- [ ] Macro documented with examples
- [ ] Migration path clear for future teams
- [ ] Team file updated
