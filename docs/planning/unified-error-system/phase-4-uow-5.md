# UoW 5: Preserve Inner Errors in SpawnError

**Phase:** 4 - Implementation  
**Parent:** `phase-4.md`  
**Dependencies:** UoW 1 (ElfError codes), UoW 3 (MmuError)  
**Estimated Lines:** ~30

---

## Objective

Update `SpawnError` to preserve inner error context instead of discarding it.

---

## Target File

`kernel/src/task/process.rs`

---

## Current State

```rust
#[derive(Debug)]
pub enum SpawnError {
    ElfError,           // LOSES inner ElfError!
    PageTableCreation,
    StackSetup,
}

impl From<ElfError> for SpawnError {
    fn from(_e: ElfError) -> Self {
        SpawnError::ElfError  // Context lost!
    }
}
```

---

## Changes Required

### 1. Update SpawnError to hold inner errors

```rust
/// TEAM_XXX: Process spawn error with preserved context (0x03xx)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnError {
    /// ELF parsing/loading failed (0x0301)
    Elf(ElfError),
    /// Page table creation failed (0x0302)
    PageTable(MmuError),
    /// Stack setup failed (0x0303)  
    Stack(MmuError),
}
```

### 2. Add error code methods

```rust
impl SpawnError {
    pub const fn code(&self) -> u16 {
        match self {
            Self::Elf(_) => 0x0301,
            Self::PageTable(_) => 0x0302,
            Self::Stack(_) => 0x0303,
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Elf(_) => "ELF loading failed",
            Self::PageTable(_) => "Page table creation failed",
            Self::Stack(_) => "Stack setup failed",
        }
    }
}
```

### 3. Add Display with inner error

```rust
impl core::fmt::Display for SpawnError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Elf(inner) => write!(f, "E{:04X}: {} ({})", self.code(), self.name(), inner),
            Self::PageTable(inner) => write!(f, "E{:04X}: {} ({})", self.code(), self.name(), inner),
            Self::Stack(inner) => write!(f, "E{:04X}: {} ({})", self.code(), self.name(), inner),
        }
    }
}
```

### 4. Update From impl

```rust
impl From<ElfError> for SpawnError {
    fn from(e: ElfError) -> Self {
        SpawnError::Elf(e)  // Preserves context!
    }
}
```

### 5. Update spawn_from_elf function

```rust
// Change:
.ok_or(SpawnError::PageTableCreation)?;
// To:
.ok_or(SpawnError::PageTable(MmuError::AllocationFailed))?;

// Change:
.map_err(|_| SpawnError::StackSetup)?
// To:
.map_err(|e| SpawnError::Stack(e))?
```

---

## Verification

1. `cargo build --release`
2. Error output now shows inner error: `E0301: ELF loading failed (E0202: Invalid ELF magic)`

---

## Exit Criteria

- [ ] SpawnError variants hold inner errors
- [ ] `code()` and `name()` methods added
- [ ] `Display` shows nested error
- [ ] `From<ElfError>` preserves context
- [ ] Build passes
