# Phase 5: Verify

## Objective

Final verification that everything works. This phase is all testing, no code changes.

---

## Tasks

### 5.1 Build Verification

```bash
# Clean build from scratch
rm -rf target/
cargo build --release

# Check binary exists and runs
./target/release/levitate --help
```

Expected: Help text shows "levitate" not "xtask".

### 5.2 Component Build Tests

```bash
# Each component should build independently
cargo run -- build linux
cargo run -- build busybox
cargo run -- build openrc
cargo run -- build initramfs

# All together
cargo run -- build all
```

Expected: All succeed without errors.

### 5.3 Boot Test

```bash
# Serial console boot (default: Linux + OpenRC)
cargo run -- run --term
```

Expected output:
```
Linux version 6.19.0-rc5-levitate
...
OpenRC 0.54 is starting up Linux 6.19.0-rc5-levitate
 * Mounting /proc ... [ ok ]
 * Mounting /run ... [ ok ]
...
levitate#
```

Interactive tests in shell:
```bash
echo "Hello"       # Basic echo
ls /               # List root
rc-status          # OpenRC services
ps                 # Process list
exit               # Should work (Ctrl+A X to exit QEMU)
```

### 5.4 Golden File Test

Compare boot output against golden file:

```bash
timeout 30 cargo run -- run --term 2>&1 | head -100 > /tmp/boot_output.txt
diff tests/golden_boot_linux_openrc.txt /tmp/boot_output.txt
```

If intentional changes, update golden file:
```bash
cp /tmp/boot_output.txt tests/golden_boot_linux_openrc.txt
```

### 5.5 Code Quality Checks

```bash
# No compiler warnings
cargo build 2>&1 | grep -i warning

# No clippy warnings
cargo clippy -- -D warnings

# Properly formatted
cargo fmt --check

# Documentation builds
cargo doc --no-deps
```

### 5.6 File Count Verification

```bash
# Should be significantly fewer files
find . -name "*.rs" -not -path "./target/*" -not -path "./linux/*" | wc -l
```

Expected: < 30 files (was 200+).

### 5.7 Directory Structure Verification

```bash
ls -la
```

Expected structure:
```
.
├── src/
├── config/
├── linux/
├── toolchain/
├── docs/
├── tests/
├── .teams/
├── .agent/
├── .cargo/
├── Cargo.toml
├── Cargo.lock
├── CLAUDE.md
├── README.md
└── (few other files)
```

Should NOT have:
- `xtask/`
- `crates/`
- `.external-kernels/`
- `scripts/`
- `run*.sh`

### 5.8 Source Structure Verification

```bash
ls -la src/
```

Expected:
```
src/
├── main.rs
├── builder/       # Core build system
├── qemu/          # QEMU management
├── vm/            # VM interaction
├── support/       # Utilities
├── disk/          # Disk management
├── tests/         # Test modules
├── run.rs
├── config.rs
└── calc.rs
```

### 5.9 Builder Module Verification

```bash
ls -la src/builder/
```

Expected:
```
src/builder/
├── mod.rs
├── linux.rs       # Linux kernel builder
├── busybox.rs     # BusyBox builder
├── openrc.rs      # OpenRC builder
├── initramfs/     # Initramfs builder (5 files)
├── iso.rs         # ISO creator
├── orchestration.rs  # Build coordination
└── commands.rs    # CLI enum
```

Should NOT have:
- `kernel.rs` (deleted - was for custom kernel)
- `userspace.rs` (deleted)
- `apps.rs` (deleted - empty)
- `c_apps.rs` (deleted - empty)
- `sysroot.rs` (deleted)
- `alpine.rs` (deleted - deprecated)

---

## Success Criteria

| Criterion | How to Verify |
|-----------|---------------|
| Builds | `cargo build --release` succeeds |
| Boots | `cargo run -- run --term` reaches shell |
| Shell works | Can run `ls`, `echo`, `ps` |
| OpenRC works | `rc-status` shows services |
| No warnings | `cargo clippy` clean |
| Formatted | `cargo fmt --check` passes |
| Docs build | `cargo doc` succeeds |
| Clean structure | No old directories remain |
| Binary name | `./target/release/levitate --help` shows "levitate" |
| File count | < 30 source files |

---

## Final Commit

```bash
git add -A
git commit -m "refactor(v2): Restructure as Rust-native distro builder

TEAM_476/477: Major restructure of LevitateOS

## What Changed
- Removed custom kernel (41,000 LOC) - archived to branch
- Removed dead xtask modules (~2,000 LOC):
  - build/kernel.rs, userspace.rs, apps.rs, c_apps.rs, sysroot.rs
  - syscall/ module
- Moved xtask/src/ to src/ - it's the product, not a task runner
- Renamed build/ to builder/ - clearer semantics
- Consolidated config files to config/
- Removed all shell wrapper scripts
- Updated all documentation

## New Structure
- src/builder/ - Core distro builder (linux, busybox, openrc, initramfs)
- src/qemu/ - QEMU command builder
- src/vm/ - VM interaction tools
- config/ - All build configs
- linux/ - Kernel submodule

## CLI
- cargo run -- build all
- cargo run -- run --term

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Handoff Notes

### What Was Done
1. Archived custom kernel to `archive/custom-kernel` branch
2. Removed 41,000+ LOC of dead kernel code
3. Removed ~2,000 LOC of dead xtask modules
4. Restructured `xtask/` → `src/`
5. Consolidated configs to `config/`
6. Updated all documentation

### What Works
- `cargo run -- build all` produces bootable image
- `cargo run -- run --term` boots to OpenRC shell
- All build components work independently

### xtask Modules Retained
| Module | Files | LOC | Purpose |
|--------|-------|-----|---------|
| `builder/linux.rs` | 1 | 117 | Linux kernel builder |
| `builder/busybox.rs` | 1 | 636 | BusyBox builder |
| `builder/openrc.rs` | 1 | 292 | OpenRC builder |
| `builder/initramfs/` | 5 | ~1,200 | Initramfs builder + TUI |
| `builder/iso.rs` | 1 | ~200 | ISO creator |
| `qemu/` | 3 | ~400 | QEMU management |
| `vm/` | 4 | ~400 | VM interaction |
| `support/` | 4 | ~400 | Utilities |
| `disk/` | 2 | ~250 | Disk management |
| Other | 3 | ~500 | main, config, calc |

**Total: ~25 files, ~4,500 LOC**

### Known Limitations
1. **x86_64 only** - aarch64 needs work
2. **No networking** - virtio-net loads but no DHCP
3. **Initramfs only** - no persistent root filesystem

### Future Work
1. aarch64 support
2. Networking (DHCP, SSH)
3. Persistent disk support
4. More OpenRC services

---

## Checklist

- [ ] `cargo build --release` succeeds
- [ ] `cargo run -- run --term` boots to shell
- [ ] Shell commands work (ls, echo, ps)
- [ ] `rc-status` shows services
- [ ] `cargo clippy` - no warnings
- [ ] `cargo fmt --check` passes
- [ ] `cargo doc` succeeds
- [ ] Golden file matches (or updated)
- [ ] No old directories remain:
  - [ ] No `xtask/`
  - [ ] No `crates/`
  - [ ] No `.external-kernels/`
  - [ ] No `scripts/`
  - [ ] No `run*.sh`
- [ ] Source structure correct:
  - [ ] `src/builder/` exists with linux, busybox, openrc, initramfs, iso
  - [ ] No `src/builder/kernel.rs` (deleted)
  - [ ] No `src/syscall/` (deleted)
- [ ] Binary named `levitate`
- [ ] < 30 source files
- [ ] Final commit made
- [ ] Team file updated with handoff notes
