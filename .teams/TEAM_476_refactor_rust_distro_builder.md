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

**Critical Work Done**:

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

- [ ] **Phase 1**: Tag and archive custom kernel
- [ ] **Phase 2**: Remove cruft + dead xtask modules
- [ ] **Phase 3**: Move xtask → src, rename build → builder
- [ ] **Phase 4**: Update docs, review tests
- [ ] **Phase 5**: Verify boot, update golden files

## Handoff Notes

This is a **planning and analysis session**. No code changes made yet.

**What was done**:
- Complete analysis of all xtask modules
- Identified specific dead code to delete
- Updated all phase documents with module-level detail
- Created xtask-analysis.md with full breakdown

**Next team should**:
1. Review `xtask-analysis.md` for complete module breakdown
2. Execute Phase 1 (archive) first
3. Pay special attention to Phase 2 xtask cleanup
4. Rewrite `orchestration.rs` (it calls dead functions)
5. Test at each checkpoint

## References

- [Alpine mkinitfs](https://github.com/alpinelinux/mkinitfs)
- [musl libc](https://musl.libc.org/)
- [BusyBox](https://busybox.net/)
- [OpenRC](https://github.com/OpenRC/openrc)
- TEAM_474: Linux kernel pivot
- TEAM_475: OpenRC integration
- TEAM_477: Plan review (found issues with original plan)
