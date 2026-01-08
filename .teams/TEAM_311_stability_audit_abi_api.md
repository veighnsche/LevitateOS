# TEAM_311: Stability Audit - ABI/API Maturation

## Objective
Investigate lowest-level components causing fragility. One change shouldn't crash everything.

## Status: Analysis Complete

## Key Findings

### Root Cause: Syscall ABI Defined in 3 Places
The primary fragility comes from syscall numbers being defined separately in:
1. `kernel/src/arch/aarch64/mod.rs` - AArch64 kernel
2. `kernel/src/arch/x86_64/mod.rs` - x86_64 kernel  
3. `userspace/libsyscall/src/sysno.rs` - Userspace

**If any drift, userspace calls wrong syscall → crash.**

### Secondary Issues
1. **SyscallFrame layouts** differ per-arch with no compile-time verification
2. **Errno codes** duplicated in 3 places (kernel syscall/mod.rs x2, libsyscall)
3. **HAL globals** use `static mut` without proper synchronization
4. **MMU init** has implicit ordering dependencies

## Proposed Solution: `crates/abi`
Create single source of truth crate shared by kernel + userspace.

Full analysis: `docs/planning/stability-maturation/phase-1.md`

## Handoff
- [x] Project builds (no changes made)
- [x] All tests pass (no changes made)
- [x] Findings documented in `docs/planning/stability-maturation/phase-1.md`
- [ ] Implementation pending USER approval
