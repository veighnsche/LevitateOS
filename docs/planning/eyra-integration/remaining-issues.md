# Eyra Integration: Remaining Issues & Decision Points

**TEAM_351** | 2026-01-09

---

## Executive Summary

Eyra builds and runs successfully on Linux. The blocking issue for LevitateOS integration is **static linking** - the current binaries require a dynamic linker which LevitateOS doesn't have.

---

## Current Status

### ✅ What Works

| Component | Status |
|-----------|--------|
| Eyra 0.22 with nightly-2025-04-28 | ✅ Builds |
| x86_64 target | ✅ Builds and runs on Linux |
| aarch64 target | ✅ Builds (cross-compiled) |
| println!, args, time, HashMap | ✅ All work |

### ❌ What Doesn't Work Yet

| Component | Issue |
|-----------|-------|
| Static linking | libgcc_eh not found |
| LevitateOS execution | Requires static binary or ELF loader |

---

## The Core Problem: Dynamic vs Static Linking

### Current Binary Properties

```
$ file eyra-hello
ELF 64-bit LSB pie executable, ARM aarch64, version 1 (SYSV),
dynamically linked, interpreter /lib/ld-linux-aarch64.so.1
```

**Problem:** The binary expects:
1. A dynamic linker (`/lib/ld-linux-aarch64.so.1`)
2. Shared libraries (even though Eyra minimizes these)

**LevitateOS doesn't have:**
- A dynamic linker
- An ELF loader that handles PT_INTERP segments
- The ability to resolve dynamic symbols

---

## Option 1: Static Linking with GCC

### Approach

Add to `.cargo/config.toml`:
```toml
rustflags = [
    "-C", "target-feature=+crt-static",
    "-C", "relocation-model=static"
]
```

### Current Blocker

```
/usr/bin/aarch64-linux-gnu-ld: cannot find -lgcc_eh: No such file or directory
```

### Solution Required

Install static version of libgcc for aarch64:
```bash
sudo apt install gcc-aarch64-linux-gnu-static
# or
sudo apt install libgcc-12-dev-arm64-cross
```

### Pros
- Produces truly static binary
- No runtime dependencies
- Guaranteed to work on LevitateOS

### Cons
- Loses ASLR (Address Space Layout Randomization)
- Larger binary size
- Requires installing additional cross-compilation packages

### Effort: Low-Medium
- Install packages, rebuild
- Estimated time: 30 minutes

---

## Option 2: Eyra's experimental-relocate Feature

### Approach

Enable Eyra's experimental self-relocating code:

```toml
[dependencies]
eyra = { version = "0.22", features = ["experimental-relocate"] }
```

Then build with:
```bash
RUSTFLAGS="-C target-feature=+crt-static" cargo build ...
```

### How It Works

Eyra contains experimental code that performs its own relocations at startup, allowing static PIE (Position Independent Executable) without a dynamic linker.

### Pros
- Keeps ASLR security benefits
- Still fully static
- No additional system packages needed

### Cons
- **Experimental** - may have bugs
- "Involves Rust code patching itself as it runs, which is outside of any Rust semantics" (from Eyra docs)
- Less battle-tested than standard linking

### Effort: Low
- Add feature flag, rebuild
- Estimated time: 15 minutes

---

## Option 3: Implement Dynamic Linker in LevitateOS

### Approach

Add a minimal ELF loader to LevitateOS that:
1. Parses PT_INTERP but ignores it
2. Loads all PT_LOAD segments
3. Performs relocations (R_AARCH64_RELATIVE, etc.)
4. Jumps to entry point

### Pros
- Supports all dynamically linked binaries
- Future-proof for more complex programs
- Industry-standard approach

### Cons
- Significant implementation effort
- Needs to handle multiple relocation types
- Complex debugging if something goes wrong
- Overkill if we only need Eyra binaries

### Effort: High
- Implement ELF loader with relocation support
- Estimated time: 2-5 days

---

## Option 4: Use musl Instead of Eyra

### Approach

Build standard Rust programs with musl target:
```bash
rustup target add aarch64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
```

### Pros
- Produces truly static binaries
- Well-tested, production-ready
- No experimental features
- Works with stable Rust

### Cons
- **Not Eyra** - loses the "pure Rust std" goal
- Still links against musl libc (C code)
- Doesn't test LevitateOS syscall compatibility with Eyra specifically

### Effort: Low
- Install musl target, rebuild
- Estimated time: 15 minutes

---

## Option 5: Defer and Test Syscalls via ulib

### Approach

Instead of Eyra, create a simple test binary using LevitateOS's native `ulib`:

```rust
#![no_std]
#![no_main]
extern crate ulib;

#[no_mangle]
pub extern "C" fn main() -> isize {
    ulib::println!("Testing syscalls...");
    // Test gettid, getuid, clock_gettime, etc.
    0
}
```

### Pros
- No cross-compilation complexity
- Uses existing LevitateOS infrastructure
- Can test all new syscalls immediately
- No external dependencies

### Cons
- Doesn't validate Eyra compatibility
- Doesn't test real std::* APIs
- Less meaningful as a "std support" milestone

### Effort: Very Low
- Write simple test program
- Estimated time: 30 minutes

---

## Recommendation Matrix

| Priority | Option | When to Choose |
|----------|--------|----------------|
| 🥇 | **Option 2 (experimental-relocate)** | You want Eyra working ASAP with minimal effort |
| 🥈 | **Option 1 (static GCC)** | You want proven approach, willing to install packages |
| 🥉 | **Option 4 (musl)** | You want working static binaries, Eyra not critical |
| 4th | **Option 5 (ulib)** | You want to test syscalls now, Eyra can wait |
| 5th | **Option 3 (ELF loader)** | You want long-term dynamic linking support |

---

## My Recommendation

**Try Option 2 first** (experimental-relocate), because:
1. Lowest effort (just add a feature flag)
2. Maintains the Eyra/pure-Rust goal
3. If it works, we're done
4. If it fails, fall back to Option 1

**Fallback to Option 1** if experimental-relocate has issues:
1. Install cross-compilation static libraries
2. Rebuild with standard static linking
3. More work but guaranteed to produce static binary

---

## Quick Commands for Each Option

### Option 1: Static GCC
```bash
# Install static libs (may vary by distro)
sudo apt install gcc-aarch64-linux-gnu

# Update .cargo/config.toml with static flags
# Rebuild
```

### Option 2: experimental-relocate
```bash
cd userspace/eyra-hello
# Edit Cargo.toml: eyra = { version = "0.22", features = ["experimental-relocate"] }
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release \
    --target aarch64-unknown-linux-gnu -Zbuild-std=std,panic_abort
```

### Option 4: musl
```bash
rustup +nightly-2025-04-28 target add aarch64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl
```

---

## Decision Required

Please choose which option you'd like me to pursue:

- [ ] **Option 1** - Static linking with GCC (install packages)
- [ ] **Option 2** - Eyra experimental-relocate feature
- [ ] **Option 3** - Implement ELF loader in LevitateOS
- [ ] **Option 4** - Use musl instead of Eyra
- [ ] **Option 5** - Test via ulib, defer Eyra

---

## Files Reference

| File | Purpose |
|------|---------|
| `userspace/eyra-hello/` | Eyra test crate |
| `userspace/eyra-hello/rust-toolchain.toml` | Pins nightly-2025-04-28 |
| `scripts/build-eyra.sh` | Build script |
| `.teams/TEAM_351_plan_eyra_integration.md` | Team log |
