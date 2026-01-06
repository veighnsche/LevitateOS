# Phase 2: Design

**Feature:** `define_kernel_error!` Macro  
**Author:** TEAM_153  
**Status:** Ready for Review

---

## Proposed Solution

A declarative macro `define_kernel_error!` that generates:
1. Enum definition with derives
2. `code()` method
3. `name()` method  
4. `Display` impl
5. `Error` impl

### Macro Location

**`levitate-error/src/lib.rs`** - New crate, exported via `levitate_error::define_kernel_error!`

Rationale: Dedicated crate allows all workspace crates to use the same error patterns without depending on levitate-hal.

---

## API Design

### Basic Usage (Simple Errors)

```rust
use levitate_hal::define_kernel_error;

define_kernel_error! {
    /// Network driver error types (0x07xx)
    pub enum NetError(0x07) {
        /// Device not initialized
        NotInitialized = 0x01 => "Network device not initialized",
        /// TX queue is full
        DeviceBusy = 0x02 => "TX queue full",
        /// Transmission failed
        SendFailed = 0x03 => "Transmission failed",
    }
}
```

### Generated Output

```rust
/// Network driver error types (0x07xx)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetError {
    /// Device not initialized
    NotInitialized,
    /// TX queue is full
    DeviceBusy,
    /// Transmission failed
    SendFailed,
}

impl NetError {
    pub const fn code(&self) -> u16 {
        match self {
            Self::NotInitialized => 0x0701,  // (0x07 << 8) | 0x01
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

impl core::fmt::Display for NetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "E{:04X}: {}", self.code(), self.name())
    }
}

impl core::error::Error for NetError {}
```

---

## Behavioral Decisions

### 1. Code Calculation

**Decision:** Automatic from `(subsystem << 8) | variant_code`

```rust
// Input
pub enum NetError(0x07) {
    NotInitialized = 0x01 => "...",
}
// Output code: 0x0701
```

**Rationale:** Eliminates manual calculation errors, ensures subsystem prefix is always correct.

### 2. Derive Traits

**Decision:** Always generate `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`

**Rationale:** All existing error types have these derives. Consistency is more important than flexibility here.

### 3. Visibility

**Decision:** Pass through visibility from macro input

```rust
define_kernel_error! {
    pub enum FooError(0x10) { ... }      // pub
    pub(crate) enum BarError(0x11) { ... } // pub(crate)
}
```

### 4. Doc Comments

**Decision:** Pass through doc comments on enum and variants

```rust
define_kernel_error! {
    /// This comment goes on the enum
    pub enum FooError(0x10) {
        /// This comment goes on the variant
        SomeVariant = 0x01 => "description",
    }
}
```

---

## Open Questions

### Q1: Nested Errors Support?

**Context:** `SpawnError` and `FsError` contain inner errors:
```rust
pub enum SpawnError {
    Elf(ElfError),      // Contains inner error
    PageTable(MmuError),
}
```

**Decision:** Option 2 - Extend macro syntax for nested variants.

**Extended Syntax:**
```rust
define_kernel_error! {
    /// Process spawn errors (0x03xx)
    pub enum SpawnError(0x03) {
        /// ELF loading failed
        Elf(ElfError) = 0x01 => "ELF loading failed",
        /// Page table creation failed
        PageTable(MmuError) = 0x02 => "Page table creation failed",
        /// Stack setup failed
        Stack(MmuError) = 0x03 => "Stack setup failed",
    }
}
```

**Generated Display for nested:**
```rust
impl Display for SpawnError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Elf(inner) => write!(f, "E{:04X}: {} ({})", self.code(), self.name(), inner),
            Self::PageTable(inner) => write!(f, "E{:04X}: {} ({})", self.code(), self.name(), inner),
            Self::Stack(inner) => write!(f, "E{:04X}: {} ({})", self.code(), self.name(), inner),
        }
    }
}
```

---

### Q2: Subsystem Constant Export?

**Context:** Should macro export subsystem constant for validation?
```rust
impl NetError {
    pub const SUBSYSTEM: u8 = 0x07;
}
```

**Decision:** Yes - useful for debugging and validation.

---

### Q3: Duplicate Code Detection?

**Context:** Should macro detect duplicate error codes within same enum?
```rust
define_kernel_error! {
    pub enum BadError(0x10) {
        Foo = 0x01 => "foo",
        Bar = 0x01 => "bar",  // Duplicate!
    }
}
```

**Decision:** No detection - rely on code review. Implementing team will catch duplicates.

---

### Q4: Macro Location - Crate vs Module?

**Context:** Where should macro live?

**Decision:** New `levitate-error` crate.

**Rationale:** Other crates (not just kernel) need the same error patterns. A dedicated crate provides:
- Clean dependency graph
- Reusable across all workspace crates
- Clear ownership of error infrastructure

---

## Design Alternatives Considered

### Alternative 1: Trait-Based

```rust
pub trait KernelError: Display + Error {
    fn code(&self) -> u16;
    fn name(&self) -> &'static str;
}
```

**Rejected because:** Still requires boilerplate for each impl. Doesn't eliminate code duplication.

### Alternative 2: Proc Macro

```rust
#[derive(KernelError)]
#[subsystem(0x07)]
pub enum NetError {
    #[code(0x01, "Network device not initialized")]
    NotInitialized,
}
```

**Rejected because:** Requires proc-macro crate, more complex build. Declarative macro is sufficient.

### Alternative 3: Build Script Generation

Generate error types from a TOML/YAML definition file.

**Rejected because:** Over-engineered for this use case. Macro is simpler and keeps code in Rust files.

---

## Exit Criteria for Phase 2

- [x] Macro API defined
- [x] Generated output specified
- [x] Behavioral decisions documented
- [x] Open questions listed with recommendations
- [x] Alternatives considered and rejected with rationale
