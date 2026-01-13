# LevitateOS v2: Rust-Native Linux Distribution Builder

## Vision

LevitateOS is a **Rust-native Linux distribution builder** - a modern alternative to Alpine Linux's shell-script toolchain. We take the same ingredients (Linux kernel, musl, BusyBox, OpenRC) but wire them together with type-safe, fast Rust tooling.

**What Alpine does with 3,000+ lines of shell scripts, we do with Rust.**

---

## Current State (TEAM_475)

Linux + OpenRC is **already the default**:
```bash
cargo xtask run --term          # Linux + OpenRC (default)
cargo xtask run --term --minimal # Linux + BusyBox init
cargo xtask run --custom-kernel  # Deprecated custom kernel
```

**This refactor removes the deprecated paths and reorganizes the codebase.**

---

## Architecture Overview

### Before (Current Mess)
```
/                               # Unclear project identity
├── xtask/                      # "Task runner" but it's THE PRODUCT
│   └── src/build/initramfs/    # Crown jewel buried 3 levels deep
├── crates/                     # 41,000 LOC dead custom kernel
├── initramfs/                  # Config files (why separate?)
├── linux/                      # Submodule (good)
├── .external-kernels/          # Reference cruft
├── scripts/, qemu/, tmp/       # Shell cruft
└── ...20+ scattered directories
```

### After (Clean Structure)
```
levitate/
├── src/                        # Main binary - "levitate"
│   ├── main.rs                 # CLI entry point
│   ├── builder/                # THE PRODUCT - distro builder
│   │   ├── mod.rs
│   │   ├── initramfs/          # Initramfs builder (5 files)
│   │   ├── linux.rs            # Linux kernel builder
│   │   ├── busybox.rs          # BusyBox builder
│   │   ├── openrc.rs           # OpenRC builder
│   │   └── iso.rs              # ISO builder
│   ├── qemu/                   # QEMU runner (3 files)
│   ├── vm/                     # VM interaction (4 files)
│   ├── support/                # Utilities (4 files)
│   ├── disk/                   # Disk management (2 files)
│   ├── run.rs                  # Run commands
│   ├── config.rs               # Config loading
│   └── calc.rs                 # Calculator utility
│
├── config/                     # All configuration
│   ├── initramfs.toml          # Initramfs manifest
│   └── files/                  # Static files for initramfs
│
├── linux/                      # Git submodule (kernel source)
├── toolchain/                  # Build outputs (gitignored)
├── docs/                       # Documentation
├── tests/                      # Golden files
├── .teams/                     # Team logs
│
├── Cargo.toml                  # Single crate
├── CLAUDE.md
└── README.md
```

**~25 files, ~4,500 LOC** (was 50 files, ~13,000 LOC in xtask alone)

---

## xtask Module Breakdown

See `docs/planning/refactor-rust-distro-builder/xtask-analysis.md` for complete analysis.

### KEEP (Core Builder)
| Module | Files | LOC | Purpose |
|--------|-------|-----|---------|
| `linux.rs` | 1 | 117 | Linux kernel builder |
| `busybox.rs` | 1 | 636 | BusyBox builder |
| `openrc.rs` | 1 | 292 | OpenRC builder |
| `initramfs/` | 5 | ~1,200 | Initramfs builder + TUI |
| `iso.rs` | 1 | ~200 | ISO creator |

### KEEP (QEMU/VM)
| Module | Files | LOC | Purpose |
|--------|-------|-----|---------|
| `qemu/` | 3 | ~400 | QEMU command builder |
| `run.rs` | 1 | 537 | Run commands |
| `vm/` | 4 | ~400 | VM interaction |
| `support/` | 4 | ~400 | Utilities |
| `disk/` | 2 | ~250 | Disk management |

### DELETE (Dead Code)
| Module | LOC | Reason |
|--------|-----|--------|
| `build/kernel.rs` | 66 | Builds from deleted crates/kernel |
| `build/userspace.rs` | 31 | Builds from deleted crates/userspace |
| `build/apps.rs` | ~200 | Empty registry |
| `build/c_apps.rs` | ~100 | Empty registry |
| `build/sysroot.rs` | ~80 | Just ensures musl target |
| `syscall/` | 1,428 | Custom kernel syscall dev |

---

## What Changes

| From | To | Why |
|------|-----|-----|
| `xtask/` | `src/` | It's the product, not a task runner |
| `xtask/src/build/` | `src/builder/` | Clearer naming |
| `initramfs/` | `config/` | Consolidate all configs |
| `crates/` | (deleted) | Dead custom kernel code (41,000 LOC) |
| `.external-kernels/` | (deleted) | Reference cruft |
| `scripts/`, `qemu/`, `tmp/` | (deleted) | Shell cruft |
| `run*.sh` | (deleted) | Use `cargo run` instead |
| `xtask/src/build/kernel.rs` | (deleted) | Dead - builds from crates/kernel |
| `xtask/src/build/userspace.rs` | (deleted) | Dead - builds from crates/userspace |
| `xtask/src/syscall/` | (deleted) | Dead - custom kernel syscalls |

---

## CLI Changes

```bash
# Before
cargo xtask build initramfs
cargo xtask run --term

# After
cargo run -- build initramfs
cargo run -- run --term

# Or after `cargo install --path .`
levitate build initramfs
levitate run --term
```

---

## Phases

| Phase | Description | Key Tasks | Status |
|-------|-------------|-----------|--------|
| 1 | Safeguards | Tag archive, create branch, run tests | Pending |
| 2 | Remove cruft | Delete crates/, .external-kernels/, dead xtask modules | Pending |
| 3 | Restructure | Move xtask → src, consolidate config | Pending |
| 4 | Polish | Update imports, Cargo.toml, docs, review tests | Pending |
| 5 | Verify | All tests pass, boots to shell | Pending |

**Detailed plans:**
- [`phase-1.md`](../refactor-rust-distro-builder/phase-1.md) - Safeguards
- [`phase-2.md`](../refactor-rust-distro-builder/phase-2.md) - Remove Cruft
- [`phase-3.md`](../refactor-rust-distro-builder/phase-3.md) - Restructure
- [`phase-4.md`](../refactor-rust-distro-builder/phase-4.md) - Polish
- [`phase-5.md`](../refactor-rust-distro-builder/phase-5.md) - Verify
- [`xtask-analysis.md`](../refactor-rust-distro-builder/xtask-analysis.md) - Complete module breakdown
- [`cleanup-inventory.md`](../refactor-rust-distro-builder/cleanup-inventory.md) - File-by-file inventory

---

## Success Criteria

1. **Single command build**: `cargo run -- build all` produces bootable image
2. **Fast iteration**: Incremental builds < 10 seconds
3. **Clear structure**: Anyone can understand the codebase in 5 minutes
4. **Boots to shell**: `cargo run -- run --term` gets to OpenRC shell
5. **All tests pass**: Golden file comparisons succeed
6. **Reduced complexity**: ~25 files, ~4,500 LOC (was 50 files, 13,000+ LOC)

---

## What We're NOT Doing

- ~~Building musl from source~~ (system `musl-gcc` works)
- ~~Creating `overlay/` directory~~ (use `config/files/`)
- ~~Creating `init/init.sh`~~ (BusyBox init + inittab works)
- ~~Adding new features~~ (this is cleanup only)

---

## Core Ingredients

All statically linked with musl. No runtime dependencies.

| Component | Version | Purpose |
|-----------|---------|---------|
| **Linux** | 6.19-rc5 | Kernel |
| **musl** | system | C library (via musl-gcc) |
| **BusyBox** | 1.36.1 | Shell + 300 utilities |
| **OpenRC** | 0.54 | Init system + services |

---

## References

- [Alpine mkinitfs](https://github.com/alpinelinux/mkinitfs)
- [musl libc](https://musl.libc.org/)
- [BusyBox](https://busybox.net/)
- [OpenRC](https://github.com/OpenRC/openrc)
- TEAM_474: Linux kernel pivot
- TEAM_475: OpenRC integration
- TEAM_476: Initial planning
- TEAM_477: Plan review and revision
