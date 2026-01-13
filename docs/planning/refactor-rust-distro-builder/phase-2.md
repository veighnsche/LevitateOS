# Phase 2: Remove Cruft

## Objective

Delete all dead code and unnecessary files. This is pure subtraction.

---

## Tasks

### 2.1 Remove Custom Kernel Code

```bash
git rm -r crates/
```

This removes:
- `crates/kernel/` - 30,000+ LOC custom kernel
- `crates/userspace/` - 11,000+ LOC Eyra userspace

### 2.2 Remove Reference/External Code

```bash
rm -rf .external-kernels/
```

This removes reference kernels (redox, theseus, tock, brush, alpine-aports).

### 2.3 Remove Shell Cruft

```bash
# Shell wrapper scripts
rm -f run.sh run-term.sh run-test.sh run-vnc.sh run-pixel6.sh
rm -f screenshot.sh test_libsyscall.sh

# Old directories
rm -rf scripts/
rm -rf qemu/
rm -rf tmp/
```

### 2.4 Remove IDE Configs

```bash
rm -rf .idea/
rm -rf .vscode/
rm -rf .windsurf/
```

### 2.5 Remove Dead xtask Modules

**Critical: These xtask modules are dead code referencing deleted crates/**

```bash
# Delete dead build modules
rm xtask/src/build/kernel.rs       # 66 LOC - builds from crates/kernel
rm xtask/src/build/userspace.rs    # 31 LOC - builds from crates/userspace
rm xtask/src/build/apps.rs         # ~200 LOC - empty registry
rm xtask/src/build/c_apps.rs       # ~100 LOC - empty registry
rm xtask/src/build/sysroot.rs      # ~80 LOC - just ensures musl target
rm xtask/src/build/alpine.rs       # deprecated per comment

# Delete syscall module (custom kernel development only)
rm -rf xtask/src/syscall/          # 1428 LOC - syscall spec fetcher
```

### 2.6 Rewrite orchestration.rs

The current `xtask/src/build/orchestration.rs` calls dead functions:

**Before (broken):**
```rust
pub fn build_all(arch: &str) -> Result<()> {
    super::sysroot::ensure_rust_musl_target(arch)?;  // DELETE
    super::apps::ensure_all_built(arch)?;            // DELETE (empty)
    super::userspace::build_userspace(arch)?;        // DELETE (dead)
    super::initramfs::create_busybox_initramfs(arch)?;
    super::kernel::build_kernel_with_features(&[], arch)  // DELETE (dead)
}
```

**After (working):**
```rust
pub fn build_all(arch: &str) -> Result<()> {
    super::linux::build_linux_kernel(arch)?;
    super::busybox::ensure_built(arch)?;
    super::openrc::build(arch)?;
    super::initramfs::create_openrc_initramfs(arch)?;
    Ok(())
}

// DELETE these functions entirely:
// - build_kernel_only()
// - build_kernel_verbose()
```

### 2.7 Update build/mod.rs

Remove dead re-exports:

**Before:**
```rust
mod kernel;
mod userspace;
mod apps;
mod c_apps;
mod sysroot;

pub use orchestration::{build_all, build_kernel_only, build_kernel_verbose};
pub use userspace::build_userspace;
```

**After:**
```rust
// Only keep these modules:
pub mod linux;
pub mod busybox;
pub mod openrc;
mod initramfs;
mod iso;
mod orchestration;
mod commands;

pub use commands::BuildCommands;
pub use initramfs::{create_busybox_initramfs, create_openrc_initramfs};
pub use iso::{build_iso, build_iso_test, build_iso_verbose};
pub use orchestration::build_all;
```

### 2.8 Update main.rs

**Remove:**
```rust
mod syscall;  // DELETE entire module

// DELETE Commands::Syscall variant
Commands::Syscall(cmd) => match cmd { ... }

// DELETE or update get_binaries() - references crates/userspace
pub fn get_binaries(arch: &str) -> Result<Vec<String>> {
    let release_dir = PathBuf::from(format!("crates/userspace/target/{target}/release"));
    // This path no longer exists
}
```

**Simplify RunArgs:**
```rust
// Remove --custom-kernel flag (it's deprecated)
// Make Linux+OpenRC the ONLY path, not an option
```

### 2.9 Clean Up Workspace

Update root `Cargo.toml`:

```toml
# Before
[workspace]
members = ["xtask"]
exclude = ["crates/userspace", "crates/kernel"]

# After
[workspace]
members = ["xtask"]
```

---

## Files to Remove (Complete List)

### Top-Level Directories
| Path | Reason |
|------|--------|
| `crates/` | Dead custom kernel (41,000+ LOC) |
| `.external-kernels/` | Reference cruft |
| `scripts/` | Old shell scripts |
| `qemu/` | Old QEMU configs |
| `tmp/` | Temporary files |
| `.idea/` | IDE config |
| `.vscode/` | IDE config |
| `.windsurf/` | IDE config |
| `xtask/initrd_resources/` | Old initrd stuff |
| `xtask/data/` | Old data files |

### xtask Dead Modules
| Path | LOC | Reason |
|------|-----|--------|
| `xtask/src/build/kernel.rs` | 66 | Builds from deleted crates/kernel |
| `xtask/src/build/userspace.rs` | 31 | Builds from deleted crates/userspace |
| `xtask/src/build/apps.rs` | ~200 | Empty registry |
| `xtask/src/build/c_apps.rs` | ~100 | Empty registry |
| `xtask/src/build/sysroot.rs` | ~80 | Just ensures musl target |
| `xtask/src/build/alpine.rs` | ~100 | Deprecated |
| `xtask/src/syscall/mod.rs` | 1428 | Custom kernel syscall dev |

### Top-Level Files
| Path | Reason |
|------|--------|
| `run.sh` | Use `cargo xtask run` |
| `run-term.sh` | Use `cargo xtask run --term` |
| `run-test.sh` | Use `cargo xtask test` |
| `run-vnc.sh` | Use `cargo xtask run --vnc` |
| `run-pixel6.sh` | Use `cargo xtask run --profile pixel6` |
| `screenshot.sh` | Use `cargo xtask vm screenshot` |
| `test_libsyscall.sh` | Obsolete |
| `limine.cfg` | Custom kernel bootloader |
| `linker.ld` | Custom kernel linker |
| `linker_repro.ld` | Debug linker |

---

## xtask Modules Being KEPT

| Path | LOC | Description |
|------|-----|-------------|
| `build/linux.rs` | 117 | Linux kernel builder |
| `build/busybox.rs` | 636 | BusyBox builder |
| `build/openrc.rs` | 292 | OpenRC builder |
| `build/initramfs/` | ~1,200 | Core initramfs builder (5 files) |
| `build/iso.rs` | ~200 | ISO creator |
| `build/orchestration.rs` | 52 | **REWRITE** |
| `build/commands.rs` | ~50 | CLI enum |
| `qemu/` | ~400 | QEMU builder (3 files) |
| `run.rs` | 537 | Run commands |
| `vm/` | ~400 | VM interaction (4 files) |
| `support/` | ~400 | Utilities (4 files) |
| `disk/` | ~250 | Disk management (2 files) |
| `config.rs` | ~100 | Config loading |
| `calc.rs` | ~200 | Calculator |
| `main.rs` | 432 | **UPDATE** |
| `tests/` | ~2000 | **REVIEW** |

---

## Verification

After removal:

```bash
# Should still build
cargo build -p xtask

# Should still boot (this is the critical test)
cargo xtask run --term

# xtask file count
find xtask/src -name "*.rs" | wc -l
# Target: ~25 files (was 50)

# Total .rs file count
find . -name "*.rs" -not -path "./target/*" -not -path "./linux/*" | wc -l
# Target: ~25 files (was 200+)
```

---

## Checklist

- [ ] `git rm -r crates/`
- [ ] `rm -rf .external-kernels/`
- [ ] Removed shell wrapper scripts
- [ ] Removed IDE configs
- [ ] Removed dead xtask modules:
  - [ ] `xtask/src/build/kernel.rs`
  - [ ] `xtask/src/build/userspace.rs`
  - [ ] `xtask/src/build/apps.rs`
  - [ ] `xtask/src/build/c_apps.rs`
  - [ ] `xtask/src/build/sysroot.rs`
  - [ ] `xtask/src/build/alpine.rs`
  - [ ] `xtask/src/syscall/`
- [ ] Rewrote `orchestration.rs`
- [ ] Updated `build/mod.rs`
- [ ] Updated `main.rs` (removed syscall, custom-kernel flag)
- [ ] Updated root `Cargo.toml`
- [ ] `cargo build -p xtask` succeeds
- [ ] `cargo xtask run --term` boots to shell
- [ ] Ready for Phase 3
