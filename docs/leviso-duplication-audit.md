# leviso Consolidation Checklist

**Date:** 2026-02-10
**Goal:** Eliminate all code duplication between `leviso/` and `distro-builder/`, converging leviso into a thin wrapper like AcornOS/IuppiterOS.

**Current state:** leviso is ~11,348 LoC reimplementing what distro-builder already provides. AcornOS and IuppiterOS are each ~2,000 LoC thin wrappers. The end state is leviso at a similar size.

**Rule:** After EVERY completed task, rebuild the ISO and verify checkpoint 1 (live boot) still passes before moving on. A task is not done until the ISO boots.

```bash
# Verification sequence (run after every task)
cd leviso && cargo run -- build
cd testing/install-tests && cargo run --bin checkpoints -- --distro levitate --checkpoint 1
```

---

## Phase 1: Drop-In Replacements

These are copy-pasted modules that can be deleted and replaced with imports. No architectural changes needed — just swap the import path, fix call sites with `cargo check`, done.

### 1.1 Timer / Timing

- [ ] Delete `leviso/src/timing.rs`
- [ ] Add `distro_builder::alpine::timing::Timer` import (or re-export from distro-builder root)
- [ ] Update all `use crate::timing::Timer` → `use distro_builder::...::Timer`
- [ ] `cargo check -p leviso` passes
- [ ] Rebuild ISO + checkpoint 1 passes
- [ ] Commit: `fix(leviso): use distro-builder Timer instead of local copy`

**Files:**
| Delete | Replace with |
|--------|-------------|
| `leviso/src/timing.rs` | `distro-builder/src/alpine/timing.rs` |

**Details:** 100% identical. Same `Timer` struct, same `Display` impl, same `elapsed()` method. Zero risk swap.

### 1.2 Cache Module

- [ ] Delete `leviso/src/cache.rs`
- [ ] Import `distro_builder::cache` instead
- [ ] Update all `use crate::cache` → `use distro_builder::cache`
- [ ] Verify `hash_files()`, `needs_rebuild()`, `write_cached_hash()`, `is_newer()` signatures match
- [ ] `cargo check -p leviso` passes
- [ ] Rebuild ISO + checkpoint 1 passes
- [ ] Commit: `fix(leviso): use distro-builder cache module instead of local copy`

**Files:**
| Delete | Replace with |
|--------|-------------|
| `leviso/src/cache.rs` | `distro-builder/src/cache.rs` |

**Details:** 99% identical. Functions: `hash_files()`, `needs_rebuild()`, `write_cached_hash()`, `is_newer()`. Only difference is minor formatting. May need to verify distro-builder re-exports `cache` module publicly.

### 1.3 Executor — Directory Operations

- [ ] Delete `leviso/src/build/executor/directories.rs`
- [ ] Import `distro_builder::executor::directories` instead
- [ ] Update call sites: `create_dirs()`, `create_dir_all()`, `set_permissions()`
- [ ] `cargo check -p leviso` passes
- [ ] Rebuild ISO + checkpoint 1 passes
- [ ] Commit: `fix(leviso): use distro-builder executor directories instead of local copy`

**Files:**
| Delete | Replace with |
|--------|-------------|
| `leviso/src/build/executor/directories.rs` | `distro-builder/src/executor/directories.rs` |

**Details:** 100% identical code. Direct drop-in.

### 1.4 Executor — File Operations

- [ ] Delete `leviso/src/build/executor/files.rs`
- [ ] Import `distro_builder::executor::files` instead
- [ ] Update call sites: `copy_file()`, `copy_dir_recursive()`, `write_file()`, `symlink()`
- [ ] Check if 5% difference (error message wording) matters — if so, update distro-builder to accept the leviso variant or vice versa
- [ ] `cargo check -p leviso` passes
- [ ] Rebuild ISO + checkpoint 1 passes
- [ ] Commit: `fix(leviso): use distro-builder executor files instead of local copy`

**Files:**
| Delete | Replace with |
|--------|-------------|
| `leviso/src/build/executor/files.rs` | `distro-builder/src/executor/files.rs` |

**Details:** 95% identical. Minor differences in error messages only.

---

## Phase 2: Adopt Shared Abstractions

These require more thought — leviso's types are structurally similar but not identical to distro-builder's. Each needs a migration strategy.

### 2.1 BuildContext Unification

- [ ] Compare `leviso/src/build/context.rs` fields with `distro-builder/src/context.rs`
- [ ] Document field differences (if any)
- [ ] If distro-builder's BuildContext is a superset: delete leviso's, import distro-builder's
- [ ] If leviso has extra fields: either add them to distro-builder (if generic) or create a leviso wrapper struct that holds `distro_builder::BuildContext` + extras
- [ ] Update all `use crate::build::context::BuildContext` references
- [ ] `cargo check -p leviso` passes
- [ ] Rebuild ISO + checkpoint 1 passes
- [ ] Commit: `feat(leviso): adopt distro-builder BuildContext`

**Files:**
| leviso | distro-builder |
|--------|---------------|
| `leviso/src/build/context.rs` | `distro-builder/src/context.rs` |

**Details:** Both hold base_dir, output, staging paths. leviso may have Rocky/systemd-specific fields that need accommodation.

### 2.2 EROFS Rootfs Builder

- [ ] Compare `leviso/src/artifact/rootfs.rs` API with `distro-builder/src/artifact/erofs.rs`
- [ ] Document differences (Rocky vs Alpine paths, compression options, etc.)
- [ ] If distro-builder's is configurable enough: switch leviso to use it
- [ ] If not: extend distro-builder's erofs builder to support leviso's needs, then switch
- [ ] `cargo check -p leviso` passes
- [ ] Rebuild ISO + checkpoint 1 passes
- [ ] Built rootfs is byte-identical or functionally equivalent
- [ ] Commit: `feat(leviso): use distro-builder EROFS builder`

**Files:**
| leviso | distro-builder |
|--------|---------------|
| `leviso/src/artifact/rootfs.rs` | `distro-builder/src/artifact/erofs.rs` |

### 2.3 CPIO / Initramfs Builder

- [ ] Compare `leviso/src/artifact/initramfs.rs` with `distro-builder/src/artifact/cpio.rs`
- [ ] Document differences (busybox-based tiny init vs systemd-based)
- [ ] leviso builds TWO initramfs (tiny live + systemd install) — verify distro-builder supports both patterns
- [ ] Switch to distro-builder's cpio builder
- [ ] `cargo check -p leviso` passes
- [ ] Rebuild ISO + checkpoint 1 passes
- [ ] Both initramfs images boot correctly
- [ ] Commit: `feat(leviso): use distro-builder CPIO builder`

**Files:**
| leviso | distro-builder |
|--------|---------------|
| `leviso/src/artifact/initramfs.rs` | `distro-builder/src/artifact/cpio.rs` |

### 2.4 ISO Builder

- [ ] Compare `leviso/src/artifact/iso.rs` with how AcornOS/IuppiterOS create ISOs
- [ ] Determine if leviso can use `reciso` directly (like AcornOS does via distro-builder)
- [ ] Switch to shared ISO creation path
- [ ] `cargo check -p leviso` passes
- [ ] Rebuild ISO + checkpoint 1 passes
- [ ] ISO boots in QEMU
- [ ] Commit: `feat(leviso): use distro-builder ISO builder`

**Files:**
| leviso | distro-builder |
|--------|---------------|
| `leviso/src/artifact/iso.rs` | Uses `reciso` via distro-builder |

### 2.5 Rebuild Detection

- [ ] After cache module is shared (1.2), review `leviso/src/rebuild.rs`
- [ ] The `Artifact` struct itself is leviso-specific (tracks leviso's specific input files) — this is fine to keep
- [ ] Verify it now uses `distro_builder::cache` under the hood
- [ ] No action needed beyond 1.2 unless `Artifact` pattern should be shared
- [ ] Rebuild ISO + checkpoint 1 passes (if any changes made)

**Files:**
| leviso | shared |
|--------|--------|
| `leviso/src/rebuild.rs` | Uses `distro_builder::cache` after Phase 1.2 |

**Details:** The rebuild detection wrapper is leviso-specific (tracks leviso's artifacts). Only the underlying cache functions were duplicated. After 1.2, this is fine.

---

## Phase 3: Component System Migration

This is the largest change. leviso's bespoke component system (~2,500 LoC) gets replaced with distro-builder's `Installable` trait + `Op` enum pattern.

### 3.1 Understand the Installable Pattern

- [ ] Read `distro-builder/src/installable.rs` — understand `Installable` trait and `Op` enum
- [ ] Read AcornOS component definitions to see how they declare components as `Vec<Op>`
- [ ] Document the mapping: which leviso `component/custom/*.rs` functions map to which `Op` variants

### 3.2 Audit leviso Components

For each component file, determine the migration path:

- [ ] `leviso/src/component/definitions.rs` — list of components → becomes declarative `Vec<Op>` definitions
- [ ] `leviso/src/component/mod.rs` — orchestration → replaced by distro-builder executor
- [ ] `leviso/src/component/custom/etc.rs` — /etc file setup → `Op::WriteFile`, `Op::CopyFile`, `Op::Symlink`
- [ ] `leviso/src/component/custom/pam.rs` — PAM configuration → `Op::WriteFile` for each PAM file
- [ ] `leviso/src/component/custom/live.rs` — live overlay → may stay leviso-specific (live ISO is distro-specific)
- [ ] `leviso/src/component/custom/packages.rs` — package management → `Op::Shell` or recipe integration
- [ ] `leviso/src/component/custom/filesystem.rs` — filesystem setup → `Op::CreateDir`, `Op::Symlink`
- [ ] `leviso/src/component/custom/firmware.rs` — firmware handling → `Op::CopyDir` or similar
- [ ] `leviso/src/component/custom/modules.rs` — kernel module config → `Op::WriteFile`

### 3.3 Migrate Components (one at a time, rebuild after each)

- [ ] Migrate `etc.rs` → `Installable` + `Op` pattern
  - [ ] Rebuild ISO + checkpoint 1 passes
  - [ ] Commit
- [ ] Migrate `pam.rs` → `Installable` + `Op` pattern
  - [ ] Rebuild ISO + checkpoint 1 passes
  - [ ] Commit
- [ ] Migrate `filesystem.rs` → `Installable` + `Op` pattern
  - [ ] Rebuild ISO + checkpoint 1 passes
  - [ ] Commit
- [ ] Migrate `modules.rs` → `Installable` + `Op` pattern
  - [ ] Rebuild ISO + checkpoint 1 passes
  - [ ] Commit
- [ ] Migrate `firmware.rs` → `Installable` + `Op` pattern
  - [ ] Rebuild ISO + checkpoint 1 passes
  - [ ] Commit
- [ ] Migrate `packages.rs` → `Installable` + `Op` pattern
  - [ ] Rebuild ISO + checkpoint 1 passes
  - [ ] Commit
- [ ] Evaluate `live.rs` — keep if genuinely leviso-specific, migrate if it fits `Op` pattern
  - [ ] Rebuild ISO + checkpoint 1 passes
  - [ ] Commit
- [ ] Delete `leviso/src/component/definitions.rs` and `mod.rs` once all components are migrated
  - [ ] Rebuild ISO + checkpoint 1 passes
  - [ ] Commit

### 3.4 Implement DistroConfig

- [ ] Implement `distro_builder::DistroConfig` trait for LevitateOS
- [ ] Wire it into the build pipeline
- [ ] This enables distro-builder to identify which distro is being built
- [ ] Rebuild ISO + checkpoint 1 passes
- [ ] Commit: `feat(leviso): implement DistroConfig trait`

### 3.5 Build Orchestration

- [ ] Compare `leviso/src/build/mod.rs` with `distro-builder/src/builder.rs`
- [ ] Migrate leviso's build sequencing to use distro-builder's `Builder` abstraction
- [ ] Delete leviso's custom orchestration code
- [ ] Rebuild ISO + checkpoint 1 passes
- [ ] Commit: `feat(leviso): use distro-builder Builder for orchestration`

### 3.6 Final Verification

- [ ] `cargo check -p leviso` — compiles clean
- [ ] `cargo test -p leviso` — all tests pass
- [ ] Full build + all checkpoints pass (1-5)
- [ ] LoC count for leviso is under 4,000
- [ ] Commit: `feat(leviso): consolidation complete`

---

## Consolidation Efforts leviso Missed (Historical Context)

These are the key refactoring efforts in distro-builder / AcornOS / IuppiterOS that leviso didn't adopt. Understanding these helps prioritize the checklist above.

1. **Installable trait + Op enum** — Replaced per-component functions with declarative component definitions. AcornOS/IuppiterOS adopted; leviso still uses procedural `component/custom/*.rs`. (Phase 3)

2. **Shared artifact builders** — `distro-builder/src/artifact/erofs.rs` and `cpio.rs` extracted from AcornOS. leviso still has its own in `artifact/`. (Phase 2.2-2.4)

3. **Shared executor** — `distro-builder/src/executor/` extracted directory/file/user operations. leviso has identical code in `build/executor/`. (Phase 1.3-1.4)

4. **Cache module extraction** — `distro-builder/src/cache.rs` extracted for shared use. leviso still has its own copy. (Phase 1.2)

5. **BuildContext unification** — distro-builder's `BuildContext` became the standard. leviso still has its own. (Phase 2.1)

6. **DistroConfig trait** — distro-builder defines `DistroConfig` for distro identification. leviso doesn't use it. (Phase 3.4)

---

## End State

When all boxes are checked:
- leviso is a ~2,000-4,000 LoC thin wrapper
- `distro-spec/src/levitate/` provides all LevitateOS constants
- `distro-builder/` provides all build logic
- leviso only contains LevitateOS-specific wiring (which distro-spec constants to use, build order, CLI)
- Removing AcornOS/IuppiterOS leaves LevitateOS unbroken (and vice versa)
