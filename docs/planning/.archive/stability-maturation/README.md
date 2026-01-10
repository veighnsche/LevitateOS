# Stability Maturation Plan

**TEAM_311**: ABI Stability & Library Consolidation
**Created**: 2026-01-08
**Last Updated**: 2026-01-08
**Status**: In Progress (Phase 2 partially complete)

---

## Overview

This plan addresses two critical issues:
1. **ABI Drift** - Syscall numbers defined in 3 places, causing crashes
2. **Hand-Rolled Code** - Reinventing wheels when battle-tested crates exist

## Documents

| Document | Description | Status |
|----------|-------------|--------|
| [phase-1.md](phase-1.md) | Discovery and Safeguards | ✅ Complete |
| [phase-2.md](phase-2.md) | Structural Extraction + Library Replacements | ⚠️ Partial (see deferred) |
| [phase-3.md](phase-3.md) | Migration (spawn → clone+exec) | ⏸️ Pending |
| [phase-4.md](phase-4.md) | Cleanup + Delete Hand-Rolled Code | ⏸️ Pending |
| [phase-5.md](phase-5.md) | Hardening and Handoff | ⏸️ Pending |
| [inventory.md](inventory.md) | Callsite inventory for migration | ✅ Complete |
| [library-audit.md](library-audit.md) | Hand-rolled vs existing crates | ✅ Complete |

---

## Key Deliverables

### 1. ABI Consolidation
- [x] Create `crates/abi` with `SyscallNumber` enum
- [x] Add tests verifying values match `linux-raw-sys`
- [x] Migrate kernel to import from `los_abi` (TEAM_311)
- [ ] Remove custom syscalls (Spawn, SpawnArgs) → Phase 3
- [ ] Implement `clone()` + `execve()` pattern → Phase 3

### 2. Library Replacements

| Hand-Rolled | Replacement | Status |
|-------------|-------------|--------|
| `kernel/src/loader/elf.rs` | `goblin` crate | 🔶 DEFERRED |
| `crates/hal/x86_64/gdt.rs` | `x86_64::structures::gdt` | 🔶 DEFERRED |
| `crates/hal/x86_64/idt.rs` | `x86_64::structures::idt` | 🔶 DEFERRED |
| `crates/hal/x86_64/multiboot2.rs` | `multiboot2` crate | 🔶 DEFERRED |
| `crates/utils/cpio.rs` | `cpio` crate (if no_std) | ⏸️ Low priority |

### 3. Already Using Good Crates ✅
`virtio-drivers`, `embedded-graphics`, `bitflags`, `spin`, `hashbrown`, `x86_64`, `aarch64-cpu`, `fdt`, `acpi`, `aml`, `linux-raw-sys`

---

## ⚠️ DEFERRED ITEMS (Boot-Critical)

> **IMPORTANT FOR FUTURE TEAMS**: The following items were identified during the library audit
> but deferred because they touch boot-critical code paths. See `phase-2.md` Section 4 for
> detailed guidance on how to approach these.

| Item | Risk Level | Why Deferred |
|------|------------|--------------|
| ELF → goblin | HIGH | 520 lines, deep memory management integration |
| GDT/IDT → x86_64 crate | CRITICAL | Boot fails immediately if wrong |
| Multiboot2 → multiboot2 crate | MEDIUM | x86_64 boot path only |

**Dependency added but refactor not done**: `goblin = "0.9"` is in `kernel/Cargo.toml`

---

## Phase Summary

```
Phase 1 ✅ COMPLETE
├── Inventory callsites
├── Create crates/abi skeleton
├── Audit for hand-rolled code
└── Document library replacements

Phase 2 ⚠️ PARTIAL (deferred items remain)
├── ✅ Extract SyscallNumber to los_abi
├── ✅ Kernel imports from los_abi
├── 🔶 DEFERRED: Replace ELF parsing with goblin
├── 🔶 DEFERRED: Replace GDT/IDT with x86_64 crate
└── ⏸️ PENDING: Extract ABI data structures

Phase 3 🔶 BLOCKED (kernel work required)
├── ⚠️ BLOCKER: sys_clone only supports threads, not fork
├── ⚠️ BLOCKER: sys_exec is a stub (returns ENOSYS)
├── Need: Implement fork-style clone + execve in kernel
└── Then: Migrate spawn callsites, remove Spawn/SpawnArgs

Phase 4 ✅ PARTIAL (what's ready is done)
├── ✅ Removed SyscallNumber from kernel arch modules (-235 lines)
├── ✅ Kernel now imports from los_abi
├── 🔶 BLOCKED: Delete hand-rolled code (needs deferred items)
└── 🔶 BLOCKED: Remove Spawn/SpawnArgs (needs Phase 3)

Phase 5 ✅ COMPLETE
├── ✅ Regression testing (kernel builds, tests pass)
├── ✅ Documentation updated
├── ✅ Handoff checklist in team file
└── ✅ Blockers documented for future teams
```

---

## Quick Reference

```bash
# Run library tests
cargo test -p los_abi

# Verify kernel builds
cargo build -p levitate-kernel --target x86_64-unknown-none

# Run golden tests (pre-existing failure on aarch64)
cargo xtask test --arch x86_64
```
