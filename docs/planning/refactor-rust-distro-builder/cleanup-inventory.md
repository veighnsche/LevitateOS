# Cleanup Inventory

Complete inventory of files/directories and their fate in the refactor.

## Legend

- **DELETE**: Remove entirely
- **KEEP**: Keep as-is
- **MOVE**: Relocate to new path
- **REWRITE**: Keep file but major changes needed
- **ARCHIVE**: Move to `archive/custom-kernel` branch

---

## Top-Level Directories

### DELETE
| Path | Reason |
|------|--------|
| `crates/` | Dead custom kernel (41,000 LOC) |
| `.external-kernels/` | Reference code cruft |
| `scripts/` | Old shell scripts |
| `qemu/` | Old QEMU configs |
| `tmp/` | Temporary files |
| `.idea/` | IDE config |
| `.vscode/` | IDE config |
| `.windsurf/` | IDE config |
| `xtask/initrd_resources/` | Old initrd stuff |
| `xtask/data/` | Old data files |

### MOVE
| From | To | Reason |
|------|-----|--------|
| `xtask/src/` | `src/` | It's the product |
| `xtask/src/build/` | `src/builder/` | Clearer naming |
| `initramfs/` | `config/` | Consolidate configs |

### KEEP
| Path | Reason |
|------|--------|
| `linux/` | Kernel submodule |
| `toolchain/` | Build outputs (gitignored) |
| `docs/` | Documentation |
| `tests/` | Test files |
| `.teams/` | Historical context |
| `.agent/` | AI rules |
| `.cargo/` | Cargo config |
| `.claude/` | Claude config |

---

## Top-Level Files

### DELETE
| File | Reason |
|------|--------|
| `run.sh` | Use `cargo run -- run` |
| `run-term.sh` | Use `cargo run -- run --term` |
| `run-test.sh` | Use `cargo run -- test` |
| `run-vnc.sh` | Use `cargo run -- run --vnc` |
| `run-pixel6.sh` | Use `cargo run -- run --profile pixel6` |
| `screenshot.sh` | Use `cargo run -- vm screenshot` |
| `test_libsyscall.sh` | Obsolete |
| `limine.cfg` | Custom kernel bootloader |
| `linker.ld` | Custom kernel linker |
| `linker_repro.ld` | Debug linker |

### KEEP (Update)
| File | Changes Needed |
|------|----------------|
| `Cargo.toml` | Merge with xtask/Cargo.toml, rename package |
| `CLAUDE.md` | Major rewrite |
| `README.md` | Complete rewrite |
| `.gitignore` | Update paths |
| `Cargo.lock` | Will regenerate |
| `rust-toolchain.toml` | Keep as-is |
| `xtask.toml` | Rename to `levitate.toml`? Or delete |
| `LICENSE` | Keep |

---

## xtask Source Files

### DELETE (Dead Code)
| Path | LOC | Reason |
|------|-----|--------|
| `xtask/src/build/kernel.rs` | 66 | Builds from deleted crates/kernel |
| `xtask/src/build/userspace.rs` | 31 | Builds from deleted crates/userspace |
| `xtask/src/build/apps.rs` | ~200 | Empty registry (APPS = &[]) |
| `xtask/src/build/c_apps.rs` | ~100 | Empty registry (C_APPS = &[]) |
| `xtask/src/build/sysroot.rs` | ~80 | Just ensures musl target |
| `xtask/src/build/alpine.rs` | ~100 | Deprecated per comment |
| `xtask/src/syscall/mod.rs` | 1,428 | Custom kernel syscall dev |

### REWRITE (Still Needed But Broken)
| Path | LOC | Changes |
|------|-----|---------|
| `xtask/src/build/orchestration.rs` | 52 | Remove calls to dead functions, rewrite for Linux+OpenRC |
| `xtask/src/build/mod.rs` | 41 | Remove dead re-exports |
| `xtask/src/main.rs` | 432 | Remove syscall, --custom-kernel, get_binaries() |

### KEEP (Core Builder)
| Old Path | New Path | LOC |
|----------|----------|-----|
| `xtask/src/build/linux.rs` | `src/builder/linux.rs` | 117 |
| `xtask/src/build/busybox.rs` | `src/builder/busybox.rs` | 636 |
| `xtask/src/build/openrc.rs` | `src/builder/openrc.rs` | 292 |
| `xtask/src/build/initramfs/mod.rs` | `src/builder/initramfs/mod.rs` | 386 |
| `xtask/src/build/initramfs/builder.rs` | `src/builder/initramfs/builder.rs` | ~200 |
| `xtask/src/build/initramfs/cpio.rs` | `src/builder/initramfs/cpio.rs` | ~300 |
| `xtask/src/build/initramfs/manifest.rs` | `src/builder/initramfs/manifest.rs` | ~200 |
| `xtask/src/build/initramfs/tui.rs` | `src/builder/initramfs/tui.rs` | ~150 |
| `xtask/src/build/iso.rs` | `src/builder/iso.rs` | ~200 |
| `xtask/src/build/commands.rs` | `src/builder/commands.rs` | ~50 |

### KEEP (QEMU/VM)
| Old Path | New Path | LOC |
|----------|----------|-----|
| `xtask/src/qemu/mod.rs` | `src/qemu/mod.rs` | 11 |
| `xtask/src/qemu/builder.rs` | `src/qemu/builder.rs` | ~300 |
| `xtask/src/qemu/profile.rs` | `src/qemu/profile.rs` | ~100 |
| `xtask/src/run.rs` | `src/run.rs` | 537 |
| `xtask/src/vm/mod.rs` | `src/vm/mod.rs` | 83 |
| `xtask/src/vm/session.rs` | `src/vm/session.rs` | ~150 |
| `xtask/src/vm/exec.rs` | `src/vm/exec.rs` | ~100 |
| `xtask/src/vm/debug.rs` | `src/vm/debug.rs` | ~100 |

### KEEP (Support)
| Old Path | New Path | LOC |
|----------|----------|-----|
| `xtask/src/support/mod.rs` | `src/support/mod.rs` | ~10 |
| `xtask/src/support/preflight.rs` | `src/support/preflight.rs` | ~100 |
| `xtask/src/support/clean.rs` | `src/support/clean.rs` | ~80 |
| `xtask/src/support/qmp.rs` | `src/support/qmp.rs` | ~150 |
| `xtask/src/disk/mod.rs` | `src/disk/mod.rs` | ~100 |
| `xtask/src/disk/image.rs` | `src/disk/image.rs` | ~150 |
| `xtask/src/config.rs` | `src/config.rs` | ~100 |
| `xtask/src/calc.rs` | `src/calc.rs` | ~200 |

### REVIEW (Tests)
| Old Path | New Path | Status |
|----------|----------|--------|
| `xtask/src/tests/mod.rs` | `src/tests/mod.rs` | Update to remove dead modules |
| `xtask/src/tests/common.rs` | `src/tests/common.rs` | KEEP |
| `xtask/src/tests/screenshot.rs` | `src/tests/screenshot.rs` | KEEP |
| `xtask/src/tests/screenshot_alpine.rs` | `src/tests/screenshot_alpine.rs` | KEEP |
| `xtask/src/tests/screenshot_levitate.rs` | `src/tests/screenshot_levitate.rs` | KEEP |
| `xtask/src/tests/unit.rs` | `src/tests/unit.rs` | KEEP |
| `xtask/src/tests/behavior.rs` | `src/tests/behavior.rs` | DELETE or REWRITE |
| `xtask/src/tests/regression.rs` | `src/tests/regression.rs` | DELETE |
| `xtask/src/tests/coreutils.rs` | `src/tests/coreutils.rs` | DELETE |
| `xtask/src/tests/serial_input.rs` | `src/tests/serial_input.rs` | REVIEW |
| `xtask/src/tests/keyboard_input.rs` | `src/tests/keyboard_input.rs` | REVIEW |
| `xtask/src/tests/shutdown.rs` | `src/tests/shutdown.rs` | REVIEW |
| `xtask/src/tests/backspace.rs` | `src/tests/backspace.rs` | REVIEW |
| `xtask/src/tests/debug_tools.rs` | `src/tests/debug_tools.rs` | REVIEW |

---

## Config Files

### MOVE to `config/`
| From | To |
|------|-----|
| `initramfs/initramfs.toml` | `config/initramfs.toml` |
| `initramfs/files/` | `config/files/` |
| `initramfs/scripts/` | `config/scripts/` (or delete if unused) |

---

## Cargo.toml Changes

### Before (workspace)
```toml
[workspace]
members = ["xtask"]
exclude = ["crates/userspace", "crates/kernel"]
```

### After (single crate)
```toml
[package]
name = "levitate"
version = "2.0.0"
edition = "2021"

[[bin]]
name = "levitate"
path = "src/main.rs"

[dependencies]
# Merged from xtask/Cargo.toml
```

---

## Final Structure

```
levitate/
├── src/
│   ├── main.rs
│   ├── builder/
│   │   ├── mod.rs
│   │   ├── initramfs/
│   │   │   ├── mod.rs
│   │   │   ├── builder.rs
│   │   │   ├── cpio.rs
│   │   │   ├── manifest.rs
│   │   │   └── tui.rs
│   │   ├── linux.rs
│   │   ├── busybox.rs
│   │   ├── openrc.rs
│   │   ├── iso.rs
│   │   ├── orchestration.rs
│   │   └── commands.rs
│   ├── qemu/
│   │   ├── mod.rs
│   │   ├── builder.rs
│   │   └── profile.rs
│   ├── vm/
│   │   ├── mod.rs
│   │   ├── session.rs
│   │   ├── exec.rs
│   │   └── debug.rs
│   ├── support/
│   │   ├── mod.rs
│   │   ├── preflight.rs
│   │   ├── clean.rs
│   │   └── qmp.rs
│   ├── disk/
│   │   ├── mod.rs
│   │   └── image.rs
│   ├── tests/
│   │   └── ... (reviewed)
│   ├── run.rs
│   ├── config.rs
│   └── calc.rs
├── config/
│   ├── initramfs.toml
│   ├── files/
│   │   └── etc/
│   └── scripts/
├── linux/
├── toolchain/
├── docs/
├── tests/
├── .teams/
├── .agent/
├── .cargo/
├── Cargo.toml
├── CLAUDE.md
└── README.md
```

**Target: ~25 files in src/, ~10 config files, total < 35 active files**
