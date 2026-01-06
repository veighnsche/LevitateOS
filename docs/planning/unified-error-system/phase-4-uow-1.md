# UoW 1: Add Error Codes to ElfError

**Phase:** 4 - Implementation  
**Parent:** `phase-4.md`  
**Dependencies:** None  
**Estimated Lines:** ~30

---

## Objective

Add error codes (0x02xx) to existing `ElfError` enum without changing API.

---

## Target File

`kernel/src/loader/elf.rs`

---

## Current State

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfError {
    TooShort,
    InvalidMagic,
    Not64Bit,
    NotLittleEndian,
    NotExecutable,
    WrongArchitecture,
    InvalidProgramHeader,
    AllocationFailed,
    MappingFailed,
}
```

---

## Changes Required

### 1. Add `code()` method

```rust
impl ElfError {
    /// TEAM_XXX: Get numeric error code for debugging
    pub const fn code(&self) -> u16 {
        match self {
            Self::TooShort => 0x0201,
            Self::InvalidMagic => 0x0202,
            Self::Not64Bit => 0x0203,
            Self::NotLittleEndian => 0x0204,
            Self::NotExecutable => 0x0205,
            Self::WrongArchitecture => 0x0206,
            Self::InvalidProgramHeader => 0x0207,
            Self::AllocationFailed => 0x0208,
            Self::MappingFailed => 0x0209,
        }
    }

    /// TEAM_XXX: Get error name for logging
    pub const fn name(&self) -> &'static str {
        match self {
            Self::TooShort => "ELF data too short",
            Self::InvalidMagic => "Invalid ELF magic number",
            Self::Not64Bit => "Not a 64-bit ELF",
            Self::NotLittleEndian => "Not little-endian",
            Self::NotExecutable => "Not an executable file",
            Self::WrongArchitecture => "Wrong architecture (not AArch64)",
            Self::InvalidProgramHeader => "Invalid program header",
            Self::AllocationFailed => "Memory allocation failed",
            Self::MappingFailed => "Memory mapping failed",
        }
    }
}
```

### 2. Add `Display` impl

```rust
impl core::fmt::Display for ElfError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "E{:04X}: {}", self.code(), self.name())
    }
}
```

### 3. Add `Error` impl

```rust
impl core::error::Error for ElfError {}
```

---

## Verification

1. `cargo build --release`
2. Grep for duplicate 0x02xx codes (should find none)
3. No API changes - callers unchanged

---

## Exit Criteria

- [ ] `code()` method added
- [ ] `name()` method added
- [ ] `Display` impl added
- [ ] `Error` impl added
- [ ] Build passes
- [ ] Code comments include TEAM_XXX
