# x86_64 Build Status for libsyscall-tests

**Status:** ❌ Not Currently Supported  
**Priority:** Low (LevitateOS primarily targets aarch64)

---

## Issue

Building `libsyscall-tests` for x86_64 fails with linker errors:

```
undefined symbol: syscall
undefined symbol: __errno_location  
undefined symbol: getcwd
...
```

## Root Cause

The binary is attempting to use Rust's standard library (`std`) instead of Eyra:

```rust
// Evidence from error messages:
library/std/src/sys/env/unix.rs
library/std/src/sys/sync/rwlock/futex.rs
library/std/src/sys/sync/mutex/futex.rs
```

This happens because:
1. `libsyscall` is a `#![no_std]` library
2. `libsyscall-tests` depends on both `eyra` and `libsyscall`
3. On aarch64 with sysroot, Eyra properly replaces std
4. On x86_64 (native host), Rust's std is being used instead of Eyra

## Why aarch64 Works

The aarch64 build succeeds because:
- Cross-compilation environment with explicit sysroot
- Build isolation prevents accidental std usage
- All stubs (libgcc_eh.a, getauxval) are properly provided

## Potential Solutions (Not Implemented)

### Option 1: Use `-Zbuild-std` for x86_64

Build Rust's std from source with Eyra:
```bash
cargo build -p libsyscall-tests \
    --target x86_64-unknown-linux-gnu \
    -Zbuild-std=std,panic_abort \
    --release
```

**Cons:** 
- Slow builds
- Complex toolchain requirements
- Not needed for LevitateOS (aarch64-only)

### Option 2: Convert to Pure no_std Tests

Remove std dependency entirely:
- No `std::panic::catch_unwind`
- No `String` or `Vec`
- No colored output

**Cons:**
- Defeats the purpose of "std support via Eyra" testing
- Less useful tests

### Option 3: Accept aarch64-Only

**Pros:**
- LevitateOS is aarch64-first
- Tests are meant to run on LevitateOS, not on host
- Simpler toolchain requirements
- Already working for the actual target

**Selected:** This is the current approach

---

## Recommendation

**Accept aarch64-only for now.** Reasons:

1. **LevitateOS is aarch64-first** - x86_64 support is secondary
2. **Tests target LevitateOS** - meant to run on the actual OS, not on host
3. **aarch64 build works perfectly** - 65KB statically-linked binary
4. **Low ROI** - fixing x86_64 requires significant effort for minimal benefit
5. **Workaround exists** - if needed, run tests in QEMU aarch64 environment

---

## For Future Teams

If x86_64 support becomes important:

1. **Investigate `-Zbuild-std`** - May allow proper Eyra integration
2. **Check Eyra upstream** - May have x86_64-specific configuration
3. **Consider separate test binary** - x86_64-specific tests without libsyscall dependency

Until then, focus on aarch64 which is the primary LevitateOS target.
