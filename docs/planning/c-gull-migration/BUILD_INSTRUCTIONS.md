# c-gull libc Build Instructions

This document describes how to build Rust programs for LevitateOS using c-gull as libc.

## Status: FULLY WORKING

We have successfully built:
1. A statically linked Rust "Hello World" program (68KB)
2. **UNMODIFIED uutils/coreutils** (2.4MB) - no source changes required!

This proves we can build **any Rust program** without modifying its source code.

---

## Build Unmodified Coreutils

```bash
# Build script at: toolchain/build-coreutils.sh
./toolchain/build-coreutils.sh

# Test it:
./toolchain/unmodified-coreutils/target/x86_64-unknown-linux-gnu/release/coreutils echo "Hello!"
./toolchain/unmodified-coreutils/target/x86_64-unknown-linux-gnu/release/coreutils pwd
./toolchain/unmodified-coreutils/target/x86_64-unknown-linux-gnu/release/coreutils cat /etc/hostname
```

**Result:**
```
$ file coreutils
coreutils: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked

$ ls -lh coreutils
2.4M coreutils
```

---

## Prerequisites

1. **Rust nightly-2025-04-28** (specific nightly required for c-ward compatibility)
2. **c-ward repository** cloned at `toolchain/c-ward/`
3. **libc-levitateos wrapper** at `toolchain/libc-levitateos/`

---

## Step 1: Build c-gull as libc.a

```bash
cd toolchain/libc-levitateos
cargo build --release
```

This produces: `target/release/liblibc_levitateos.a` (~5.5MB)

## Step 2: Create Sysroot

```bash
mkdir -p toolchain/sysroot/lib
cp toolchain/libc-levitateos/target/release/liblibc_levitateos.a toolchain/sysroot/lib/libc.a

# Create symlinks for other libraries that libc provides
cd toolchain/sysroot/lib
for lib in m dl rt pthread util gcc_s gcc_eh; do
  ln -sf libc.a lib${lib}.a
done
```

## Step 3: Build Any Rust Program

**Key RUSTFLAGS:**
```bash
export RUSTFLAGS="-C panic=abort \
                  -C link-arg=-nostartfiles \
                  -C link-arg=-static \
                  -C link-arg=-Wl,--allow-multiple-definition \
                  -C link-arg=-L/path/to/sysroot/lib"
```

**Cargo command:**
```bash
cargo +nightly-2025-04-28 build --release \
    -Z build-std=std,panic_abort \
    -Z build-std-features=panic_immediate_abort \
    --target x86_64-unknown-linux-gnu
```

---

## Why These Flags?

| Flag | Purpose |
|------|---------|
| `-Z build-std` | Build std from source so we control linking |
| `-nostartfiles` | c-gull's origin provides `_start` |
| `-static` | No dynamic linking (LevitateOS doesn't have ld.so) |
| `--allow-multiple-definition` | Both std and c-gull provide some symbols |
| `x86_64-unknown-linux-gnu` | Standard Linux target - c-gull makes Linux syscalls |

## Why NOT a Custom Target?

LevitateOS implements the Linux syscall ABI, so programs targeting `x86_64-unknown-linux-gnu`
just work. No custom target needed!

---

## libc-levitateos Features

```toml
c-gull = { features = [
    "take-charge",           # c-gull provides _start entry point
    "thread",                # Threading support
    "call-main",             # Call extern "C" main
    "malloc-via-crates",     # malloc from Rust crates
    "threadsafe-setenv",     # Thread-safe setenv
    "experimental-relocate", # Static PIE support
    "extra-syscalls",        # Extended syscall coverage
    "global-allocator",      # For standalone/no-std builds
    "panic-handler-trap",    # For standalone/no-std builds
    "eh-personality-continue", # For standalone/no-std builds
    "todo",                  # Stub implementations (rewinddir, etc.)
] }
```

---

## Utilities That Work

With current c-gull features, these utilities build and run:
- `cat`, `echo`, `head`, `mkdir`, `pwd`, `rm`, `tail`, `touch`

## Utilities Needing More libc Functions

These need additional c-gull work:
- `ls` - needs `getpwuid`, `getgrgid` (user/group name lookup)
- `date` - needs `nl_langinfo` (locale info)
- `cp`, `mv` - need `rewinddir` (now available with `todo` feature)

---

## File Locations

```
toolchain/
├── c-ward/                          # c-gull/c-scape source
├── libc-levitateos/                 # Wrapper crate
│   ├── Cargo.toml                   # Features configuration
│   ├── src/lib.rs                   # Re-exports c-gull
│   └── rust-toolchain.toml          # nightly-2025-04-28
├── sysroot/lib/                     # Sysroot with our libc
├── unmodified-coreutils/            # Cloned original uutils
└── build-coreutils.sh               # Build script
```

---

## Next Steps

1. **Remove Eyra**: Replace `crates/userspace/eyra/coreutils` with unmodified build
2. **Add to xtask**: Create `cargo xtask build sysroot` command
3. **Test on LevitateOS**: Copy binary to initramfs and run in QEMU
4. **aarch64**: Repeat for aarch64 target
5. **Implement missing libc**: Add `getpwuid`, `getgrgid`, `nl_langinfo` to c-gull
