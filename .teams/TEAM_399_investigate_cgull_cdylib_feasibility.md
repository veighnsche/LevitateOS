# TEAM_399: Investigate c-gull cdylib Feasibility

**Date**: 2026-01-10
**Status**: Investigation Complete
**Blocker For**: TEAM_397 General Purpose OS Feature Plan (Phase B)

---

## Executive Summary

**Can c-gull be built as libc.so.6?**

| Question | Answer |
|----------|--------|
| Does c-gull currently support cdylib? | **NO** |
| Are the technical foundations present? | **YES** (partial) |
| Is this a quick fix? | **NO** - significant work required |
| Is there an alternative path? | **YES** - multiple options |

**Verdict**: Building c-gull as libc.so.6 is **theoretically possible** but **not currently implemented** and would require **substantial upstream contribution** or forking.

---

## 1. Current State Analysis

### c-gull/c-scape Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   c-gull (libc facade)                  │
│  - Re-exports c-scape::*                                │
│  - Adds: nss, resolve, sysconf, system, termios, time   │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│                   c-scape (libc core)                   │
│  - 53+ source files implementing POSIX/libc functions   │
│  - All functions use #[no_mangle] unsafe extern "C"     │
│  - Delegates to rustix for actual syscalls              │
└─────────────────────────┬───────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────┐
│                   rustix (syscall layer)                │
│  - Direct Linux syscalls, no libc dependency            │
│  - Safe Rust wrappers around raw syscalls               │
└─────────────────────────────────────────────────────────┘
```

### Build Configuration (Current)

```toml
# c-gull/Cargo.toml - NO crate-type specified
[lib]
# Defaults to rlib (Rust library)

# c-scape/Cargo.toml - NO crate-type specified
[lib]
# Defaults to rlib (Rust library)
```

### Symbol Export Pattern

c-scape functions follow the correct pattern for C ABI export:

```rust
// Example from c-scape/src/fs/mod.rs
#[no_mangle]
unsafe extern "C" fn chown(
    pathname: *const c_char,
    owner: uid_t,
    group: gid_t,
) -> c_int {
    libc!(libc::chown(pathname, owner, group));  // Compile-time signature check only
    // ... actual implementation using rustix
}
```

The `libc!()` macro is **NOT a runtime fallback** - it's a compile-time signature verification that never executes.

---

## 2. What's Required for cdylib

### Technical Requirements

| Requirement | c-gull Status | Notes |
|-------------|---------------|-------|
| `crate-type = ["cdylib"]` | ❌ Missing | Must add to Cargo.toml |
| `#[no_mangle]` exports | ✅ Present | Functions correctly annotated |
| No std dependency | ⚠️ Partial | Has `std` feature, needs `no_std` mode |
| No circular libc dependency | ✅ OK | Uses rustix, not libc |
| Startup code (crt0, crti) | ❌ Missing | Needs origin crate integration |
| Symbol versioning | ❌ Missing | glibc compatibility requires versioned symbols |
| Thread-local storage | ⚠️ Partial | TLS model must match platform expectations |

### cdylib Build Implications

From [Rust Reference on Linkage](https://doc.rust-lang.org/reference/linkage.html):
- cdylib links all Rust dependencies statically
- Only `#[no_mangle]` extern functions are exported
- No Rust metadata included (unlike dylib)

### What cdylib Enables

```bash
# With crate-type = ["cdylib"], cargo would produce:
target/release/libc_gull.so

# This could theoretically be symlinked:
ln -s libc_gull.so /lib/libc.so.6
```

---

## 3. Known Blockers

### Blocker 1: No Upstream Support

**Evidence**: Searched c-ward GitHub issues for "dynamic", "shared", "cdylib", ".so" - **zero relevant results**.

**Mustang README explicitly states**:
> "Dynamic linking isn't implemented yet."

This is a deliberate design choice, not an oversight.

### Blocker 2: Missing Startup Code

To be a full libc replacement, c-gull needs:
- `_start` entry point (crt0)
- `_init`/`_fini` constructors (crti/crtn)
- `.init_array`/`.fini_array` support

The `origin` crate provides some of this for static binaries but not for shared library mode.

### Blocker 3: Symbol Versioning

glibc uses symbol versioning (e.g., `memcpy@GLIBC_2.14`). Without this:
- Programs compiled against glibc may fail to link
- No binary compatibility with existing Linux programs

### Blocker 4: Missing libc Functions

From Mustang README:
> "Many libc C functions that aren't typically needed by most Rust programs aren't implemented yet."

This includes potentially critical functions for general-purpose compatibility.

### Blocker 5: Testing Infrastructure

No existing test suite validates c-gull against the full POSIX/libc ABI. Building it as libc.so without comprehensive testing is risky.

---

## 4. Comparison with Alternatives

### Option A: Contribute cdylib Support to c-gull

| Aspect | Assessment |
|--------|------------|
| Effort | Very High (months) |
| Upstream acceptance | Uncertain |
| Maintenance burden | Shared with upstream |
| Compatibility | Best long-term |

**Required work**:
1. Add `crate-type = ["cdylib", "staticlib"]` to Cargo.toml
2. Ensure all exports work in shared library context
3. Implement symbol versioning
4. Create test suite for dynamic loading
5. Handle startup code for shared library mode

### Option B: Port musl-libc

| Aspect | Assessment |
|--------|------------|
| Effort | Medium |
| Upstream acceptance | N/A (C codebase) |
| Maintenance burden | On us |
| Compatibility | Excellent (proven) |

**Advantages**:
- Production-ready
- Well-tested
- Supports both static and dynamic linking
- Used by Alpine Linux, Void Linux

**Disadvantages**:
- Written in C (not Rust)
- Requires cross-compilation setup

### Option C: Use relibc (Redox)

| Aspect | Assessment |
|--------|------------|
| Effort | Medium-High |
| Compatibility | Good for Redox, unknown for generic Linux |

**Status**: relibc currently builds as `staticlib` only, but has an `ld_so` component suggesting dynamic linking is in progress.

### Option D: Static-Only First (Recommended)

| Aspect | Assessment |
|--------|------------|
| Effort | Low |
| Compatibility | Limited but immediate |

**Strategy**:
1. Build c-gull as `staticlib` (libc.a)
2. Compile programs with `-static`
3. Defer dynamic linking to future milestone

This achieves "General Purpose for Static Binaries" quickly.

---

## 5. Experimental: Quick cdylib Test

To test if c-gull CAN be built as cdylib (even if incomplete):

```toml
# Fork c-gull, modify Cargo.toml:
[lib]
crate-type = ["cdylib", "rlib"]

[features]
default = ["take-charge", "malloc-via-crates", "thread"]
```

```bash
# Attempt build:
cargo build --release --features "take-charge"

# Check symbols:
nm -D target/release/libc_gull.so | grep " T " | head -20
```

**Expected result**: Build may succeed but symbols may be incomplete or missing versioning.

---

## 6. Recommendations

### Immediate Path (for TEAM_397 Phase B)

1. **Revise Phase B** to focus on **static linking first**:
   ```
   Phase B.1: Build c-gull as libc.a (staticlib)
   Phase B.2: Test static binary compilation
   Phase B.3: Defer libc.so to Phase B+ (future)
   ```

2. **Add new investigation task**:
   ```
   TEAM_400: Experimental c-gull cdylib build
   - Fork c-gull
   - Add crate-type = ["cdylib"]
   - Document what breaks
   - Identify minimum viable fix set
   ```

### Medium-Term Path

3. **Evaluate musl-libc as parallel track**:
   - Set up cross-compilation for musl
   - Test musl-compiled binaries on LevitateOS
   - This provides a fallback if c-gull cdylib proves too difficult

4. **Engage upstream**:
   - Open issue on c-ward GitHub discussing dynamic linking use case
   - Gauge interest before investing significant effort

### Long-Term Path

5. **Choose ONE path and commit**:
   - If c-gull cdylib is feasible → invest in upstream contribution
   - If not feasible → port musl-libc
   - Don't maintain two parallel libc implementations

---

## 7. Decision Matrix

| Factor | c-gull cdylib | musl-libc | Static-Only |
|--------|---------------|-----------|-------------|
| Time to first result | 2-3 months | 1-2 months | 1-2 weeks |
| Rust-native | ✅ Yes | ❌ No | ✅ Yes |
| Production-ready | ❌ Experimental | ✅ Yes | ⚠️ Limited |
| Dynamic linking | ✅ Goal | ✅ Yes | ❌ No |
| Maintenance | Medium | High | Low |
| Community support | Low | High | N/A |

**Recommendation**: **Static-Only first**, then evaluate musl-libc vs c-gull cdylib based on experimental results.

---

## 8. Conclusion

Building c-gull as libc.so.6 is **not a quick win**. The architecture supports it in theory (correct symbol exports), but:

1. **No upstream support** - this would be novel work
2. **Missing critical pieces** - startup code, symbol versioning
3. **Untested path** - no one has done this before

For LevitateOS's goal of "run any Unix program without modification", the pragmatic path is:

1. **Now**: Static binary support (c-gull as libc.a)
2. **Next**: Experimental cdylib investigation
3. **Future**: Either fix c-gull cdylib OR port musl-libc

---

## References

- [c-ward GitHub](https://github.com/sunfishcode/c-ward)
- [Mustang README](https://github.com/sunfishcode/mustang) - "Dynamic linking isn't implemented yet"
- [Rust Linkage Reference](https://doc.rust-lang.org/reference/linkage.html)
- [cdylib RFC 1510](https://rust-lang.github.io/rfcs/1510-cdylib.html)
- [relibc (Redox)](https://gitlab.redox-os.org/redox-os/relibc)
- [musl-libc](https://musl.libc.org/)
