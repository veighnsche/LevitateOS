# TEAM_476: Refactor to Rust-Native Linux Distribution Builder

## Objective

Transform LevitateOS from a "custom kernel OS" into a "Rust-native Linux distribution builder" - a modern alternative to Alpine Linux's shell-script toolchain.

**Key Insight**: The custom kernel (~39,000 LOC) is dead weight. The real value is in the `xtask/` build system that compiles Linux + musl + BusyBox + OpenRC from source.

## Progress Log

### Session 1 (2026-01-13) - Initial Planning

**Context**: User frustrated with custom kernel design. Quote: "This sucks man... The entire OS design is flawed... The main thing of our OS is the initramfs builder it seems..."

**Research Completed**:
1. Analyzed Alpine Linux build system
2. Compared to LevitateOS initramfs builder

**Decision**: Double down on build system. Archive custom kernel.

### Session 2 (2026-01-13) - Plan Review & xtask Analysis

**Context**: TEAM_477 reviewed the plan and found issues:
- Plan was not informed about current state (Linux+OpenRC already default)
- Proposed blindly moving xtask without analyzing stale code

### Session 3 (2026-01-13) - Phase 1 Execution

**Phase 1: Safeguards - COMPLETED**

1. **Created git tag**: `v1.0-custom-kernel`
2. **Created archive branch**: `archive/custom-kernel`
3. **Verified boot works**: `cargo xtask run --term` boots to OpenRC shell
4. **Created golden file**: `tests/golden_boot_linux_openrc.txt` (316 lines)
   - Linux version 6.19.0-rc5-levitate
   - OpenRC 0.54 startup
   - Services mounted (/proc, /run)
   - Shell prompt reached

**Rollback available**:
```bash
git checkout v1.0-custom-kernel      # Return to pre-refactor state
git checkout archive/custom-kernel   # View preserved custom kernel
```

**Critical Work Done in Session 2**:

1. **Complete xtask Module Analysis** (`xtask-analysis.md`):
   - Analyzed all 50 xtask source files
   - Categorized each as KEEP, DELETE, or REWRITE
   - Identified ~2,000 LOC of dead code in xtask itself

2. **Dead xtask Modules Identified**:
   | Module | LOC | Reason |
   |--------|-----|--------|
   | `build/kernel.rs` | 66 | Builds from deleted crates/kernel |
   | `build/userspace.rs` | 31 | Builds from deleted crates/userspace |
   | `build/apps.rs` | ~200 | Empty registry |
   | `build/c_apps.rs` | ~100 | Empty registry |
   | `build/sysroot.rs` | ~80 | Just ensures musl target |
   | `build/alpine.rs` | ~100 | Deprecated |
   | `syscall/mod.rs` | 1,428 | Custom kernel syscall dev |

3. **Updated Phase Documents**:
   - `phase-2.md`: Added specific xtask module deletions
   - `phase-3.md`: Added module structure after cleanup
   - `phase-4.md`: Added test review decisions
   - `phase-5.md`: Added verification for source structure
   - `cleanup-inventory.md`: Complete file-by-file inventory

4. **Key Discoveries**:
   - `orchestration.rs` still calls dead functions - needs rewrite
   - `behavior.rs` tests custom kernel boot - needs delete/rewrite
   - `regression.rs` tests kernel internals - needs delete
   - Several test modules may still work with Linux

## Key Decisions

1. **Archive, not delete custom kernel**: Preserve history in `archive/custom-kernel` branch
2. **Use system musl-gcc**: No need to build musl from source (it already works)
3. **Use BusyBox init + OpenRC**: BusyBox for inittab parsing, OpenRC for service management
4. **Focus on x86_64 first**: aarch64 support can come later
5. **Delete dead xtask modules**: Don't blindly move stale code

## Architecture (New)

```
cargo run -- build all
       │
       ├── linux     (from submodule)
       ├── busybox   (static musl)
       ├── openrc    (static musl)
       └── initramfs (CPIO archive)
              │
              ▼
       target/initramfs/
       └── x86_64-openrc.cpio
```

## Files Created/Modified

| File | Action |
|------|--------|
| `docs/planning/levitate-v2/PLAN.md` | Created → Updated with xtask breakdown |
| `docs/planning/refactor-rust-distro-builder/phase-1.md` | Created |
| `docs/planning/refactor-rust-distro-builder/phase-2.md` | Created → Updated with xtask module details |
| `docs/planning/refactor-rust-distro-builder/phase-3.md` | Created → Updated with target structure |
| `docs/planning/refactor-rust-distro-builder/phase-4.md` | Created → Updated with test review |
| `docs/planning/refactor-rust-distro-builder/phase-5.md` | Created → Updated with verification |
| `docs/planning/refactor-rust-distro-builder/cleanup-inventory.md` | Created → Updated with xtask files |
| `docs/planning/refactor-rust-distro-builder/xtask-analysis.md` | Created - Complete module analysis |

## What to Remove (Summary)

| Category | Items | ~LOC |
|----------|-------|------|
| Custom kernel | `crates/kernel/` | 30,000 |
| Userspace | `crates/userspace/` | 11,000 |
| Dead xtask modules | kernel.rs, userspace.rs, syscall/, etc. | ~2,000 |
| Shell wrappers | `run*.sh` scripts | 200 |
| Old configs | `linker.ld`, `limine.cfg` | 100 |
| IDE configs | `.idea/`, `.vscode/` | N/A |
| Reference code | `.external-kernels/` | N/A |

**Total to remove**: ~43,000+ LOC + cruft

## What to Keep (xtask modules)

| Component | Files | ~LOC |
|-----------|-------|------|
| linux.rs | 1 | 117 |
| busybox.rs | 1 | 636 |
| openrc.rs | 1 | 292 |
| initramfs/ | 5 | ~1,200 |
| iso.rs | 1 | ~200 |
| qemu/ | 3 | ~400 |
| run.rs | 1 | 537 |
| vm/ | 4 | ~400 |
| support/ | 4 | ~400 |
| disk/ | 2 | ~250 |
| main.rs, config.rs, calc.rs | 3 | ~700 |

**Total to keep**: ~25 files, ~4,500 LOC

## Remaining Work

- [x] **Phase 1**: Tag and archive custom kernel ✅
- [x] **Phase 2**: Remove cruft + dead xtask modules ✅
- [x] **Phase 3**: Move xtask → src, rename build → builder ✅
- [ ] **Phase 4**: Update docs, review tests
- [ ] **Phase 5**: Verify boot, update golden files

### Session 4 (2026-01-13) - Phase 2 Execution

**Phase 2: Remove Cruft + Dead xtask Modules - COMPLETED**

1. **Removed crates/** (~41,000 LOC):
   - Deleted entire custom kernel and userspace crates

2. **Removed cruft directories**:
   - `.external-kernels/`, `scripts/`, `qemu/`, `tmp/`

3. **Removed shell wrapper scripts**:
   - `run*.sh`, `limine.cfg`, `linker*.ld`

4. **Updated .gitignore**:
   - Added `.vscode/`, `.windsurf/`, `.claude/`, `.agent/`

5. **Removed dead xtask modules**:
   - `build/kernel.rs`, `build/userspace.rs`, `build/apps.rs`
   - `build/c_apps.rs`, `build/sysroot.rs`, `build/alpine.rs`
   - `build/iso.rs` (was broken, referenced deleted modules)
   - `syscall/` (entire module)

6. **Rewrote modules**:
   - `orchestration.rs`: Now only builds Linux+BusyBox+OpenRC
   - `build/mod.rs`: Removed dead module references
   - `main.rs`: Removed syscall command, --custom-kernel flag
   - `run.rs`: Removed all ISO and custom kernel paths
   - All test modules: Updated to use Linux+OpenRC boot
   - `vm/exec.rs`, `vm/session.rs`: Updated for Linux boot

7. **Updated Cargo.toml**:
   - Simplified workspace (only xtask member)
   - Removed kernel-specific dependencies
   - Updated profiles for CLI tool

8. **Build verification**:
   - `cargo build -p xtask` compiles successfully
   - `cargo xtask build openrc-initramfs` works

### Session 5 (2026-01-13) - Final Cleanup

**Context**: Continued from Session 4 after context compaction. Final cleanup of orphaned files.

**Removed orphaned build artifacts**:
- `initramfs/`, `initrd_root/`, `iso_root/`, `limine-bin/`
- `levitate.iso`, `tinyos_disk.img`, `kernel64_rust.bin`
- `initramfs_x86_64.cpio`, `userspace_bins/`

**Removed obsolete test artifacts**:
- Old golden files (kept only `tests/golden_boot_linux_openrc.txt`)
- `tests/coreutils/` (orphaned test directory)

**Removed orphaned directories**:
- `crates/` (only contained `userspace/target/` build cache)

**Removed obsolete docs**:
- `.agent/rules/kernel-development.md` (custom kernel rules)

**Git cleanup**:
- Untracked `.agent/` from git (was still tracked despite .gitignore)
- Verified `.vscode/`, `.windsurf/`, `.claude/` in .gitignore but not deleted from disk

**Build verification**:
- `cargo build -p xtask` compiles successfully (17 warnings about unused code - minor cleanup for Phase 4)

## Handoff Notes

**Phase 2 is FULLY complete.** The xtask now compiles and builds Linux+OpenRC.

**What was done**:
- Removed ~43,000 LOC of dead code (crates/, dead xtask modules)
- Rewrote all modules that referenced deleted code
- Updated all test modules for Linux boot
- Simplified Cargo.toml for distribution builder focus
- Cleaned up all orphaned build artifacts and test files
- Untracked IDE/AI config directories from git

**Remaining warnings** (minor, for Phase 4):
- Unused imports in `initramfs/mod.rs`
- Dead code in `initramfs/builder.rs`, `initramfs/manifest.rs`, `initramfs/tui.rs`
- Unused functions in `linux.rs`, `openrc.rs`, `qemu/builder.rs`

### Session 6 (2026-01-13) - Phase 3 Execution

**Phase 3: Restructure - COMPLETED**

1. **Moved xtask/src/* to src/**:
   - Main binary now at root level
   - Removed xtask/ directory entirely

2. **Renamed build/ to builder/**:
   - Semantic: "builder" describes what it IS

3. **Updated Cargo.toml**:
   - Package name: `levitate` (was `xtask`)
   - Version: `2.0.0`
   - Binary: `levitate` at `src/main.rs`

4. **Updated all import paths**:
   - `mod build;` → `mod builder;`
   - `build::` → `builder::`
   - `crate::build` → `crate::builder`

5. **Build verification**:
   - `cargo build` succeeds
   - `./target/debug/levitate --help` works

**New structure**:
```
src/
├── main.rs
├── builder/       (was build/)
│   ├── busybox.rs
│   ├── linux.rs
│   ├── openrc.rs
│   ├── initramfs/
│   └── ...
├── qemu/
├── vm/
├── support/
├── disk/
├── tests/
├── run.rs
├── config.rs
└── calc.rs
```

**Next team should**:
1. Execute Phase 4: Update CLAUDE.md and docs, fix warnings
2. Execute Phase 5: Verify boot matches golden file

## References

- [Alpine mkinitfs](https://github.com/alpinelinux/mkinitfs)
- [musl libc](https://musl.libc.org/)
- [BusyBox](https://busybox.net/)
- [OpenRC](https://github.com/OpenRC/openrc)
- TEAM_474: Linux kernel pivot
- TEAM_475: OpenRC integration
- TEAM_477: Plan review (found issues with original plan)
