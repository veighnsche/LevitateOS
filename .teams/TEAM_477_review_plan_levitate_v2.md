# TEAM_477: Review Plan - Levitate v2

## Objective
Review the Levitate v2 plan (`docs/planning/levitate-v2/PLAN.md`) for completeness, architectural alignment, and compliance with project rules.

## Progress Log

### Session 1 (2026-01-13)

**Context**: TEAM_476 created a comprehensive plan to pivot LevitateOS from a custom kernel to a Rust-native Linux distribution builder.

**Review Checklist**:
- [x] Questions audit
- [x] Scope check (over/under engineering)
- [x] Architecture alignment
- [x] Rules compliance
- [x] Claims verification
- [x] Refinements applied

## Review Summary

### 0. Current State (Critical Context)

**TEAM_475 already made Linux + OpenRC the default.** The current codebase:
- `cargo xtask run --term` → Linux + OpenRC (default)
- `cargo xtask run --term --minimal` → Linux + BusyBox init
- `cargo xtask run --term --custom-kernel` → Old custom kernel (deprecated)

The plan is about **removing the deprecated `--custom-kernel` path** and the `crates/` directory, NOT adding Linux support. This is a cleanup of dead code, not a migration.

### 1. Questions Audit

**Status: PASS**

No open questions block this plan. The existing questions in `docs/questions/` relate to:
- TEAM_400-405: General purpose OS design (archived approach)
- TEAM_444: dash shell questions (still relevant for shell integration)
- TEAM_449: BusyBox integration (directly applicable)
- TEAM_469: procfs design (not relevant to refactor)

The plan correctly pivots away from the custom kernel approach, making most of the old questions moot. The BusyBox/dash questions are still relevant and the plan addresses them.

### 2. Scope Check

**Status: MINOR ISSUES**

**Well-Scoped:**
- Clear phase separation (5 phases)
- Concrete deliverables at each phase
- Reasonable LOC targets
- Good verification checkpoints

**Potential Over-engineering:**
| Issue | Location | Recommendation |
|-------|----------|----------------|
| Building musl from source | Phase 2 | Consider: system `musl-gcc` works, building adds complexity. Should be optional. |
| Renaming `initramfs/` to `configs/` + `overlay/` | Phase 2-4 | **UNNECESSARY** - see below |

**Critical Note on Directory Structure:**

The plan proposes creating `configs/initramfs.toml` and `overlay/etc/`, but these already exist:
- `initramfs/initramfs.toml` - 159 lines, fully working declarative manifest
- `initramfs/files/etc/` - Contains inittab, passwd, group, profile

Renaming these provides **zero value** and creates unnecessary churn. The plan should instead:
1. Keep `initramfs/` as-is
2. Just remove `crates/` and the `--custom-kernel` code paths

**Potential Under-engineering:**
| Issue | Location | Recommendation |
|-------|----------|----------------|
| aarch64 support punted | Phase 5 | Fine for v2, but should be explicit in "Won't Have" |

### 3. Architecture Alignment

**Status: NEEDS REVISION**

**Aligned:**
- Uses existing `xtask/` structure correctly
- Module boundaries respected (Rule 7)
- Private fields with public APIs

**Misaligned:**
- Plan proposes `configs/` + `overlay/` but `initramfs/` already serves this purpose
- Plan proposes new TOML manifest format but existing one is complete (159 lines)
- Plan proposes `init/init.sh` but BusyBox init + `/etc/inittab` already handles this

**Actual Scope Should Be:**
1. Remove `crates/` (git rm -r)
2. Remove `--custom-kernel` flag and related code paths
3. Remove shell wrapper scripts (`run*.sh`)
4. Update Cargo.toml workspace members
5. Update CLAUDE.md and README.md
6. **That's it.** Don't reorganize what already works.

### 4. Rules Compliance

| Rule | Status | Notes |
|------|--------|-------|
| Rule 0 (Quality > Speed) | ✅ PASS | Plan prioritizes clean architecture |
| Rule 4 (Silence is Golden) | ✅ PASS | Phase 5 mentions golden tests |
| Rule 5 (No Compatibility Hacks) | ✅ PASS | Explicitly uses breaking changes |
| Rule 6 (Cleanup Phase) | ✅ PASS | Phase 4 is dedicated cleanup |
| Rule 7 (Module Scoping) | ✅ PASS | Each module owns its state |
| Rule 10 (Handoff Checklist) | ⚠️ PARTIAL | Phase 5 has checklist, but missing in other phases |

### 5. Claims Verification

| Claim | Verification | Status |
|-------|--------------|--------|
| musl 1.2.5 is latest | Web search confirmed | ✅ CORRECT |
| Custom kernel ~39,000 LOC | `find crates -name "*.rs" | wc -l` = 218 files, 41,055 LOC | ✅ CORRECT (actually more) |
| xtask ~3,000 LOC | initramfs alone is 1,472 LOC | ⚠️ UNDERESTIMATE |
| Linux submodule exists | `ls linux/` shows built kernel | ✅ CORRECT |
| BusyBox builder 636 LOC | `wc -l busybox.rs` = 635 | ✅ CORRECT |
| OpenRC builder 292 LOC | `wc -l openrc.rs` = 291 | ✅ CORRECT |
| Shell wrappers exist | 7 `*.sh` files in root | ✅ CORRECT |
| `configs/` doesn't exist | `ls configs/` = not found | ✅ CORRECT (needs creation) |
| `overlay/` doesn't exist | `ls overlay/` = not found | ✅ CORRECT (needs creation) |

### 6. Detailed Issues

## Issues Found

### Critical

None.

### Important

**1. Building musl from source may be unnecessary**

The plan proposes creating `xtask/src/build/musl.rs` to build musl from source. However:
- `busybox.rs` and `openrc.rs` already work with system `musl-gcc`
- Building musl adds ~200 LOC and another failure point
- Only benefit is "no system dependency" but QEMU is already required

**Recommendation**: Make musl builder optional. Default to system `musl-gcc`, fall back to built musl only if system musl-gcc is missing.

**2. LOC estimates for xtask are underestimated**

| Component | Claimed LOC | Actual LOC |
|-----------|-------------|------------|
| Initramfs builder | ~1,200 | 1,472 |
| Total xtask | ~3,000 | ~5,000+ |

Not a blocker, but adjust expectations.

**3. Plan duplicates existing structure in PLAN.md vs phase docs**

The main `PLAN.md` has 590 lines covering everything, while phase docs add another ~1,200 lines. Much overlap. Consider consolidating.

### Minor

**1. Version pinning inconsistency**

Plan says musl 1.2.5, Linux 6.19-rc5, BusyBox 1.36.1, OpenRC 0.54. Should confirm these are pinned in actual code or document where they're defined.

**2. Missing `.gitignore` content in plan**

Phase 4 shows `.gitignore` updates but doesn't mention `toolchain/musl-*/` pattern that would be needed for the new musl builder.

**3. Typo in phase-2.md**

Line 156 shows URL template `https://musl.libc.org/releases/musl-{version}.tar.gz` - the actual URL is `/releases/` not `/releases/musl-`.

## Recommendations

### Simplify the Plan

The plan is over-scoped. Given that Linux + OpenRC is already the default, the actual work is:

**Phase 1: Archive & Remove Dead Code**
```bash
git tag v1.0-custom-kernel
git checkout -b archive/custom-kernel && git checkout main
git rm -r crates/
rm -f run*.sh screenshot.sh test_libsyscall.sh
```

**Phase 2: Remove Custom Kernel Code Paths**
- Remove `--custom-kernel` flag from `main.rs`
- Remove `build_kernel_only()`, `build_kernel_verbose()` from `build/`
- Remove `run_qemu_term()` custom kernel path from `run.rs`
- Update Cargo.toml: `members = ["xtask"]`

**Phase 3: Documentation**
- Update CLAUDE.md (remove kernel development sections)
- Update README.md (new project identity)

**That's it.** Don't:
- Create `musl.rs` builder (system musl-gcc works)
- Rename `initramfs/` to `configs/` + `overlay/` (unnecessary churn)
- Create `init/init.sh` (BusyBox init + inittab already works)

### Verification

After each phase:
```bash
cargo build -p xtask
cargo xtask run --term  # Should boot to OpenRC shell
```

## Verdict

**PLAN NEEDS MAJOR REVISION**

### What the Plan Gets Right
- Goal: focus on the build system, remove custom kernel
- Archiving strategy (preserve history)
- Breaking changes over compatibility (Rule 5)

### What the Plan Gets Wrong

**1. Unnecessary additions:**
- `musl.rs` builder (system musl-gcc works)
- `configs/` + `overlay/` (initramfs/ already exists)
- `init/init.sh` (BusyBox init works)

**2. Critical gap: No structural reorganization**

The plan doesn't address the fundamental problem: **everything is scattered and misnamed.**

Current mess:
```
/                               # What is this project?
├── xtask/                      # "Task runner" but it's THE PRODUCT
│   └── src/build/initramfs/    # Crown jewel buried 3 levels deep
├── crates/                     # Dead code
├── initramfs/                  # Config (why separate from xtask?)
├── linux/                      # Submodule
├── toolchain/                  # Build outputs
├── .external-kernels/          # Cruft
├── scripts/                    # Cruft
├── qemu/                       # Cruft
├── tmp/                        # Cruft
└── ...20+ scattered directories
```

**The initramfs builder should be the MAIN crate, not buried in xtask.**

### Proposed Structure

```
levitate/
├── src/                        # Main binary - "levitate" not "xtask"
│   ├── main.rs                 # CLI entry point
│   ├── lib.rs                  # Library interface (for programmatic use)
│   ├── builder/                # THE PRODUCT - distro builder
│   │   ├── mod.rs
│   │   ├── initramfs/          # Initramfs builder (extracted)
│   │   │   ├── mod.rs
│   │   │   ├── cpio.rs
│   │   │   ├── manifest.rs
│   │   │   └── tui.rs
│   │   ├── linux.rs            # Linux kernel builder
│   │   ├── busybox.rs          # BusyBox builder
│   │   ├── openrc.rs           # OpenRC builder
│   │   └── iso.rs              # ISO builder
│   ├── qemu/                   # QEMU runner (supplemental)
│   └── support/                # Utilities
│
├── config/                     # All configuration in one place
│   ├── initramfs.toml          # Initramfs manifest
│   ├── files/                  # Files to include (was initramfs/files/)
│   │   └── etc/
│   ├── linux.defconfig         # Kernel config
│   └── busybox.config          # BusyBox config
│
├── linux/                      # Git submodule (kernel source)
├── toolchain/                  # Build outputs (gitignored)
│   ├── busybox-out/
│   ├── openrc-out/
│   └── ...
│
├── docs/                       # Documentation
│   └── planning/
├── tests/                      # Test files
├── .teams/                     # Team logs
│
├── Cargo.toml                  # Single crate, not workspace
├── CLAUDE.md
└── README.md
```

### Key Changes

| From | To | Why |
|------|-----|-----|
| `xtask/` | `src/` | It's not a task runner, it's the product |
| `xtask/src/build/initramfs/` | `src/builder/initramfs/` | Promote to top-level |
| `initramfs/` | `config/` | Consolidate all configs |
| `crates/` | (delete) | Dead code |
| `.external-kernels/` | (delete) | Cruft |
| `scripts/`, `qemu/`, `tmp/` | (delete) | Cruft |

### Binary Name

```bash
# Before (confusing)
cargo xtask build initramfs

# After (clear)
cargo run -- build initramfs
# Or after install:
levitate build initramfs
```

## Handoff Notes

**The plan needs revision before execution.**

### What Needs to Happen

**Phase 1: Clean up cruft**
```bash
git rm -r crates/
git rm -r .external-kernels/
rm -rf scripts/ qemu/ tmp/ .idea/ .vscode/ .windsurf/
rm -f run*.sh screenshot.sh test_libsyscall.sh
```

**Phase 2: Restructure (the missing piece)**
```bash
# Move xtask to root src/
mv xtask/src src/
mv xtask/Cargo.toml ./  # Merge with root Cargo.toml

# Consolidate config
mv initramfs/ config/
mv config/initramfs.toml config/
mv config/files/ config/files/  # Already there

# Rename build/ to builder/ (semantics)
mv src/build src/builder
```

**Phase 3: Update imports and Cargo.toml**
- Change package name from `xtask` to `levitate`
- Update all `use crate::build::` to `use crate::builder::`
- Update CLI help text

**Phase 4: Documentation**
- Update CLAUDE.md for new structure
- Update README.md with new identity

### Why This Matters

The current `xtask` naming suggests this is auxiliary tooling. It's not - **it's the entire product**. A Linux distro builder deserves to be:

1. Named properly (`levitate` not `xtask`)
2. Structured clearly (builder at top level)
3. Not buried in nested directories

### Questions for User

Before implementing, clarify:
1. Should it be a library crate (`levitate-builder`) + binary crate (`levitate`)? Or single binary?
2. Keep `cargo xtask` alias for backwards compatibility during transition?
3. Target binary name: `levitate` or `levitate-builder`?

## References

- Plan: `docs/planning/levitate-v2/PLAN.md` (needs revision)
- Current state: TEAM_475 (Linux+OpenRC is default)
- This review: TEAM_477
