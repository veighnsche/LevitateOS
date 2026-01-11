# Phase 4: Cleanup

## Dead Code Removal

After migration, scan for orphaned code:

```bash
# Find unused imports
cargo clippy --workspace -- -W unused_imports

# Find dead code
cargo +nightly build --target x86_64-unknown-none -Z dead-code-warning
```

### Expected Removals

| File | Reason |
|------|--------|
| `src/arch/mod.rs` | Replaced by `arch/` crates |
| `src/memory/mod.rs` | Replaced by `mm/` crate |
| `src/task/mod.rs` (old) | Replaced by `sched/` crate |
| `src/fs/vfs/mod.rs` | Replaced by `vfs/` crate |
| `src/syscall/mod.rs` (old) | Replaced by `syscall/` crate |

### Import Cleanup

After each migration step, run:
```bash
cargo fix --allow-dirty --broken-code
cargo fmt --all
```

## Documentation Sync

### Module Documentation

Each new crate needs a `lib.rs` with module-level docs:

```rust
//! # los_mm - Memory Management
//!
//! This crate provides kernel memory management including:
//! - Physical frame allocation (buddy allocator)
//! - Kernel heap
//! - Virtual memory area (VMA) management
//! - User address space handling
//!
//! ## Usage
//!
//! ```rust
//! use los_mm::{PhysFrame, alloc_frame, free_frame};
//! use los_mm::user::{AddressSpace, map_user_page};
//! ```
```

### README Updates

Each crate gets a README.md:

```markdown
# los_mm

Memory management for LevitateOS kernel.

## Crates in this Workspace

| Crate | Description |
|-------|-------------|
| `los_mm` | Physical/virtual memory management |
| `los_sched` | Process/thread scheduling |
| `los_vfs` | Virtual filesystem layer |
| `los_syscall` | Syscall dispatch |

## Building

```bash
cargo build --target x86_64-unknown-none
cargo build --target aarch64-unknown-none
```
```

### Behavior Inventory Update

Update `docs/testing/behavior-inventory.md` with new crate locations:

```markdown
## Memory Management [MM]

| ID | Behavior | Location | Test |
|----|----------|----------|------|
| [MM1] | Physical frame allocation | `los_mm/src/phys.rs` | `tests/mm_phys.rs` |
| [MM2] | Kernel heap allocation | `los_mm/src/heap.rs` | `tests/mm_heap.rs` |
| [MM3] | User page mapping | `los_mm/src/user.rs` | `tests/mm_user.rs` |
| [MM4] | Copy-on-write | `los_mm/src/cow.rs` | `tests/mm_cow.rs` |
```

## Team File Update

Update `.teams/TEAM_422_refactor_kernel_architecture.md` at each phase completion with:

1. **Files Moved**: List of files moved and their new locations
2. **API Changes**: Any public API changes
3. **Breaking Changes**: Changes that affect other teams
4. **Test Updates**: Tests added or modified
5. **Rollback Notes**: How to revert if needed

### Template

```markdown
## Phase N Completion

**Date**: YYYY-MM-DD
**Commit Range**: abc123..def456

### Files Moved
- `src/memory/*.rs` → `mm/src/*.rs`

### API Changes
- `use crate::memory::user::*` → `use los_mm::user::*`

### Call Site Updates
- Updated 47 files
- Full list in commit def456

### Tests
- All existing tests pass
- Added 3 new unit tests for mm crate

### Rollback
```bash
git revert def456..abc123
```
```

## Deprecation Notices

If maintaining backwards compatibility during transition (not recommended per Phase 2 rules, but sometimes necessary):

```rust
// In old location (temporary bridge)
#[deprecated(since = "0.2.0", note = "moved to los_mm::user")]
pub use los_mm::user::validate_user_buffer;
```

**Note**: We prefer clean breaks (Rule: No compatibility shims) but document this pattern in case external dependencies exist.

## Cargo.toml Cleanup

After all migrations, clean up workspace root:

```toml
# Cargo.toml (workspace root)
[workspace]
resolver = "2"
members = [
    "kernel",       # Thin binary
    "arch/aarch64",
    "arch/x86_64",
    "mm",
    "sched",
    "vfs",
    "syscall",
    "drivers/*",
    "hal",
    "utils",
    "error",
    "term",
    "pci",
    "gpu",
]

[workspace.dependencies]
# Shared versions
log = "0.4"
bitflags = "2.4"
spin = "0.9"
linux-raw-sys = "0.9"

[profile.release]
opt-level = "z"
lto = true
panic = "abort"
```

## Feature Flag Audit

After migration, audit feature flags for consistency:

| Feature | Used In | Purpose |
|---------|---------|---------|
| `verbose` | kernel, mm, sched, vfs | Debug logging |
| `diskless` | kernel | Skip initrd |
| `multitask-demo` | sched | Demo tasks |
| `verbose-syscalls` | syscall | Log syscalls |

Ensure feature propagation is correct in Cargo.toml:

```toml
[features]
default = []
verbose = ["los_mm/verbose", "los_sched/verbose", "los_vfs/verbose"]
```

## Lint Configuration

Add workspace-wide lint configuration:

```toml
# Cargo.toml
[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "warn"
missing_docs = "warn"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

Each crate inherits:
```toml
# mm/Cargo.toml
[lints]
workspace = true
```
