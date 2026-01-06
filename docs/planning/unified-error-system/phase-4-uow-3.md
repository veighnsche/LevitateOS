# UoW 3: Create MmuError Type

**Phase:** 4 - Implementation  
**Parent:** `phase-4.md`  
**Dependencies:** None  
**Estimated Lines:** ~80

---

## Objective

Replace `&'static str` errors in `levitate-hal/src/mmu.rs` with typed `MmuError` enum.

---

## Target File

`levitate-hal/src/mmu.rs`

---

## Current String Errors (from grep)

```rust
// map_page, unmap_page, etc return Result<(), &'static str>
"Page table allocation failed"
"VA not 2MB aligned for block mapping"
"PA not 2MB aligned for block mapping"
"Address not mapped"
// ... more in walk_to_entry
```

---

## Changes Required

### 1. Create MmuError enum (add near top of file)

```rust
/// TEAM_XXX: MMU error type with error codes (0x01xx)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmuError {
    /// Page table allocation failed (0x0101)
    AllocationFailed,
    /// Virtual address not mapped (0x0102)
    NotMapped,
    /// Invalid virtual address (0x0103)
    InvalidVirtualAddress,
    /// Address not aligned (0x0104)
    Misaligned,
    /// Walk failed - level not reached (0x0105)
    WalkFailed,
}

impl MmuError {
    pub const fn code(&self) -> u16 {
        match self {
            Self::AllocationFailed => 0x0101,
            Self::NotMapped => 0x0102,
            Self::InvalidVirtualAddress => 0x0103,
            Self::Misaligned => 0x0104,
            Self::WalkFailed => 0x0105,
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::AllocationFailed => "Page table allocation failed",
            Self::NotMapped => "Address not mapped",
            Self::InvalidVirtualAddress => "Invalid virtual address",
            Self::Misaligned => "Address not properly aligned",
            Self::WalkFailed => "Page table walk failed",
        }
    }
}

impl core::fmt::Display for MmuError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "E{:04X}: {}", self.code(), self.name())
    }
}

impl core::error::Error for MmuError {}
```

### 2. Update function signatures

**Before:**
```rust
pub fn map_page(...) -> Result<(), &'static str>
pub fn unmap_page(...) -> Result<(), &'static str>
pub fn map_block_2mb(...) -> Result<(), &'static str>
```

**After:**
```rust
pub fn map_page(...) -> Result<(), MmuError>
pub fn unmap_page(...) -> Result<(), MmuError>
pub fn map_block_2mb(...) -> Result<(), MmuError>
```

### 3. Update error returns

**Before:**
```rust
Err("Page table allocation failed")
```

**After:**
```rust
Err(MmuError::AllocationFailed)
```

---

## Functions to Update

1. `map_page()` - line ~727
2. `unmap_page()` - line ~745
3. `map_block_2mb()` - line ~872
4. `map_range()` - line ~894
5. `identity_map_range()` - line ~841
6. `get_or_create_table()` - line ~800
7. `walk_to_entry()` - line ~620

---

## Verification

1. `cargo build --release`
2. `cargo test -p levitate-hal`
3. Check callers compile (kernel/src/boot.rs, etc.)

---

## Exit Criteria

- [ ] `MmuError` enum created with 5 variants
- [ ] All MMU functions return `Result<(), MmuError>`
- [ ] All string errors replaced
- [ ] Build passes
- [ ] Tests pass
