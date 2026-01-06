# UoW 7: Add Error Codes to NetError

**Phase:** 4 - Implementation  
**Parent:** `phase-4.md`  
**Dependencies:** None  
**Estimated Lines:** ~20

---

## Objective

Add error codes (0x07xx) to existing `NetError` enum without changing API.

---

## Target File

`kernel/src/net.rs`

---

## Current State

```rust
#[allow(dead_code)]
#[derive(Debug)]
pub enum NetError {
    NotInitialized,
    DeviceBusy,
    SendFailed,
}
```

---

## Changes Required

### 1. Add derives and remove dead_code

```rust
/// TEAM_XXX: Network error type with error codes (0x07xx)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetError {
    /// Device not initialized (0x0701)
    NotInitialized,
    /// TX queue is full (0x0702)
    DeviceBusy,
    /// Transmission failed (0x0703)
    SendFailed,
}
```

### 2. Add code() and name() methods

```rust
impl NetError {
    pub const fn code(&self) -> u16 {
        match self {
            Self::NotInitialized => 0x0701,
            Self::DeviceBusy => 0x0702,
            Self::SendFailed => 0x0703,
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::NotInitialized => "Network device not initialized",
            Self::DeviceBusy => "TX queue full",
            Self::SendFailed => "Transmission failed",
        }
    }
}
```

### 3. Add Display and Error impls

```rust
impl core::fmt::Display for NetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "E{:04X}: {}", self.code(), self.name())
    }
}

impl core::error::Error for NetError {}
```

---

## Verification

1. `cargo build --release`
2. No API changes - callers unchanged

---

## Exit Criteria

- [ ] Derives added (Clone, Copy, PartialEq, Eq)
- [ ] `code()` method added
- [ ] `name()` method added
- [ ] `Display` impl added
- [ ] `Error` impl added
- [ ] Build passes
