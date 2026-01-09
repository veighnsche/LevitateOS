# Phase 3: Implementation — Build Eyra Test Binary

**TEAM_351** | Eyra Integration Plan  
**Created:** 2026-01-09  
**Depends on:** Phase 2 (Prerequisites complete)

---

## 1. Objective

Build a minimal Rust binary using Eyra that can run on LevitateOS.

---

## 2. Approach

Eyra replaces the standard `std` crate with a pure-Rust implementation that makes Linux syscalls directly. Since LevitateOS implements Linux-compatible syscalls, Eyra binaries should work.

### Strategy

1. Create a new crate in `userspace/eyra-hello/`
2. Configure it to use Eyra as `std`
3. Cross-compile for aarch64/x86_64
4. Add to initramfs
5. Boot and test

---

## 3. Steps

### Step 1: Create Eyra Test Crate

**File:** `userspace/eyra-hello/Cargo.toml`

```toml
[package]
name = "eyra-hello"
version = "0.1.0"
edition = "2021"

[dependencies]
eyra = { version = "0.17", rename = "std" }

# Static linking (no dynamic libs on LevitateOS)
[profile.release]
lto = true
panic = "abort"
```

**File:** `userspace/eyra-hello/build.rs`

```rust
fn main() {
    println!("cargo:rustc-link-arg=-nostartfiles");
}
```

**File:** `userspace/eyra-hello/src/main.rs`

```rust
fn main() {
    println!("Hello from Eyra on LevitateOS!");
    
    // Test basic std functionality
    let args: Vec<String> = std::env::args().collect();
    println!("argc = {}", args.len());
    
    // Test threading (optional, may fail initially)
    // let handle = std::thread::spawn(|| {
    //     println!("Hello from thread!");
    // });
    // handle.join().unwrap();
    
    println!("Eyra test complete!");
}
```

### Step 2: Create Build Script

**File:** `scripts/build-eyra-hello.sh`

```bash
#!/bin/bash
set -e

ARCH=${1:-aarch64}
TARGET="${ARCH}-unknown-linux-gnu"

cd userspace/eyra-hello

# Use nightly for -Zbuild-std
cargo +nightly build \
    --release \
    --target $TARGET \
    -Zbuild-std=std,panic_abort \
    -Zbuild-std-features=panic_immediate_abort

# Copy to initramfs staging
cp target/$TARGET/release/eyra-hello ../../initramfs/
```

### Step 3: Add to Initramfs

Modify `scripts/make_initramfs.sh` to include `eyra-hello` if present.

### Step 4: Test Manually

```bash
# Build kernel + initramfs
cargo xtask build --arch aarch64

# Run with VNC
cargo xtask run-vnc --arch aarch64

# In shell, run:
# /eyra-hello
```

---

## 4. UoWs (Units of Work)

### UoW 3.1: Create eyra-hello crate structure

**Tasks:**
1. Create `userspace/eyra-hello/` directory
2. Create `Cargo.toml` with Eyra dependency
3. Create `build.rs` with `-nostartfiles`
4. Create `src/main.rs` with hello world

**Exit Criteria:**
- `cargo +nightly build` succeeds locally (with Linux target)

### UoW 3.2: Create cross-compilation setup

**Tasks:**
1. Create build script for aarch64/x86_64
2. Verify static linking works
3. Check binary size is reasonable

**Exit Criteria:**
- Static binary for aarch64 exists
- `file` command shows statically linked ELF

### UoW 3.3: Integrate with initramfs build

**Tasks:**
1. Add eyra-hello to initramfs
2. Verify it's included in CPIO archive

**Exit Criteria:**
- `cargo xtask build` includes eyra-hello in initramfs

---

## 5. Potential Issues

### Issue: Missing syscalls

**Symptom:** Binary crashes with "Unknown syscall" log  
**Solution:** Check log, implement missing syscall

### Issue: TLS initialization failure

**Symptom:** Crash early in startup  
**Solution:** Verify `arch_prctl` (x86_64) or TPIDR_EL0 (aarch64) works

### Issue: mmap failure

**Symptom:** Allocator can't get memory  
**Solution:** Debug mmap syscall, check VMA tracking

### Issue: Nightly Rust required

**Symptom:** Build fails without `-Zbuild-std`  
**Solution:** Use `cargo +nightly` or update rust-toolchain.toml

---

## 6. Success Criteria

- [ ] `eyra-hello` binary builds for aarch64
- [ ] Binary is statically linked
- [ ] Binary is included in initramfs
- [ ] Ready for Phase 4 testing

---

## 7. Next Phase

**Phase 4:** Run the binary on LevitateOS and debug any issues.
