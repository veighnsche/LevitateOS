# xtask Module Analysis

Complete analysis of all xtask modules for the refactor.

## Summary

| Category | Files | LOC | Action |
|----------|-------|-----|--------|
| Core Builder | 9 | ~1,500 | KEEP |
| QEMU/Runner | 6 | ~700 | KEEP |
| Support/Utils | 6 | ~400 | KEEP |
| Tests | 14 | ~2,000 | REVIEW/PARTIAL KEEP |
| Dead Code | 8 | ~1,700 | DELETE |
| CLI/Entry | 2 | ~500 | REFACTOR |

**Total Active Code**: ~25 files, ~4,600 LOC (from 50 files, ~13,000 LOC)

---

## KEEP - Core Builder Modules

These are the heart of the Linux distro builder:

### `build/linux.rs` (117 LOC) - ✅ KEEP
- Builds Linux kernel from `linux/` submodule
- Uses `levitate_defconfig`
- Supports x86_64 and aarch64 cross-compilation
- **No changes needed**

### `build/busybox.rs` (636 LOC) - ✅ KEEP
- Builds BusyBox from source with musl
- Native build + distrobox fallback
- aarch64 cross-compiler download
- Applet list for initramfs
- **No changes needed**

### `build/openrc.rs` (292 LOC) - ✅ KEEP
- Builds OpenRC with meson + musl
- Creates static binaries
- **No changes needed**

### `build/initramfs/` - ✅ KEEP (Core Product)
| File | LOC | Description |
|------|-----|-------------|
| `mod.rs` | 386 | Main builder, creates both BusyBox and OpenRC initramfs |
| `builder.rs` | ~200 | Event-based build system |
| `cpio.rs` | ~300 | Pure Rust CPIO writer |
| `manifest.rs` | ~200 | TOML manifest parser |
| `tui.rs` | ~150 | Terminal UI for build progress |

**No changes needed** - this is the crown jewel.

### `build/iso.rs` - ✅ KEEP
- Creates bootable Limine ISO
- Used for x86_64 boot
- **No changes needed**

---

## KEEP - QEMU/VM Runner

### `qemu/` - ✅ KEEP
| File | LOC | Description |
|------|-----|-------------|
| `mod.rs` | 11 | Module exports |
| `builder.rs` | ~300 | QEMU command builder pattern |
| `profile.rs` | ~100 | QEMU profiles (default, pixel6, x86_64) |

**No changes needed**

### `run.rs` (537 LOC) - ✅ KEEP
- `run_qemu()` - GUI mode
- `run_qemu_term()` - Terminal mode (custom kernel path)
- `run_qemu_term_linux()` - Linux kernel terminal mode ← **Primary**
- `run_qemu_gdb()` - GDB server
- `run_qemu_vnc()` - VNC display
- `verify_gpu()` - GPU verification

**Minor cleanup**: Remove custom kernel paths from `run_qemu_term()` if custom kernel is fully deprecated.

### `vm/` - ✅ KEEP
| File | LOC | Description |
|------|-----|-------------|
| `mod.rs` | 83 | Commands enum + exports |
| `session.rs` | ~150 | Persistent VM session |
| `exec.rs` | ~100 | One-shot command execution |
| `debug.rs` | ~100 | CPU regs, memory dump |

**No changes needed**

---

## KEEP - Support/Utilities

### `support/` - ✅ KEEP
| File | LOC | Description |
|------|-----|-------------|
| `mod.rs` | ~10 | Exports |
| `preflight.rs` | ~100 | Dependency checks |
| `clean.rs` | ~80 | Cleanup routines |
| `qmp.rs` | ~150 | QEMU QMP client |

**No changes needed**

### `disk/` - ✅ KEEP
| File | LOC | Description |
|------|-----|-------------|
| `mod.rs` | ~100 | Disk commands + exports |
| `image.rs` | ~150 | Disk image creation |

**Minor review needed**: May reference crates/userspace binaries.

### `config.rs` (~100 LOC) - ✅ KEEP
- Loads `xtask.toml` configuration
- Golden file ratings (gold/silver)
- **No changes needed**

### `calc.rs` (~200 LOC) - ✅ KEEP
- Calculator for memory/address/bit math
- Useful debugging tool
- **No changes needed**

---

## NEEDS REVIEW - Tests

### ✅ KEEP (Generic/Useful)
| File | LOC | Reason |
|------|-----|--------|
| `common.rs` | ~50 | Shared test utilities |
| `screenshot.rs` | ~100 | Screenshot capture |
| `screenshot_alpine.rs` | ~100 | Alpine screenshots |
| `screenshot_levitate.rs` | ~100 | Levitate screenshots |

### ⚠️ STALE (Custom Kernel Specific)
| File | LOC | Reason |
|------|-----|--------|
| `behavior.rs` | 385 | Tests custom kernel boot sequence, golden files reference kernel PID, shell, GPU |
| `regression.rs` | 1000+ | Tests GIC, MMU, GPU, VirtIO - all custom kernel internals |
| `coreutils.rs` | ~200 | Tests coreutils which was replaced by BusyBox |

### ⚠️ NEEDS REVIEW (May Work with Linux)
| File | LOC | Reason |
|------|-----|--------|
| `unit.rs` | ~100 | Host-side unit tests - probably still valid |
| `serial_input.rs` | ~100 | Serial input - may work with Linux |
| `keyboard_input.rs` | ~100 | Keyboard input - may work with Linux |
| `shutdown.rs` | ~100 | Shutdown test - may work with Linux |
| `backspace.rs` | ~50 | Backspace regression - may work with Linux |
| `debug_tools.rs` | ~200 | Debug tools - needs review |

---

## ❌ DELETE - Dead Code (Custom Kernel)

### `build/kernel.rs` (66 LOC) - ❌ DELETE
```rust
// Builds from crates/kernel - which is being deleted
let status = Command::new("cargo")
    .current_dir("crates/kernel")
    .args(&args)
```

### `build/userspace.rs` (31 LOC) - ❌ DELETE
```rust
// Builds from crates/userspace - which is being deleted
let status = Command::new("cargo")
    .current_dir("crates/userspace")
```

### `build/orchestration.rs` (52 LOC) - ❌ REWRITE
Still calls dead functions:
```rust
super::userspace::build_userspace(arch)?;  // DEAD
super::kernel::build_kernel_with_features(&[], arch)  // DEAD
```

**Must rewrite to:**
```rust
pub fn build_all(arch: &str) -> Result<()> {
    super::linux::build_linux_kernel(arch)?;
    super::busybox::ensure_built(arch)?;
    super::openrc::build(arch)?;
    super::initramfs::create_openrc_initramfs(arch)?;
    Ok(())
}
```

### `build/apps.rs` (~200 LOC) - ❌ DELETE
Empty registry:
```rust
pub static APPS: &[ExternalApp] = &[
    // TEAM_459: All apps removed - BusyBox is the single source of utilities
];
```

### `build/c_apps.rs` (~100 LOC) - ❌ DELETE
Empty registry:
```rust
pub static C_APPS: &[ExternalCApp] = &[
    // Empty - dash removed, BusyBox provides shell now
];
```

### `build/sysroot.rs` (~80 LOC) - ❌ DELETE
Just ensures musl target, not needed:
```rust
pub fn ensure_rust_musl_target(arch: &str) -> Result<()> {
    // Just checks rustup target
}
```

### `syscall/mod.rs` (1428 LOC) - ❌ DELETE
For custom kernel syscall development - completely dead.

### `build/alpine.rs` - ⚠️ DEPRECATED
Comment says "deprecated - use OpenRC instead". Review if anything useful.

---

## ⚠️ REFACTOR - Entry Points

### `main.rs` (432 LOC) - ⚠️ MAJOR REFACTOR NEEDED

**Dead code references:**
```rust
mod syscall;  // DELETE

// get_binaries() references crates/userspace
pub fn get_binaries(arch: &str) -> Result<Vec<String>> {
    let release_dir = PathBuf::from(format!("crates/userspace/target/{target}/release"));
    // ...
}
```

**CLI cleanup:**
- Remove `Commands::Syscall`
- Update build commands to only list valid targets
- Remove `get_binaries()` function or update for BusyBox

### `build/mod.rs` (41 LOC) - ⚠️ CLEANUP NEEDED

Remove re-exports of dead modules:
```rust
// DELETE these:
pub use orchestration::{build_all, build_kernel_only, build_kernel_verbose};
pub use userspace::build_userspace;

// These orchestration functions need rewriting
```

### `build/commands.rs` - ⚠️ REVIEW

Check which build commands reference dead code.

---

## Migration Path

### Phase 1: Remove Dead Code Files
```bash
rm xtask/src/build/kernel.rs
rm xtask/src/build/userspace.rs
rm xtask/src/build/apps.rs
rm xtask/src/build/c_apps.rs
rm xtask/src/build/sysroot.rs
rm -rf xtask/src/syscall/
```

### Phase 2: Rewrite orchestration.rs
Replace with Linux-first build:
```rust
pub fn build_all(arch: &str) -> Result<()> {
    super::linux::build_linux_kernel(arch)?;
    super::busybox::ensure_built(arch)?;
    super::openrc::build(arch)?;
    super::initramfs::create_openrc_initramfs(arch)?;
    Ok(())
}

// Remove build_kernel_only and build_kernel_verbose
```

### Phase 3: Update main.rs
- Remove syscall module import
- Remove `Commands::Syscall`
- Remove/update `get_binaries()`
- Update build commands

### Phase 4: Update build/mod.rs
- Remove dead re-exports
- Update public API

### Phase 5: Review Tests
- Decide which tests to keep/update for Linux+OpenRC
- Create new golden files for Linux boot

---

## Final File Count

**Before**: 50 files, ~13,000 LOC
**After**: ~25 files, ~4,600 LOC

**Reduction**: 50% fewer files, 65% less code
