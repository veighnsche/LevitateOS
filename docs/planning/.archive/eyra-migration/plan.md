# Eyra Migration Plan

**TEAM_361** | Migration from ulib to Eyra/std  
**Created:** 2026-01-09

---

## Executive Summary

Migrate LevitateOS userspace from the handrolled `ulib` (no_std) to Eyra/std for:
- Standard Rust idioms
- Better error handling
- Less custom code to maintain
- Access to crates.io ecosystem

---

## Current Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Userspace Apps                     │
│  init, shell, cat, cp, ls, mkdir, rm, touch, etc.   │
├─────────────────────────────────────────────────────┤
│                      ulib                            │
│  (handrolled std: alloc, fs, io, env, time)         │
├─────────────────────────────────────────────────────┤
│                   libsyscall                         │
│  (raw syscall wrappers)                              │
├─────────────────────────────────────────────────────┤
│                     Kernel                           │
└─────────────────────────────────────────────────────┘
```

## Target Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Userspace Apps                     │
│  init, shell, cat, cp, ls, mkdir, rm, touch, etc.   │
├─────────────────────────────────────────────────────┤
│                   Eyra (std)                         │
│  (full Rust standard library via Linux syscalls)    │
├─────────────────────────────────────────────────────┤
│                     Kernel                           │
└─────────────────────────────────────────────────────┘

Note: libsyscall kept for low-level/custom syscalls only
```

---

## Migration Phases

### Phase 1: Proof of Concept (1-2 hours)
- Migrate ONE utility to Eyra/std
- **Candidate:** `cat` (simple, well-understood)
- Verify it works in LevitateOS
- Measure binary size difference

### Phase 2: Core Utilities (4-6 hours)
- Migrate levbox utilities one by one:
  - cat, cp, ln, ls, mkdir, mv, pwd, rm, rmdir, touch
- Each utility becomes standalone std binary
- Test each after migration

### Phase 3: Shell Migration (2-3 hours)
- Rewrite shell with std
- Benefits: proper String handling, better command parsing
- Add features: command history, tab completion (optional)

### Phase 4: Init Migration (1 hour)
- Simple migration
- init is minimal, mostly spawning shell

### Phase 5: Cleanup (1-2 hours)
- Remove ulib (or archive it)
- Update build system
- Update documentation

---

## Per-Utility Migration Pattern

### Before (ulib/no_std)
```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libsyscall::{println, common_panic_handler};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    common_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Manual argument parsing
    // Raw syscalls
    // Custom error handling
    libsyscall::exit(0);
}
```

### After (Eyra/std)
```rust
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Standard Rust idioms
    // Real error handling with ?
    // Standard library types
    
    Ok(())
}
```

---

## Build System Changes

### New Cargo.toml Pattern
```toml
[package]
name = "cat"
version = "0.1.0"
edition = "2021"

# TEAM_363: Exclude from parent workspace - uses different toolchain
[workspace]

# Eyra provides std via pure-Rust Linux syscalls
[dependencies]
eyra = { version = "0.22", features = ["experimental-relocate"] }

[profile.release]
panic = "abort"
opt-level = "s"  # "s" for size, balanced
lto = true
strip = true     # Remove symbols for smaller binaries

# For minimal binary size
[unstable]
build-std-features = ["panic_immediate_abort"]
```

**Important:** Each app's `main.rs` must include:
```rust
extern crate eyra;  // Required for -Zbuild-std compatibility
```

### Build Command
```bash
cargo build --release \
    --target x86_64-unknown-linux-gnu \
    -Zbuild-std=std,panic_abort
```

---

## Binary Size Comparison

| Utility | ulib (no_std) | Eyra (std) | Delta |
|---------|---------------|------------|-------|
| cat | ~25 KB | ~300 KB | +275 KB |
| shell | ~50 KB | ~350 KB | +300 KB |

**Note:** Size increase is acceptable for the benefits gained.
Total disk usage for all utilities: ~3 MB (acceptable for modern systems).

---

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Missing syscalls | Eyra test already passes; add syscalls as needed |
| Binary size | Acceptable tradeoff; use LTO and opt-level=z |
| Build complexity | Document build process clearly |
| Regression | Test each utility after migration |

---

## Success Criteria

1. All utilities work identically to ulib versions
2. Eyra test (`cargo xtask test eyra`) passes
3. Shell is fully functional
4. Build system is documented
5. ulib can be removed without breaking anything

---

## Open Questions

### Q1: Parallel Installation?
Keep both ulib and Eyra versions during migration, or replace in-place?

**Recommendation:** Replace in-place, one utility at a time.

### Q2: New Directory Structure?
Put Eyra apps in new location or replace existing?

**Recommendation:** Replace existing in `crates/userspace/`.

### Q3: initramfs Size Limit?
Is 3-5 MB initramfs acceptable?

**Recommendation:** Yes, modern systems have plenty of RAM.

---

## Timeline Estimate

| Phase | Effort | Cumulative |
|-------|--------|------------|
| Phase 1: PoC | 2 hours | 2 hours |
| Phase 2: Utilities | 6 hours | 8 hours |
| Phase 3: Shell | 3 hours | 11 hours |
| Phase 4: Init | 1 hour | 12 hours |
| Phase 5: Cleanup | 2 hours | 14 hours |

**Total: ~14 hours of work**
