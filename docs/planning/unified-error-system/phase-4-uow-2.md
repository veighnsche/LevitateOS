# UoW 2: Add Error Codes to FdtError

**Phase:** 4 - Implementation  
**Parent:** `phase-4.md`  
**Dependencies:** None  
**Estimated Lines:** ~15

---

## Objective

Add error codes (0x09xx) to existing `FdtError` enum without changing API.

---

## Target File

`levitate-hal/src/fdt.rs`

---

## Current State

```rust
#[derive(Debug)]
pub enum FdtError {
    InvalidHeader,
    InitrdMissing,
}
```

---

## Changes Required

### 1. Add derives

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdtError {
```

### 2. Add `code()` and `name()` methods

```rust
impl FdtError {
    pub const fn code(&self) -> u16 {
        match self {
            Self::InvalidHeader => 0x0901,
            Self::InitrdMissing => 0x0902,
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::InvalidHeader => "Invalid DTB header",
            Self::InitrdMissing => "Initrd properties missing in DTB",
        }
    }
}
```

### 3. Add `Display` and `Error` impls

```rust
impl core::fmt::Display for FdtError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "E{:04X}: {}", self.code(), self.name())
    }
}

impl core::error::Error for FdtError {}
```

---

## Verification

1. `cargo build --release`
2. `cargo test -p levitate-hal`
3. No API changes - callers unchanged

---

## Exit Criteria

- [ ] Derives added (Clone, Copy, PartialEq, Eq)
- [ ] `code()` method added
- [ ] `name()` method added
- [ ] `Display` impl added
- [ ] `Error` impl added
- [ ] Build and tests pass
