# UoW 6: Create FsError Type

**Phase:** 4 - Implementation  
**Parent:** `phase-4.md`  
**Dependencies:** None  
**Estimated Lines:** ~50

---

## Objective

Replace `&'static str` errors in filesystem modules with typed `FsError` enum.

---

## Target Files

- `kernel/src/fs/mod.rs`
- `kernel/src/fs/fat.rs`
- `kernel/src/fs/ext4.rs`

---

## Current String Errors

```rust
// fs/mod.rs
"Failed to mount FAT32: {}"

// fs/fat.rs
"Failed to open FAT volume"
"Failed to open root dir"

// fs/ext4.rs
"Failed to load ext4 filesystem"
```

---

## Changes Required

### 1. Create FsError in fs/mod.rs

```rust
/// TEAM_XXX: Filesystem error type with error codes (0x05xx)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsError {
    /// Failed to open volume (0x0501)
    VolumeOpen,
    /// Failed to open directory (0x0502)
    DirOpen,
    /// Failed to open file (0x0503)
    FileOpen,
    /// Read error (0x0504)
    ReadError,
    /// Write error (0x0505)
    WriteError,
    /// Filesystem not mounted (0x0506)
    NotMounted,
    /// Block device error (0x0507)
    BlockError(crate::block::BlockError),
}

impl FsError {
    pub const fn code(&self) -> u16 {
        match self {
            Self::VolumeOpen => 0x0501,
            Self::DirOpen => 0x0502,
            Self::FileOpen => 0x0503,
            Self::ReadError => 0x0504,
            Self::WriteError => 0x0505,
            Self::NotMounted => 0x0506,
            Self::BlockError(_) => 0x0507,
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::VolumeOpen => "Failed to open volume",
            Self::DirOpen => "Failed to open directory",
            Self::FileOpen => "Failed to open file",
            Self::ReadError => "Read error",
            Self::WriteError => "Write error",
            Self::NotMounted => "Filesystem not mounted",
            Self::BlockError(_) => "Block device error",
        }
    }
}

impl core::fmt::Display for FsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BlockError(inner) => write!(f, "E{:04X}: {} ({})", self.code(), self.name(), inner),
            _ => write!(f, "E{:04X}: {}", self.code(), self.name()),
        }
    }
}

impl core::error::Error for FsError {}

impl From<crate::block::BlockError> for FsError {
    fn from(e: crate::block::BlockError) -> Self {
        FsError::BlockError(e)
    }
}
```

### 2. Update fs/mod.rs functions

```rust
pub fn init() -> Result<(), FsError>
pub fn init_ext4() -> Result<(), FsError>
pub fn list_dir(...) -> Result<Vec<String>, FsError>
```

### 3. Update fs/fat.rs

```rust
pub fn mount_and_list() -> Result<Vec<String>, FsError> {
    // ...
    .map_err(|_| FsError::VolumeOpen)?;
    .map_err(|_| FsError::DirOpen)?;
}
```

### 4. Update fs/ext4.rs

```rust
pub fn mount_and_list() -> Result<Vec<String>, FsError> {
    // ...
    .map_err(|_| FsError::VolumeOpen)?;
}
```

---

## Verification

1. `cargo build --release`
2. Check callers in `kernel/src/init.rs`

---

## Exit Criteria

- [ ] `FsError` enum created with 7 variants
- [ ] All FS functions return `Result<T, FsError>`
- [ ] All string errors replaced
- [ ] Build passes
