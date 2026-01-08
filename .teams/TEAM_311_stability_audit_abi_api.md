# TEAM_311: ABI Stability Refactor - crates/abi + syscall migration

## Objective
Create `crates/abi` as single source of truth. Migrate custom syscalls to Linux ABI. Break code, fix callsites.

## Status: Planning Complete - Ready for Implementation

## Decisions (USER Approved)
1. **Create `crates/abi`**: YES
2. **Custom syscalls**: FULL MIGRATION to Linux clone/execve
3. **Breaking changes**: BREAK THE CODE, FIX THE CALLSITES (no shims)

## Refactor Plan Created
```
docs/planning/stability-maturation/
├── phase-1.md          # Discovery and Safeguards
├── phase-1-step-1.md   # Inventory current state
├── phase-1-step-2.md   # Create crates/abi foundation
├── phase-1-step-3.md   # Add regression tests
├── phase-2.md          # Structural Extraction
├── phase-3.md          # Migration
├── phase-4.md          # Cleanup
└── phase-5.md          # Hardening and Handoff
```

## Key Changes Planned
1. Create `crates/abi` with errno, syscall numbers, ABI structs
2. Remove custom syscalls: Spawn (1000), SpawnArgs (1001)
3. Migrate to Linux pattern: `fork()` + `execve()`
4. Delete duplicate definitions in kernel/userspace
5. Add compile-time ABI verification tests

## Handoff Checklist (Rule 10)
- [x] Project builds cleanly (x86_64 + aarch64)
- [x] All library tests pass (`cargo test -p los_abi`)
- [x] Behavioral regression tests: N/A (no behavior changes)
- [x] Team file updated with progress
- [x] Remaining TODOs documented

## Completed Work
- [x] `crates/abi` created with SyscallNumber enum
- [x] Kernel migrated to use `los_abi::SyscallNumber` (both architectures)
- [x] `goblin` dependency added for future ELF refactor
- [x] Library audit documented in `docs/planning/stability-maturation/library-audit.md`
- [x] Deferred items documented in `phase-2.md` Section 4
- [x] README.md updated with DEFERRED section for future teams

## Deferred Items (Documented in phase-2.md Section 4)

> **Future teams**: See `docs/planning/stability-maturation/phase-2.md` Section 4 for detailed
> guidance on approaching these boot-critical refactors.

| Item | Risk | Status |
|------|------|--------|
| ELF → goblin | HIGH | Dependency added, refactor deferred |
| GDT/IDT → x86_64 crate | CRITICAL | Not started |
| Multiboot2 → multiboot2 crate | MEDIUM | Not started |

## Known Warnings (Intentional)
```
warning: use of deprecated unit variant `los_abi::SyscallNumber::Spawn`
warning: use of deprecated unit variant `los_abi::SyscallNumber::SpawnArgs`
```
These are **intentional** - they remind Phase 3 to remove these syscalls after implementing clone+exec.

## Next Steps for Future Teams

### Priority 1: Kernel Fork/Exec (Unblocks Phase 3)
1. Implement fork-style `sys_clone` (copy address space, not share)
   - Current: `kernel/src/syscall/process.rs:424` returns ENOSYS for non-thread clones
2. Implement proper `sys_exec` (replace current process with new ELF)
   - Current: `kernel/src/syscall/process.rs:139` is a stub
3. Then migrate spawn() callsites → fork+exec
4. Then remove Spawn/SpawnArgs from los_abi

### Priority 2: Deferred Library Replacements
See `docs/planning/stability-maturation/phase-2.md` Section 4:
- ELF → goblin (dependency added, refactor needed)
- GDT/IDT → x86_64 crate (boot-critical)
- Multiboot2 → multiboot2 crate

### Priority 3: Remaining Cleanup
- Extract ABI data structures to los_abi (Stat, Termios, flags)
- Delete userspace errno/sysno files (after los_abi integration)
