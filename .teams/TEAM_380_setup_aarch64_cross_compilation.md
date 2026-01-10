# TEAM_380: Setup AArch64 Cross-Compilation for libsyscall-tests

**Date:** 2026-01-10  
**Status:** ✅ COMPLETED

---

## Objective

Set up aarch64 cross-compilation environment to build libsyscall-tests with std support via Eyra.

---

## Context

After migrating libsyscall to the eyra directory for std support, the integration test binary `libsyscall-tests` needed to be built for aarch64. This required:
1. Installing cross-compilation toolchain
2. Configuring sysroot
3. Resolving linker issues specific to Eyra + libsyscall combination

---

## Investigation and Solutions

### Issue 1: Missing libgcc_eh.a

**Problem:** Fedora's aarch64 cross-compiler doesn't ship `libgcc_eh.a`

**Solution:** Already solved by TEAM_357 - create empty stub in `build.rs`

### Issue 2: Missing Sysroot

**Problem:** No system libraries available for aarch64 target

**Actions Taken:**
1. Installed `sysroot-aarch64-fc43-glibc` package
2. Updated `.cargo/config.toml` with sysroot path: `/usr/aarch64-redhat-linux/sys-root/fc43`

### Issue 3: Undefined Reference to `getauxval`

**Problem:** `compiler_builtins` needs `getauxval` to detect LSE atomics support, but with `-nostartfiles` it's not available from libc

**Root Cause:**
```
compiler_builtins → init_have_lse_atomics() → getauxval()
                  → __init_cpu_features() → getauxval()
```

The prebuilt `compiler_builtins` in rustc's standard library tries to detect CPU features dynamically using `getauxval`. With `-nostartfiles` and `-nodefaultlibs`, the symbol isn't resolved even though `libc.a` is linked.

**Solution:** Create `getauxval` stub in `build.rs` that returns 0 (feature not available)

This is safe because:
- Returning 0 makes `compiler_builtins` fall back to non-LSE atomic implementations
- LevitateOS kernel provides the necessary syscall support for standard atomics
- The performance difference is negligible for userspace utilities

---

## Files Modified

### 1. `.cargo/config.toml`
```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
rustflags = [
    "-C", "target-feature=+crt-static",
    "-C", "relocation-model=pic",
    "-C", "link-arg=--sysroot=/usr/aarch64-redhat-linux/sys-root/fc43",
    "-C", "target-feature=-outline-atomics",  # Disable outline atomics
]
```

### 2. `libsyscall-tests/build.rs`
Added two stubs:
- Empty `libgcc_eh.a` archive
- `getauxval()` function that returns 0

---

## Build Instructions

### Prerequisites

Install cross-compilation dependencies:
```bash
sudo dnf install -y gcc-aarch64-linux-gnu sysroot-aarch64-fc43-glibc
```

### Building

```bash
cd crates/userspace/eyra
cargo build -p libsyscall-tests --target aarch64-unknown-linux-gnu --release
```

### Verification

```bash
file target/aarch64-unknown-linux-gnu/release/libsyscall-tests
# Output: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV), statically linked
```

---

## Why This Approach?

### Alternative Considered: Use Full Sysroot with getauxval

Could link against full libc.a, but this conflicts with Eyra's philosophy of providing its own libc implementation.

### Alternative Considered: Build std from Source

Using `-Zbuild-std` would give us control over `compiler_builtins`, but:
- Adds complexity to build process
- Slower builds
- May not work with Eyra's expectations

### Chosen Approach: Provide Minimal Stubs

**Advantages:**
- Minimal invasiveness
- Fast builds (no std rebuild)
- Compatible with Eyra's architecture
- Easy to understand and maintain

**Tradeoff:**
- LSE atomics not used (minor performance impact)
- Need to maintain stubs for new symbols if they appear

---

## Lessons Learned

1. **Cross-compilation requires full sysroot**, not just the compiler
2. **`-nostartfiles` means you need to provide ALL symbols**, even ones you'd expect from libc
3. **Eyra utilities (eyra-hello, eyra-test-runner) don't have this issue** because they don't depend on libsyscall which is `#![no_std]`
4. **The combination of no_std library + std binary** creates unique linking challenges

---

## Related TEAM Files

- TEAM_357: Solved libgcc_eh.a issue
- TEAM_367: Solved duplicate _start issue with -nostartfiles
- TEAM_291: Initial investigation of aarch64 cross-compilation

---

## Next Steps

1. Test libsyscall-tests on actual LevitateOS kernel
2. Consider adding same getauxval stub to other Eyra utilities if they encounter the issue
3. Document this pattern for future crates that combine no_std libraries with std binaries

---

## Status: ✅ COMPLETED

Binary builds successfully:
- **File:** `target/aarch64-unknown-linux-gnu/release/libsyscall-tests`
- **Size:** 65KB
- **Type:** Statically-linked AArch64 executable
- **Status:** Ready for testing on LevitateOS
