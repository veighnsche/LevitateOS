# TEAM_381: Centralize -nostartfiles Configuration

**Date:** 2026-01-10  
**Status:** ✅ COMPLETED

---

## Objective

Investigate and resolve the redundant `-nostartfiles` pattern where every Eyra binary needs its own `build.rs` file to specify this flag. Find a better abstraction to eliminate duplication.

---

## Background

Every Eyra-based binary (eyra-hello, eyra-test-runner, libsyscall-tests, coreutils, etc.) was using the same pattern:

```rust
// build.rs
fn main() {
    println!("cargo:rustc-link-arg=-nostartfiles");
    // ... other build logic
}
```

This felt like a missing abstraction since **every** Eyra binary needs this flag.

---

## Investigation

### Question 1: Why is `-nostartfiles` needed?

**Answer:** Duplicate `_start` symbol conflict

When building with Eyra:
1. **Origin provides `_start`**: Eyra → c-gull → c-scape → origin
   - The `take-charge` feature enables `origin-start`
   - Origin defines a naked `_start` function that jumps to `program::entry`
2. **GCC also provides `_start`**: By default, GCC links `crt1.o` (or `Scrt1.o` for PIE)
   - These system startup files contain their own `_start` that calls `__libc_start_main`
3. **Result:** Linker error - duplicate symbol `_start`

**The `-nostartfiles` flag tells GCC:** "Don't link the default crt*.o files (crt1.o, crti.o, crtn.o, crtbegin.o, crtend.o)"

This allows Origin's `_start` to be the only entry point.

### Question 2: What does Origin's _start do?

From `origin/src/arch/aarch64.rs`:

```rust
naked_fn!(
    pub(super) fn _start() -> !;
    "mov x0, sp",   // Pass the incoming sp as arg to entry
    "mov x30, xzr", // Set return address to zero
    "b {entry}";    // Jump to entry
    entry = sym super::program::entry
);
```

Origin's `_start`:
- Is architecture-specific (aarch64, x86_64, etc.)
- Provides a minimal entry point
- Passes control to `program::entry` with initial stack pointer
- Enabled via `origin-start` feature (which is enabled by `take-charge`)

### Question 3: Can we configure this centrally?

**YES!** Cargo's `.cargo/config.toml` supports per-target `rustflags`, which can include link arguments.

---

## Solution

### Before: Individual build.rs files (❌ Duplication)

```
crates/userspace/eyra/
├── eyra-hello/build.rs         → println!("cargo:rustc-link-arg=-nostartfiles");
├── eyra-test-runner/build.rs   → println!("cargo:rustc-link-arg=-nostartfiles");
├── libsyscall-tests/build.rs   → println!("cargo:rustc-link-arg=-nostartfiles");
└── coreutils/build.rs          → println!("cargo:rustc-link-arg=-nostartfiles");
```

### After: Workspace-level config (✅ DRY)

**`.cargo/config.toml`:**
```toml
[target.x86_64-unknown-linux-gnu]
rustflags = [
    "-C", "target-feature=+crt-static",
    "-C", "relocation-model=pic",
    "-C", "link-arg=-nostartfiles",  # <-- ADDED
]

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
rustflags = [
    "-C", "target-feature=+crt-static",
    "-C", "relocation-model=pic",
    "-C", "link-arg=--sysroot=/usr/aarch64-redhat-linux/sys-root/fc43",
    "-C", "target-feature=-outline-atomics",
    "-C", "link-arg=-nostartfiles",  # <-- ADDED
]
```

**build.rs files:**
```rust
// NOTE: -nostartfiles is now configured at workspace level
fn main() {
    // Only aarch64-specific stubs (libgcc_eh.a, getauxval) if needed
}
```

---

## Benefits

### 1. **DRY Principle**
- Single source of truth for `-nostartfiles`
- No need to copy-paste across build.rs files

### 2. **Fail-Fast**
- If a developer forgets `-nostartfiles`, it fails at the workspace level
- Catch errors early in development

### 3. **Easier Maintenance**
- Update once in `.cargo/config.toml` instead of N build.rs files
- Clear documentation in one place

### 4. **Reduced Boilerplate**
- New Eyra binaries don't need build.rs for `-nostartfiles`
- Only need build.rs for actual build logic (stubs, code generation, etc.)

---

## Why This Works

Cargo's build order:
1. Workspace `.cargo/config.toml` is read
2. `rustflags` are applied to all compilations in the workspace
3. `build.rs` runs (can add additional flags via `println!("cargo:rustc-link-arg=...")`)
4. Final link command includes both workspace flags and build.rs flags

Since `-nostartfiles` is **always** needed for Eyra binaries, it belongs in the workspace config, not in individual build scripts.

---

## Verification

Tested with:
- ✅ `eyra-hello` - builds successfully
- ✅ `eyra-test-runner` - builds successfully  
- ✅ `libsyscall-tests` - builds successfully
- ✅ All binaries are statically-linked ELF executables

---

## Files Modified

1. **`.cargo/config.toml`** - Added `-nostartfiles` to both targets with detailed comments
2. **`eyra-hello/build.rs`** - Removed redundant `-nostartfiles` line
3. **`eyra-test-runner/build.rs`** - Removed redundant `-nostartfiles` line
4. **`libsyscall-tests/build.rs`** - Removed redundant `-nostartfiles` line

---

## Key Insights for Future Teams

### When to use build.rs vs .cargo/config.toml

| Concern | Use build.rs | Use .cargo/config.toml |
|---------|-------------|------------------------|
| **Applies to all binaries in workspace** | ❌ | ✅ Use config.toml |
| **Target-specific flags** | ❌ | ✅ Use config.toml |
| **Binary-specific logic** | ✅ | ❌ |
| **Generate code/files** | ✅ | ❌ |
| **Conditional compilation** | ✅ | ❌ |

### Understanding Eyra's _start Symbol

```
User Binary
    ↓
[Eyra crate]
    ↓
[c-gull] ← feature "eyra" enables "take-charge"
    ↓
[c-scape] ← "take-charge" enables "origin-start"
    ↓
[origin] ← provides _start symbol (naked function)
    ↓
[program::entry] ← Eyra's actual entry point
    ↓
[main()] ← Your Rust code
```

**Without `-nostartfiles`:**
```
[System crt1.o _start] ← ❌ Duplicate!
[Origin _start]        ← ❌ Duplicate!
```

**With `-nostartfiles`:**
```
[Origin _start] ← ✅ Only entry point
```

---

## Related TEAM Files

- **TEAM_357** - Discovered libgcc_eh.a issue, first use of build.rs for `-nostartfiles`
- **TEAM_367** - Applied `-nostartfiles` to all utilities, noted the pattern
- **TEAM_380** - Added getauxval stub, questioned the duplicate build.rs pattern
- **TEAM_381** - THIS FILE - Centralized the configuration

---

## Recommendations

### For New Eyra Binaries

1. ✅ **DO**: Rely on workspace `.cargo/config.toml` for `-nostartfiles`
2. ✅ **DO**: Add `build.rs` only if you need:
   - Architecture-specific stubs (libgcc_eh.a, getauxval)
   - Code generation
   - Custom build logic
3. ❌ **DON'T**: Add `build.rs` just for `-nostartfiles`

### For Coreutils Integration

The coreutils submodule has its own workspace. Options:
1. Add similar `.cargo/config.toml` in the coreutils directory
2. Or keep the `build.rs` in `crates/userspace/eyra/coreutils/build.rs` that applies to the submodule

---

## Status: ✅ COMPLETED

All Eyra binaries now build with workspace-level `-nostartfiles` configuration. The redundant pattern has been eliminated, resulting in cleaner, more maintainable code.
