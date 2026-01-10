# Understanding -nostartfiles in Eyra Workspace

## TL;DR

**All Eyra binaries need `-nostartfiles`** because Eyra's Origin crate provides its own `_start` symbol, which conflicts with the system's `crt1.o`.

This is now configured **once** in `.cargo/config.toml` for the entire workspace. Individual binaries don't need to specify it.

---

## Why Every Eyra Binary Needs This

### The Problem

When you build a C/C++ program, GCC automatically links several "startup files":
- `crt1.o` or `Scrt1.o` - Contains `_start` that calls `__libc_start_main`
- `crti.o` / `crtn.o` - Initialization/finalization code
- `crtbegin.o` / `crtend.o` - GCC's additional startup code

When you use Eyra:
- **Eyra provides its own `_start`** via the Origin crate
- **GCC still tries to link system `crt1.o`** by default
- **Result: Duplicate `_start` symbols** → Linker error

### The Solution

The `-nostartfiles` flag tells GCC:
> "Don't link the default crt*.o startup files"

This allows Origin's `_start` to be the only entry point.

---

## How Eyra Provides _start

```
Your Binary
    ↓
eyra (std replacement)
    ↓
c-gull (libc replacement)
    ↓
c-scape (syscall wrappers)
    ↓ (enables origin-start via take-charge feature)
origin (program startup)
    ↓ (_start symbol - architecture-specific naked function)
program::entry
    ↓
main()
```

Origin's `_start` is a minimal, architecture-specific entry point:

**aarch64:**
```asm
_start:
    mov x0, sp      // Pass stack pointer as argument
    mov x30, xzr    // Clear return address
    b entry         // Jump to Rust entry point
```

**x86_64:**
```asm
_start:
    xor ebp, ebp    // Clear frame pointer  
    mov rdi, rsp    // Pass stack pointer as argument
    call entry      // Call Rust entry point
```

---

## Workspace Configuration

**Location:** `crates/userspace/eyra/.cargo/config.toml`

```toml
[target.x86_64-unknown-linux-gnu]
rustflags = [
    "-C", "target-feature=+crt-static",
    "-C", "relocation-model=pic",
    "-C", "link-arg=-nostartfiles",  # ← Prevents system _start
]

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
rustflags = [
    "-C", "target-feature=+crt-static",
    "-C", "relocation-model=pic",
    "-C", "link-arg=--sysroot=/usr/aarch64-redhat-linux/sys-root/fc43",
    "-C", "target-feature=-outline-atomics",
    "-C", "link-arg=-nostartfiles",  # ← Prevents system _start
]
```

This applies to **all binaries** in the workspace automatically.

---

## For New Binaries

### ✅ DO

- Rely on workspace `.cargo/config.toml` for `-nostartfiles`
- Add `build.rs` only if you need architecture-specific stubs:
  - `libgcc_eh.a` stub (aarch64 cross-compilation)
  - `getauxval()` stub (if using libsyscall or similar)
  - Code generation
  - Custom build logic

### ❌ DON'T

- Add `build.rs` just to specify `-nostartfiles`
- That's now handled at the workspace level

---

## Example: Minimal Eyra Binary

**Cargo.toml:**
```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2021"

[dependencies]
eyra = { version = "0.22", features = ["experimental-relocate"] }
```

**src/main.rs:**
```rust
fn main() {
    println!("Hello from Eyra!");
}
```

**That's it!** No `build.rs` needed for `-nostartfiles`.

---

## When You DO Need build.rs

### Example: Cross-compiling to aarch64

If you're building for aarch64, you need stubs for missing symbols:

**build.rs:**
```rust
fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    
    if target_arch == "aarch64" {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        
        // Create libgcc_eh.a stub
        let lib_path = format!("{}/libgcc_eh.a", out_dir);
        std::process::Command::new("ar")
            .args(["rcs", &lib_path])
            .status()
            .expect("Failed to create libgcc_eh.a stub");
        
        println!("cargo:rustc-link-search=native={}", out_dir);
    }
}
```

This is **in addition to** the workspace-level `-nostartfiles`, not instead of it.

---

## Troubleshooting

### Error: "duplicate symbol: _start"

**Cause:** Somehow `-nostartfiles` is not being applied

**Solutions:**
1. Check that you're building from within the eyra workspace
2. Verify `.cargo/config.toml` exists and has the flag
3. Try `cargo clean` and rebuild

### Error: "cannot find entry symbol _start"

**Cause:** `-nostartfiles` is working, but Origin isn't providing `_start`

**Solutions:**
1. Check that eyra is being used (not just origin directly)
2. Verify `experimental-relocate` feature is enabled
3. Make sure the binary is using eyra as its standard library

### Error: "undefined reference to `getauxval`"

**Cause:** `compiler_builtins` needs `getauxval` on aarch64, but it's not available

**Solution:** Add getauxval stub in build.rs (see libsyscall-tests/build.rs for example)

---

## References

- **TEAM_357** - First investigation of libgcc_eh.a and -nostartfiles
- **TEAM_367** - Applied pattern to all utilities
- **TEAM_380** - Deep dive into aarch64 cross-compilation
- **TEAM_381** - Centralized configuration (this solution)

- **Origin crate:** https://crates.io/crates/origin
- **Eyra crate:** https://crates.io/crates/eyra
- **GCC startup files:** https://gcc.gnu.org/onlinedocs/gccint/Initialization.html
