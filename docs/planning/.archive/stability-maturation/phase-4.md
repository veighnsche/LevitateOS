# Phase 4 — Cleanup

**TEAM_311**: ABI Stability Refactor
**Parent**: `docs/planning/stability-maturation/`
**Depends On**: Phase 3 complete
**Status**: Pending
**Last Updated**: 2026-01-08

---

## 1. Dead Code Removal

### 1.1 Files to Delete (Ready Now)
| File | Reason | Status |
|------|--------|--------|
| `userspace/libsyscall/src/sysno.rs` | Replaced by los_abi | Ready after Phase 3 |
| `userspace/libsyscall/src/errno.rs` | Use linux-raw-sys | Ready after Phase 3 |

### 1.2 Files to Delete (DEFERRED - Depends on Phase 2 Deferred Items)

> **⚠️ IMPORTANT**: These files can only be deleted AFTER the deferred library replacements
> from Phase 2 are completed. See `phase-2.md` Section 4 for details.

| File | Reason | Blocker |
|------|--------|---------|
| `kernel/src/loader/elf.rs` | Replaced by `goblin` crate | 🔶 Phase 2 Step 5 deferred |
| `crates/hal/src/x86_64/gdt.rs` | Replaced by `x86_64::structures::gdt` | 🔶 Phase 2 Step 6 deferred |
| `crates/hal/src/x86_64/idt.rs` | Replaced by `x86_64::structures::idt` | 🔶 Phase 2 Step 6 deferred |
| `crates/hal/src/x86_64/multiboot2.rs` | Replaced by `multiboot2` crate | 🔶 Phase 2 Step 7 deferred |

### 1.2 Code to Delete
| Location | Code | Reason |
|----------|------|--------|
| `kernel/src/syscall/mod.rs` | `errno` module | Moved to los_abi |
| `kernel/src/syscall/mod.rs` | `errno_file` module | Duplicate, deleted |
| `kernel/src/arch/aarch64/mod.rs` | `SyscallNumber` enum | Moved to los_abi |
| `kernel/src/arch/x86_64/mod.rs` | `SyscallNumber` enum | Moved to los_abi |
| `kernel/src/syscall/process.rs` | `sys_spawn()` | Custom syscall removed |
| `kernel/src/syscall/process.rs` | `sys_spawn_args()` | Custom syscall removed |

### 1.3 Unused Imports to Remove
After migration, grep for:
- `use crate::arch::SyscallNumber` (should be `use los_abi::...`)
- `SYS_SPAWN`, `SYS_SPAWN_ARGS` (should not exist)

---

## 2. Encapsulation Tightening

### 2.1 los_abi Public API
Only export:
```rust
// crates/abi/src/lib.rs
pub use errno::*;
pub use syscall::SyscallNumber;  // Trait
pub use syscall::aarch64;        // Platform module
pub use syscall::x86_64;         // Platform module
pub use flags::*;
```

### 2.2 Make Internal Fields Private
- `Stat` fields: Keep public (ABI requirement)
- `Termios` fields: Keep public (ABI requirement)
- Helper functions: Make `pub(crate)` where possible

---

## 3. File Size Audit

Target: All files < 500 lines (ideal), < 1000 lines (maximum)

| File | Current | Target | Action |
|------|---------|--------|--------|
| `kernel/src/arch/aarch64/mod.rs` | ~461 | < 300 | Remove SyscallNumber (~140 lines) |
| `kernel/src/arch/x86_64/mod.rs` | ~506 | < 300 | Remove SyscallNumber (~140 lines) |
| `kernel/src/syscall/mod.rs` | ~294 | ~250 | Remove errno modules |

---

## 4. Steps

### Step 1 — Delete Replaced Files
- [ ] Delete `userspace/libsyscall/src/sysno.rs`
- [ ] Delete `userspace/libsyscall/src/errno.rs`
- [ ] Update `userspace/libsyscall/src/lib.rs` imports

### Step 2 — Remove Dead Code from Kernel
- [ ] Remove errno modules from syscall/mod.rs
- [ ] Remove SyscallNumber from arch modules
- [ ] Remove sys_spawn handlers

### Step 3 — Verify No Dangling References
- [ ] `cargo build --workspace`
- [ ] `cargo clippy --workspace`
- [ ] Search for TODO comments referencing removed code

### Step 4 — Final Cleanup
- [ ] Remove any commented-out code
- [ ] Update documentation references
- [ ] Remove TEAM comments for completed work

### Step 5 — Verify Library Replacements
- [ ] Verify `goblin` ELF parsing works for both aarch64 and x86_64
- [ ] Verify `x86_64` crate GDT/IDT works correctly
- [ ] Run golden tests to verify boot still works
- [ ] Delete hand-rolled files listed in 1.1

See `phase-4-step-*.md` files for details.
